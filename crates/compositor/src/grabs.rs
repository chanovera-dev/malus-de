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
            tracing::info!("🛑 Soltado botón del ratón: finalizando MoveSurfaceGrab.");
            handle.unset_grab(self, data, event.serial, event.time, true);

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

                let cur_loc = handle.current_location();
                let win_loc = data.space.element_location(&self.window).unwrap_or_default();
                let win_geo = self.window.geometry();

                tracing::info!(
                    "🔍 Fin de arrastre: cursor={:?}, win_loc={:?}, win_size={:?}, output_geo={:?}",
                    cur_loc, win_loc, win_geo.size, output_geo
                );

                let cursor_margin = 40.0;
                let window_margin = 25;

                let is_left_edge = cur_loc.x <= (output_geo.loc.x as f64 + cursor_margin)
                    || win_loc.x <= (output_geo.loc.x + window_margin);

                let is_right_edge = cur_loc.x >= (output_geo.loc.x + output_geo.size.w) as f64 - cursor_margin
                    || (win_loc.x + win_geo.size.w) >= (output_geo.loc.x + output_geo.size.w - window_margin);

                let is_top_edge = cur_loc.y <= (output_geo.loc.y + usable_zone.loc.y) as f64 + cursor_margin
                    || win_loc.y <= (output_geo.loc.y + usable_zone.loc.y + window_margin);

                if is_top_edge {
                    tracing::info!("🪟 Aero Snap: borde superior detectado -> Maximizar");
                    Some((crate::state::SnapMode::Maximize, usable_zone))
                } else if is_left_edge {
                    tracing::info!("🪟 Aero Snap: borde izquierdo detectado -> Mitad Izquierda");
                    Some((crate::state::SnapMode::LeftHalf, usable_zone))
                } else if is_right_edge {
                    tracing::info!("🪟 Aero Snap: borde derecho detectado -> Mitad Derecha");
                    Some((crate::state::SnapMode::RightHalf, usable_zone))
                } else {
                    None
                }
            });

            if let Some((mode, zone)) = snap_action {
                crate::state::snap_window(data, &self.window, mode, zone);
            }
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
