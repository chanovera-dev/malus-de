use smithay::{
    desktop::Window,
    input::pointer::{
        AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
        GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
        GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
        GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent,
    },
    utils::{Logical, Point, Rectangle},
};

use crate::state::MalusState;

pub struct MoveSurfaceGrab {
    pub start_data: GrabStartData<MalusState>,
    pub window: Window,
    pub initial_window_location: Point<i32, Logical>,
}

impl PointerGrab<MalusState> for MoveSurfaceGrab {
    fn motion(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        _focus: Option<(
            <MalusState as smithay::input::SeatHandler>::PointerFocus,
            Point<f64, Logical>,
        )>,
        event: &MotionEvent,
    ) {
        // Mientras el grab está activo, ningún cliente tiene foco del puntero
        handle.motion(data, None, event);

        let delta = event.location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;

        data.space
            .map_element(self.window.clone(), new_location.to_i32_round(), true);

        // Actualizar overlay de previsualización Aero Snap en tiempo real
        let cur_loc = event.location;
        let preview = data.space.outputs().next().and_then(|output| {
            let (zone, output_geo) = {
                let layer_map = smithay::desktop::layer_map_for_output(output);
                (layer_map.non_exclusive_zone(), data.space.output_geometry(output).unwrap_or_default())
            };
            let usable_zone = if zone.size.w > 0 && zone.size.h > 0 {
                zone
            } else {
                Rectangle::new(
                    (0, 32).into(),
                    (output_geo.size.w, output_geo.size.h - 32).into(),
                )
            };

            evaluate_snap(cur_loc, output_geo, usable_zone)
                .and_then(|mode| preview_for_snap(mode, usable_zone))
        });

        data.snap_preview = preview;
    }

    fn relative_motion(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        focus: Option<(
            <MalusState as smithay::input::SeatHandler>::PointerFocus,
            Point<f64, Logical>,
        )>,
        event: &RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event);
    }

    fn button(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);
        if handle.current_pressed().is_empty() {
            // Limpiar previsualización inmediatamente al soltar
            data.snap_preview = None;

            let cur_loc = handle.current_location();
            let win_loc = data.space.element_location(&self.window).unwrap_or_default();
            let win_geo = self.window.geometry();
            let outputs_count = data.space.outputs().count();

            tracing::info!(
                "🛑 Soltado botón del ratón: cur_loc={:?}, win_loc={:?}, win_size={:?}, outputs_count={}",
                cur_loc, win_loc, win_geo.size, outputs_count
            );

            // Detección de Aero Snap al soltar la ventana en los bordes de la pantalla
            let snap_action = data.space.outputs().next().and_then(|output| {
                let layer_map = smithay::desktop::layer_map_for_output(output);
                let zone = layer_map.non_exclusive_zone();
                let output_geo = data.space.output_geometry(output).unwrap_or_default();
                let usable_zone = if zone.size.w > 0 && zone.size.h > 0 {
                    zone
                } else {
                    Rectangle::new(
                        (0, 32).into(),
                        (output_geo.size.w, output_geo.size.h - 32).into(),
                    )
                };

                let mode = evaluate_snap(cur_loc, output_geo, usable_zone);
                tracing::info!(
                    "🔍 Aero Snap evaluación al soltar: modo={:?}, cur_loc={:?}, output_geo={:?}, usable_zone={:?}",
                    mode, cur_loc, output_geo, usable_zone
                );

                mode.map(|m| (m, usable_zone))
            });

            if let Some((mode, zone)) = snap_action {
                crate::state::snap_window(data, &self.window, mode, zone);
            }

            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn axis(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        details: AxisFrame,
    ) {
        handle.axis(data, details);
    }

    fn frame(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
    ) {
        handle.frame(data);
    }

    fn gesture_swipe_begin(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GestureSwipeBeginEvent,
    ) {
        handle.gesture_swipe_begin(data, event);
    }

    fn gesture_swipe_update(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GestureSwipeUpdateEvent,
    ) {
        handle.gesture_swipe_update(data, event);
    }

    fn gesture_swipe_end(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GestureSwipeEndEvent,
    ) {
        handle.gesture_swipe_end(data, event);
    }

    fn gesture_pinch_begin(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GesturePinchBeginEvent,
    ) {
        handle.gesture_pinch_begin(data, event);
    }

    fn gesture_pinch_update(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GesturePinchUpdateEvent,
    ) {
        handle.gesture_pinch_update(data, event);
    }

    fn gesture_pinch_end(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GesturePinchEndEvent,
    ) {
        handle.gesture_pinch_end(data, event);
    }

    fn gesture_hold_begin(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GestureHoldBeginEvent,
    ) {
        handle.gesture_hold_begin(data, event);
    }

    fn gesture_hold_end(
        &mut self,
        data: &mut MalusState,
        handle: &mut PointerInnerHandle<'_, MalusState>,
        event: &GestureHoldEndEvent,
    ) {
        handle.gesture_hold_end(data, event);
    }

    fn start_data(&self) -> &GrabStartData<MalusState> {
        &self.start_data
    }

    fn unset(&mut self, _data: &mut MalusState) {}
}

pub fn evaluate_snap(
    cur_loc: Point<f64, Logical>,
    output_geo: Rectangle<i32, Logical>,
    usable_zone: Rectangle<i32, Logical>,
) -> Option<crate::state::SnapMode> {
    let edge_margin = 35.0;

    let is_left = cur_loc.x <= output_geo.loc.x as f64 + edge_margin;
    let is_right = cur_loc.x >= (output_geo.loc.x + output_geo.size.w) as f64 - edge_margin;
    let is_top = cur_loc.y <= (output_geo.loc.y + usable_zone.loc.y) as f64 + edge_margin;

    if is_left {
        Some(crate::state::SnapMode::LeftHalf)
    } else if is_right {
        Some(crate::state::SnapMode::RightHalf)
    } else if is_top {
        Some(crate::state::SnapMode::Maximize)
    } else {
        None
    }
}

pub fn preview_for_snap(
    mode: crate::state::SnapMode,
    usable_zone: Rectangle<i32, Logical>,
) -> Option<Rectangle<i32, Logical>> {
    match mode {
        crate::state::SnapMode::Maximize => Some(Rectangle::new(
            (usable_zone.loc.x + 8, usable_zone.loc.y + 8).into(),
            (usable_zone.size.w - 16, usable_zone.size.h - 16).into(),
        )),
        crate::state::SnapMode::LeftHalf => Some(Rectangle::new(
            (usable_zone.loc.x + 8, usable_zone.loc.y + 8).into(),
            (usable_zone.size.w / 2 - 12, usable_zone.size.h - 16).into(),
        )),
        crate::state::SnapMode::RightHalf => Some(Rectangle::new(
            (usable_zone.loc.x + usable_zone.size.w / 2 + 4, usable_zone.loc.y + 8).into(),
            (usable_zone.size.w - usable_zone.size.w / 2 - 12, usable_zone.size.h - 16).into(),
        )),
        crate::state::SnapMode::CenterRestore => None,
    }
}

