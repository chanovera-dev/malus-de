use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use anyhow::{Context, Result};
use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Axis, ButtonState, Event, InputEvent, KeyState, KeyboardKeyEvent,
            PointerAxisEvent, PointerButtonEvent,
        },
        renderer::{
            gles::GlesRenderer,
            utils::draw_render_elements,
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement, WaylandSurfaceTexture},
                Element, Kind,
            },
            Color32F, Frame, Renderer,
        },
        winit::{self, WinitEvent},
    },
    desktop::space::space_render_elements,
    input::{
        keyboard::{keysyms, FilterResult},
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        wayland_server::{
            protocol::wl_surface::WlSurface,
            Display, ListeningSocket,
        },
        winit::platform::pump_events::PumpStatus,
    },
    utils::{Rectangle, Scale, Transform, SERIAL_COUNTER},
    wayland::{
        compositor::{with_surface_tree_downward, SurfaceAttributes, TraversalAction},
        seat::WaylandFocus,
        shell::wlr_layer::Layer,
    },
};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge;
use tracing::{info, warn};
use malus_de_common::ipc::IpcMessage;

use crate::state::{ClientState, MalusState};

pub fn send_frames_surface_tree(surface: &WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

pub fn run_winit() -> Result<()> {
    info!("Inicializando Display Wayland de Malus DE...");
    let mut display: Display<MalusState> = Display::new().context("Error al crear Display Wayland")?;
    let mut state = MalusState::new(&display);

    let keyboard = state
        .seat
        .add_keyboard(Default::default(), 200, 200)
        .map_err(|e| anyhow::anyhow!("Error al inicializar teclado: {e:?}"))?;
    let pointer = state.seat.add_pointer();

    // Crear socket de escucha de clientes Wayland (wayland-1 .. 32)
    let listener = ListeningSocket::bind_auto("wayland", 1..33)
        .map_err(|e| anyhow::anyhow!("Error al bindear socket de Wayland: {e:?}"))?;
    let socket_name = listener
        .socket_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    info!("Socket Wayland escuchando activamente en: WAYLAND_DISPLAY={}", socket_name);
    state.socket_name = socket_name.clone();

    // Inicializar backend gráfico de Winit en ventana anidada
    let (mut backend, mut winit_event_loop) = winit::init::<GlesRenderer>()
        .map_err(|e| anyhow::anyhow!("Error al inicializar WinitGraphicsBackend: {e:?}"))?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Malus".to_string(),
            model: "Virtual Display (Nested)".to_string(),
        },
    );
    let _global = output.create_global::<MalusState>(&state.display_handle);
    output.change_current_state(Some(mode), Some(Transform::Normal), None, Some((0, 0).into()));
    output.set_preferred(mode);
    state.space.map_output(&output, (0, 0));

    info!("============================================================");
    info!("✨ MALUS DE: Compositor Wayland Smithay (Modo Anidado Activo)");
    info!("   Socket activo: WAYLAND_DISPLAY={}", socket_name);
    info!("   Prueba lanzar una app cliente en otra terminal:");
    info!("   WAYLAND_DISPLAY={} GDK_BACKEND=wayland gnome-calculator", socket_name);
    info!("============================================================");

    // Color acrílico oscuro de Malus DE (#18181b = rgba(24, 24, 27, 1.0))
    let dark_acrylic = Color32F::new(0.094, 0.094, 0.106, 1.0);
    let start_time = Instant::now();
    let mut clients = Vec::new();

    while state.is_running {
        let status = winit_event_loop.dispatch_new_events(|event| match event {
            WinitEvent::CloseRequested => {
                info!("Cierre de ventana anidada solicitado por el usuario.");
                state.is_running = false;
            }
            WinitEvent::Resized { size, .. } => {
                output.change_current_state(
                    Some(Mode { size, refresh: 60_000 }),
                    None,
                    None,
                    None,
                );
            }
            WinitEvent::Input(event) => match event {
                InputEvent::Keyboard { event } => {
                    let key = event.key_code().raw();
                    let is_pressed = event.state() == KeyState::Pressed;

                    keyboard.input::<(), _>(
                        &mut state,
                        event.key_code(),
                        event.state(),
                        SERIAL_COUNTER.next_serial(),
                        event.time_msec(),
                        |state, modifiers, keysym_handle| {
                            let sym = keysym_handle.modified_sym();
                            let has_super = modifiers.logo;
                            let has_alt = modifiers.alt;
                            let has_ctrl = modifiers.ctrl;

                            let is_space = sym == keysyms::KEY_space.into();
                            let is_k = sym == keysyms::KEY_k.into() || sym == keysyms::KEY_K.into();
                            let is_tab = sym == keysyms::KEY_Tab.into() || sym == keysyms::KEY_ISO_Left_Tab.into() || key == 23;
                            let is_w = sym == keysyms::KEY_w.into() || sym == keysyms::KEY_W.into();
                            let is_q = sym == keysyms::KEY_q.into() || sym == keysyms::KEY_Q.into();
                            let is_h = sym == keysyms::KEY_h.into() || sym == keysyms::KEY_H.into();
                            let is_m = sym == keysyms::KEY_m.into() || sym == keysyms::KEY_M.into();
                            let is_f4 = sym == keysyms::KEY_F4.into();
                            let is_f9 = sym == keysyms::KEY_F9.into();
                            let is_esc = sym == keysyms::KEY_Escape.into();
                            let is_left = sym == keysyms::KEY_Left.into() || key == 105 || key == 113;
                            let is_right = sym == keysyms::KEY_Right.into() || key == 106 || key == 114;

                            if is_pressed {
                                // 1. Atajos para alternar Spotlight:
                                // Super + Espacio (macOS nativo), Alt + Espacio, Ctrl + Espacio, Super + K
                                if (is_space && (has_super || has_alt || has_ctrl)) || (is_k && has_super) {
                                    info!("🔍 Atajo de Spotlight detectado (Super={}, Alt={}, Ctrl={}): alternando...",
                                        has_super, has_alt, has_ctrl);
                                    toggle_spotlight(&state.socket_name);
                                    return FilterResult::Intercept(());
                                }

                                // 2. Navegación en el App Switcher (Selector Visual ⌘Tab) mientras está activo
                                if state.app_switcher.is_active {
                                    if is_tab {
                                        let has_shift = modifiers.shift;
                                        let len = state.app_switcher.items.len();
                                        if len > 0 {
                                            if has_shift {
                                                state.app_switcher.selected_index = (state.app_switcher.selected_index + len - 1) % len;
                                            } else {
                                                state.app_switcher.selected_index = (state.app_switcher.selected_index + 1) % len;
                                            }
                                            info!("🔀 AppSwitcher ciclo: idx={}/{}", state.app_switcher.selected_index, len);
                                            state.ipc_broadcaster.broadcast(&IpcMessage::UpdateAppSwitcher {
                                                selected_index: state.app_switcher.selected_index,
                                            });
                                        }
                                        return FilterResult::Intercept(());
                                    } else if is_left {
                                        let len = state.app_switcher.items.len();
                                        if len > 0 {
                                            state.app_switcher.selected_index = (state.app_switcher.selected_index + len - 1) % len;
                                            state.ipc_broadcaster.broadcast(&IpcMessage::UpdateAppSwitcher {
                                                selected_index: state.app_switcher.selected_index,
                                            });
                                        }
                                        return FilterResult::Intercept(());
                                    } else if is_right {
                                        let len = state.app_switcher.items.len();
                                        if len > 0 {
                                            state.app_switcher.selected_index = (state.app_switcher.selected_index + 1) % len;
                                            state.ipc_broadcaster.broadcast(&IpcMessage::UpdateAppSwitcher {
                                                selected_index: state.app_switcher.selected_index,
                                            });
                                        }
                                        return FilterResult::Intercept(());
                                    } else if is_q {
                                        // Atajo secreto de macOS: ⌘Q mientras ⌘Tab está activo cierra la app seleccionada
                                        if !state.app_switcher.items.is_empty() {
                                            let target_id = state.app_switcher.items[state.app_switcher.selected_index].window_id;
                                            info!("🚪 AppSwitcher ⌘Q: cerrando ventana seleccionada id={}", target_id);
                                            state.close_window_by_id(target_id);
                                            state.app_switcher.items.remove(state.app_switcher.selected_index);
                                            if state.app_switcher.items.is_empty() {
                                                state.app_switcher.is_active = false;
                                                state.ipc_broadcaster.broadcast(&IpcMessage::HideAppSwitcher);
                                            } else {
                                                state.app_switcher.selected_index = state.app_switcher.selected_index.min(state.app_switcher.items.len() - 1);
                                                state.ipc_broadcaster.broadcast(&IpcMessage::ShowAppSwitcher {
                                                    items: state.app_switcher.items.clone(),
                                                    selected_index: state.app_switcher.selected_index,
                                                });
                                            }
                                        }
                                        return FilterResult::Intercept(());
                                    } else if is_h {
                                        // Atajo secreto de macOS: ⌘H mientras ⌘Tab está activo oculta/minimiza la app seleccionada
                                        if !state.app_switcher.items.is_empty() {
                                            let target_id = state.app_switcher.items[state.app_switcher.selected_index].window_id;
                                            info!("🪟 AppSwitcher ⌘H: ocultando ventana seleccionada id={}", target_id);
                                            state.hide_window_by_id(target_id);
                                            state.app_switcher.items[state.app_switcher.selected_index].is_minimized = true;
                                            state.ipc_broadcaster.broadcast(&IpcMessage::ShowAppSwitcher {
                                                items: state.app_switcher.items.clone(),
                                                selected_index: state.app_switcher.selected_index,
                                            });
                                        }
                                        return FilterResult::Intercept(());
                                    } else if is_esc {
                                        info!("❌ AppSwitcher cancelado con Escape");
                                        state.app_switcher.is_active = false;
                                        state.ipc_broadcaster.broadcast(&IpcMessage::HideAppSwitcher);
                                        return FilterResult::Intercept(());
                                    }
                                }

                                // 3. Invocación inicial de ⌘Tab / Super+Tab / Alt+Tab
                                if (has_super || has_alt) && is_tab {
                                    let items = state.build_app_switcher_items();
                                    if !items.is_empty() {
                                        let has_shift = modifiers.shift;
                                        let initial_idx = if items.len() > 1 {
                                            if has_shift { items.len() - 1 } else { 1 }
                                        } else {
                                            0
                                        };
                                        state.app_switcher.is_active = true;
                                        state.app_switcher.items = items.clone();
                                        state.app_switcher.selected_index = initial_idx;
                                        info!("🔀 AppSwitcher activado con {} ventanas (idx={})", items.len(), initial_idx);
                                        state.ipc_broadcaster.broadcast(&IpcMessage::ShowAppSwitcher {
                                            items,
                                            selected_index: initial_idx,
                                        });
                                        return FilterResult::Intercept(());
                                    }
                                }

                                // 4. Atajos universales de macOS: ⌘Q / ⌘W / Alt+F4 para cerrar ventana activa
                                if (has_super && (is_w || is_q)) || (has_alt && is_f4) {
                                    info!("🚪 Atajo macOS ⌘Q / ⌘W / Alt+F4: cerrando ventana activa");
                                    state.close_active_window();
                                    return FilterResult::Intercept(());
                                }

                                // 5. Atajos universales de macOS: ⌘H / ⌘M / Alt+F9 para ocultar / minimizar con Genie Effect
                                if (has_super && (is_m || is_h)) || (has_alt && is_f9) {
                                    info!("🪟 Atajo macOS ⌘H / ⌘M / Alt+F9: minimizando ventana activa vía Genie Effect");
                                    state.minimize_active_window();
                                    return FilterResult::Intercept(());
                                }

                                // 6. Atajos de Aero Snap / Tiling (Super o Alt + Flechas)
                                if has_super || has_alt {
                                    let is_up = sym == keysyms::KEY_Up.into() || key == 103 || key == 111;
                                    let is_down = sym == keysyms::KEY_Down.into() || key == 108 || key == 116;

                                    if is_left || is_right || is_up || is_down {
                                        let active_window = state.get_active_window();
                                        let snap_info = active_window.and_then(|win| {
                                            state.space.outputs().next().map(|output| {
                                                let layer_map = smithay::desktop::layer_map_for_output(output);
                                                let zone = layer_map.non_exclusive_zone();
                                                let output_geo = state.space.output_geometry(output).unwrap_or_default();
                                                let usable_zone = if zone.size.w > 0 && zone.size.h > 0 {
                                                    zone
                                                } else {
                                                    Rectangle::new(
                                                        (0, 32).into(),
                                                        (output_geo.size.w, output_geo.size.h - 32).into(),
                                                    )
                                                };

                                                let snap_mode = if is_left {
                                                    crate::state::SnapMode::LeftHalf
                                                } else if is_right {
                                                    crate::state::SnapMode::RightHalf
                                                } else if is_up {
                                                    crate::state::SnapMode::Maximize
                                                } else {
                                                    crate::state::SnapMode::CenterRestore
                                                };

                                                (win, snap_mode, usable_zone)
                                            })
                                        });

                                        if let Some((win, snap_mode, usable_zone)) = snap_info {
                                            info!("🪟 Atajo Aero Snap activado: modo={:?}, ventana activa encontrada", snap_mode);
                                            crate::state::snap_window(state, &win, snap_mode, usable_zone);
                                            return FilterResult::Intercept(());
                                        }
                                    }
                                }
                            } else {
                                // 7. Liberación de teclas (Key Release)
                                // Cuando el usuario suelta Super o Alt, confirmar selección en App Switcher
                                if state.app_switcher.is_active && !modifiers.logo && !modifiers.alt {
                                    info!("🎯 AppSwitcher commit: soltado Super/Alt, enfocando ventana");
                                    state.app_switcher.is_active = false;
                                    state.ipc_broadcaster.broadcast(&IpcMessage::HideAppSwitcher);
                                    if !state.app_switcher.items.is_empty() {
                                        let target_id = state.app_switcher.items[state.app_switcher.selected_index].window_id;
                                        state.switch_to_window(target_id);
                                    }
                                    return FilterResult::Intercept(());
                                }
                            }
                            FilterResult::Forward
                        },
                    );
                }
                InputEvent::PointerMotionAbsolute { event } => {
                    let output_geo = state.space.output_geometry(&output).unwrap_or_default();
                    let point = event.position_transformed(output_geo.size);
                    let serial = SERIAL_COUNTER.next_serial();

                    let layer_target = {
                        let layer_map = smithay::desktop::layer_map_for_output(&output);
                        let under_layer = layer_map.layer_under(Layer::Overlay, point)
                            .or_else(|| layer_map.layer_under(Layer::Top, point));

                        under_layer.and_then(|layer| {
                            layer_map.layer_geometry(layer).and_then(|geo| {
                                let local_point = point - geo.loc.to_f64();
                                let hit = if !state.active_popups.is_empty() {
                                    layer.surface_under(local_point, smithay::desktop::WindowSurfaceType::POPUP)
                                } else {
                                    None
                                };
                                let hit = hit.or_else(|| {
                                    if layer.namespace() == "malus-top-bar" && local_point.y >= 32.0 {
                                        None
                                    } else {
                                        layer.surface_under(local_point, smithay::desktop::WindowSurfaceType::TOPLEVEL)
                                    }
                                });

                                hit.map(|(surf, sub_loc)| (surf, (geo.loc + sub_loc).to_f64()))
                            })
                        })
                    };

                    let under_target = if let Some((surf, focus_loc)) = layer_target {
                        Some((surf, focus_loc))
                    } else if let Some((window, loc)) = state.space.element_under(point) {
                        let local_point = point - loc.to_f64();
                        window
                            .surface_under(local_point, smithay::desktop::WindowSurfaceType::all())
                            .map(|(surf, sub_loc)| (surf, (loc + sub_loc).to_f64()))
                    } else {
                        None
                    };

                    pointer.motion(
                        &mut state,
                        under_target,
                        &MotionEvent {
                            location: point,
                            serial,
                            time: event.time_msec(),
                        },
                    );
                    pointer.frame(&mut state);
                }
                InputEvent::PointerButton { event } => {
                    let is_pressed = event.state() == ButtonState::Pressed;
                    let point = pointer.current_location();

                    let layer_target = {
                        let layer_map = smithay::desktop::layer_map_for_output(&output);
                        let under_layer = layer_map.layer_under(Layer::Overlay, point)
                            .or_else(|| layer_map.layer_under(Layer::Top, point));

                        under_layer.and_then(|layer| {
                            layer_map.layer_geometry(layer).and_then(|geo| {
                                let local_point = point - geo.loc.to_f64();
                                let hit = if !state.active_popups.is_empty() {
                                    layer.surface_under(local_point, smithay::desktop::WindowSurfaceType::POPUP)
                                } else {
                                    None
                                };
                                let hit = hit.or_else(|| {
                                    if layer.namespace() == "malus-top-bar" && local_point.y >= 32.0 {
                                        None
                                    } else {
                                        layer.surface_under(local_point, smithay::desktop::WindowSurfaceType::TOPLEVEL)
                                    }
                                });

                                hit.map(|(surf, sub_loc)| {
                                    (
                                        surf.clone(),
                                        (geo.loc + sub_loc).to_f64(),
                                        layer.namespace().to_string(),
                                        layer.can_receive_keyboard_focus(),
                                    )
                                })
                            })
                        })
                    };

                    let (under_target, layer_info, regular_win) = if let Some((surf, focus_loc, ns, can_kb)) = layer_target {
                        (Some((surf.clone(), focus_loc)), Some((surf, ns, can_kb)), None)
                    } else if let Some((window, loc)) = state.space.element_under(point) {
                        let local_point = point - loc.to_f64();
                        let target = window
                            .surface_under(local_point, smithay::desktop::WindowSurfaceType::all())
                            .map(|(surf, sub_loc)| (surf, (loc + sub_loc).to_f64()));
                        (target, None, Some((window.clone(), loc)))
                    } else {
                        (None, None, None)
                    };

                    // Sincronizar foco del puntero antes de procesar el botón para que el cliente reciba enter/button
                    let motion_serial = SERIAL_COUNTER.next_serial();
                    pointer.motion(
                        &mut state,
                        under_target,
                        &MotionEvent {
                            location: point,
                            serial: motion_serial,
                            time: event.time_msec(),
                        },
                    );

                    let btn_serial = SERIAL_COUNTER.next_serial();
                    if is_pressed {
                        // Si hay popups activos y el clic NO fue sobre el popup, descartamos los popups
                        let clicked_on_popup = layer_info.as_ref().map(|(s, _, _)| {
                            state.active_popups.iter().any(|p| p.wl_surface() == s)
                        }).unwrap_or(false);

                        if !clicked_on_popup && !state.active_popups.is_empty() {
                            state.dismiss_all_popups();
                        }

                        // Liberar cualquier grab residual para que los clics nunca se queden congelados
                        if pointer.is_grabbed() {
                            tracing::warn!("⚠️ Grab de puntero residual detectado al hacer clic; liberando grab");
                            pointer.unset_grab(&mut state, btn_serial, event.time_msec());
                        }
                        if keyboard.is_grabbed() {
                            tracing::warn!("⚠️ Grab de teclado residual detectado al hacer clic; liberando grab");
                            keyboard.unset_grab(&mut state);
                        }

                        if let Some((surf, ns, can_kb)) = layer_info {
                            info!("🎯 Clic de ratón sobre layer surface: namespace={}, pos={:?}", ns, point);
                            if can_kb {
                                keyboard.set_focus(&mut state, Some(surf), btn_serial);
                            }
                        } else if let Some((window, loc)) = regular_win {
                            info!("🖱️ Clic en ventana regular: button={:#x}, pos={:?}, win_loc={:?}, geo={:?}", event.button_code(), point, loc, window.geometry());
                            let was_already_top = state.space.elements().last().map(|w| w == &window).unwrap_or(false);
                            if !was_already_top {
                                state.space.raise_element(&window, true);
                                // NOTA: No enviamos send_configure() durante Pressed para evitar que GTK4
                                // cancele el GtkGestureClick de los botones de título (cerrar/minimizar/maximizar).
                            }
                            if let Some(surface) = window.wl_surface() {
                                if keyboard.current_focus().as_ref() != Some(surface.as_ref()) {
                                    keyboard.set_focus(&mut state, Some(surface.clone().into_owned()), btn_serial);
                                    state.notify_active_app(Some(&surface));
                                }
                            }

                            // Si Super o Alt está presionado y es clic izquierdo (0x110 = BTN_LEFT): arrastrar ventana inmediatamente desde cualquier punto
                            let has_super = keyboard.modifier_state().logo;
                            let has_alt = keyboard.modifier_state().alt;
                            if (has_super || has_alt) && event.button_code() == 0x110 {
                                info!("🚀 Atajo Super/Alt + Clic: iniciando arrastre directo de ventana desde {:?}", loc);
                                let initial_window_location = loc;
                                let start_data = smithay::input::pointer::GrabStartData {
                                    focus: None,
                                    button: event.button_code(),
                                    location: point,
                                };
                                let grab = crate::grabs::MoveSurfaceGrab {
                                    start_data,
                                    window: window.clone(),
                                    initial_window_location,
                                };
                                pointer.set_grab(&mut state, grab, btn_serial, smithay::input::pointer::Focus::Clear);
                            }

                            // Si Super o Alt está presionado y es clic derecho (0x111 = BTN_RIGHT): redimensionar ventana directamente desde cualquier punto
                            if (has_super || has_alt) && event.button_code() == 0x111 {
                                let initial_window_location = loc;
                                let initial_window_size = window.geometry().size;

                                let rel_x = point.x - loc.x as f64;
                                let rel_y = point.y - loc.y as f64;
                                let w = initial_window_size.w as f64;
                                let h = initial_window_size.h as f64;

                                let is_right = rel_x >= w / 2.0;
                                let is_bottom = rel_y >= h / 2.0;

                                let edges = match (is_right, is_bottom) {
                                    (true, true) => ResizeEdge::BottomRight,
                                    (false, true) => ResizeEdge::BottomLeft,
                                    (true, false) => ResizeEdge::TopRight,
                                    (false, false) => ResizeEdge::TopLeft,
                                };

                                info!("📐 Atajo Super/Alt + Clic Derecho: iniciando redimensión de ventana ({:?})", edges);
                                let start_data = smithay::input::pointer::GrabStartData {
                                    focus: None,
                                    button: event.button_code(),
                                    location: point,
                                };
                                let grab = crate::grabs::ResizeSurfaceGrab {
                                    start_data,
                                    window: window.clone(),
                                    edges,
                                    initial_window_location,
                                    initial_window_size,
                                    last_window_size: initial_window_size,
                                };
                                pointer.set_grab(&mut state, grab, btn_serial, smithay::input::pointer::Focus::Clear);
                            }
                        } else {
                            info!("🖱️ Clic en espacio libre del escritorio: button={:#x}, pos={:?}", event.button_code(), point);
                            state.notify_active_app(None);
                        }
                    }

                    pointer.button(
                        &mut state,
                        &ButtonEvent {
                            serial: btn_serial,
                            time: event.time_msec(),
                            button: event.button_code(),
                            state: event.state(),
                        },
                    );
                    pointer.frame(&mut state);
                }
                InputEvent::PointerAxis { event } => {
                    let mut frame = AxisFrame::new(event.time_msec()).source(event.source());
                    if let Some(horizontal) = event.amount(Axis::Horizontal) {
                        frame = frame.value(Axis::Horizontal, horizontal);
                        if let Some(discrete) = event.amount_v120(Axis::Horizontal) {
                            frame = frame.v120(Axis::Horizontal, discrete as i32);
                        }
                    }
                    if let Some(vertical) = event.amount(Axis::Vertical) {
                        frame = frame.value(Axis::Vertical, vertical);
                        if let Some(discrete) = event.amount_v120(Axis::Vertical) {
                            frame = frame.v120(Axis::Vertical, discrete as i32);
                        }
                    }
                    if event.amount(Axis::Horizontal).is_some() || event.amount(Axis::Vertical).is_some() {
                        pointer.axis(&mut state, frame);
                        pointer.frame(&mut state);
                    }
                }
                _ => {}
            },
            _ => {}
        });

        if !state.is_running {
            break;
        }

        match status {
            PumpStatus::Continue => (),
            PumpStatus::Exit(_) => break,
        }

        let size = backend.window_size();
        let damage = Rectangle::new((0, 0).into(), size);

        // Renderizado del frame
        {
            if let Ok((renderer, mut framebuffer)) = backend.bind() {
                let elements = match space_render_elements::<
                    GlesRenderer,
                    smithay::desktop::Window,
                    _,
                >(renderer, [&state.space], &output, 1.0)
                {
                    Ok(elems) => {
                        if !elems.is_empty() {
                            static ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                            if !ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                                info!("🎨 Frame con elementos visibles renderizado: total elementos={}", elems.len());
                            }
                        }
                        elems
                    }
                    Err(err) => {
                        warn!("Error al obtener render elements de space: {:?}", err);
                        Vec::new()
                    }
                };

                let now = Instant::now();
                let mut anim_data = Vec::new();
                for anim in &state.active_animations {
                    let elems = render_elements_from_surface_tree::<GlesRenderer, WaylandSurfaceRenderElement<GlesRenderer>>(
                        renderer,
                        &anim.surface,
                        (0, 0),
                        Scale::from(1.0),
                        1.0,
                        Kind::Unspecified,
                    );
                    if elems.is_empty() {
                        warn!("⚠️ [Genie] anim_data: elems está VACÍO para anim id={}", anim.window_id);
                    }
                    anim_data.push((anim.clone(), elems));
                }

                if let Ok(mut frame) = renderer.render(&mut framebuffer, size, Transform::Flipped180) {
                    let _ = frame.clear(dark_acrylic, &[damage]);
                    let _ = draw_render_elements(&mut frame, 1.0, &elements, &[damage]);

                    // Dibujar overlay translúcido Aero Glass de previsualización al arrastrar cerca del borde
                    if let Some(preview) = state.snap_preview {
                        let preview_phys = preview.to_physical_precise_round(1.0);
                        // Fondo azul translúcido (#0078d4 con 35% de opacidad)
                        let _ = frame.draw_solid(
                            preview_phys,
                            &[Rectangle::from_size(preview_phys.size)],
                            Color32F::new(0.0, 0.47, 0.83, 0.35),
                        );

                        // Borde resaltado azul de 3px
                        let border_color = Color32F::new(0.38, 0.72, 1.0, 0.85);
                        let b_thickness = 3;
                        let b_top = Rectangle::new(preview_phys.loc, (preview_phys.size.w, b_thickness).into());
                        let b_bottom = Rectangle::new(
                            (preview_phys.loc.x, preview_phys.loc.y + preview_phys.size.h - b_thickness).into(),
                            (preview_phys.size.w, b_thickness).into(),
                        );
                        let b_left = Rectangle::new(preview_phys.loc, (b_thickness, preview_phys.size.h).into());
                        let b_right = Rectangle::new(
                            (preview_phys.loc.x + preview_phys.size.w - b_thickness, preview_phys.loc.y).into(),
                            (b_thickness, preview_phys.size.h).into(),
                        );

                        let _ = frame.draw_solid(b_top, &[Rectangle::from_size(b_top.size)], border_color);
                        let _ = frame.draw_solid(b_bottom, &[Rectangle::from_size(b_bottom.size)], border_color);
                        let _ = frame.draw_solid(b_left, &[Rectangle::from_size(b_left.size)], border_color);
                        let _ = frame.draw_solid(b_right, &[Rectangle::from_size(b_right.size)], border_color);
                    }

                    // Renderizado auténtico de Efecto Lámpara de Aladino (Genie Effect / Magic Lamp)
                    // Renderizado auténtico de Efecto Lámpara de Aladino (Genie Effect / Magic Lamp)
                    // Implementación canónica con física elástica, sombra volumétrica y volumen 3D
                    for (anim, elems) in &anim_data {
                        let elapsed = now.duration_since(anim.start_time);
                        let raw_t = (elapsed.as_secs_f32() / anim.duration.as_secs_f32()).clamp(0.0, 1.0);

                        // Curva de física orgánica con inercia elástica (Spring Easing)
                        let t = if anim.is_minimizing {
                            if raw_t < 0.65 {
                                (raw_t / 0.65).clamp(0.0, 1.0).powf(1.35) * 0.65
                            } else {
                                let p = ((raw_t - 0.65) / 0.35).clamp(0.0, 1.0);
                                0.65 + (1.0 - (1.0 - p).powf(2.6)) * 0.35
                            }
                        } else {
                            if raw_t < 0.75 {
                                (raw_t / 0.75).clamp(0.0, 1.0).powf(0.85) * 0.75
                            } else {
                                let p = ((raw_t - 0.75) / 0.25).clamp(0.0, 1.0);
                                0.75 + (1.0 - (1.0 - p).powf(2.2)) * 0.25
                            }
                        };

                        let split = 0.30f32;
                        let (k, j) = if anim.is_minimizing {
                            let k = if t <= split { t / split } else { 1.0 };
                            let j = if t > split { (t - split) / (1.0 - split) } else { 0.0 };
                            (k, j)
                        } else {
                            // Restauración: emerge de la lámpara (j: 1 -> 0), luego su base se asienta (k: 1 -> 0)
                            let j = 1.0 - if t <= (1.0 - split) { t / (1.0 - split) } else { 1.0 };
                            let k = 1.0 - if t > (1.0 - split) { (t - (1.0 - split)) / split } else { 0.0 };
                            (k, j)
                        };

                        info!("🎬 [Genie Frame] id={}, is_min={}, t={:.2}, k={:.2}, j={:.2}, elems={}",
                            anim.window_id, anim.is_minimizing, t, k, j, elems.len());

                        let win_x = anim.window_rect.loc.x as f32;
                        let win_y = anim.window_rect.loc.y as f32;
                        let win_w = (anim.window_rect.size.w as f32).max(40.0);
                        let win_h = (anim.window_rect.size.h as f32).max(40.0);

                        let slot_x = anim.dock_slot_rect.loc.x as f32;
                        let slot_y = anim.dock_slot_rect.loc.y as f32;
                        let slot_w = (anim.dock_slot_rect.size.w as f32).max(24.0);
                        let slot_h = (anim.dock_slot_rect.size.h as f32).max(24.0);

                        // Coordenadas objetivo en el Dock (alineación precisa al slot registrado)
                        let target_w = slot_w.max(32.0);
                        let target_x = slot_x;
                        let target_y = slot_y + slot_h * 0.40;

                        // Altura de extensión y desplazamiento de fase Compiz
                        let expand_h = (target_y - win_y - win_h).max(10.0);
                        let full_height = (target_y - win_y) - expand_h * (1.0 - k);
                        let current_height = (full_height * (1.0 - j)).max(1.0);
                        let offset_y = (target_y - win_y) * j;

                        // Función de deformación para una rebanada normalizada v en [0.0, 1.0]
                        // (v = 0.0: borde superior de la ventana; v = 1.0: borde inferior)
                        let deform_slice = |v: f32| -> (f32, f32, f32) {
                            let local_y = v * current_height;
                            let screen_y = win_y + local_y + offset_y;

                            // Ancho de la rebanada (estrechamiento hacia el embudo del Dock)
                            let slice_w = target_w
                                + (win_w - target_w) * (1.0 - j) * (1.0 - v)
                                + (win_w - target_w) * (1.0 - k) * v;
                            let slice_w = slice_w.max(2.0);

                            // Desplazamiento horizontal progresivo hacia el slot del dock (acotado a 1.0 para evitar desfase)
                            let norm_y = if full_height > 1.0 { (local_y / full_height).clamp(0.0, 1.0) } else { v };
                            let norm_x = (norm_y * k + j).min(1.0);
                            let offset_x = (target_x - win_x) * norm_x;

                            // Bucle sinusoidal ondulante en "S" (estilo Compiz / GNOME Shell Magic Lamp predeterminado)
                            let s_phase = (1.0 - norm_y) * 2.0 * std::f32::consts::PI + std::f32::consts::PI;
                            let delta_x = (win_x + win_w * 0.5) - (target_x + target_w * 0.5);
                            let effect_x = s_phase.sin() * (delta_x / 7.0) * k * (1.0 - j * 0.7);

                            let left_x = win_x + offset_x + effect_x;
                            (screen_y, left_x, slice_w)
                        };

                        let alpha = if j > 0.85 {
                            ((1.0 - j) / 0.15).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        // Ultra-alta resolución de malla adaptativa (160 - 240 rebanadas)
                        let n_strips = ((win_h / 2.5) as usize).clamp(160, 240);
                        let sampled_slices: Vec<(f32, f32, f32)> = (0..=n_strips)
                            .map(|i| {
                                let v = i as f32 / n_strips as f32;
                                deform_slice(v)
                            })
                            .collect();

                        // Renderizado de la textura de la ventana con traslape continuo anti-fisuras
                        for elem in elems {
                            let src = elem.src();

                            for i in 0..n_strips {
                                let local_v0 = i as f32 / n_strips as f32;
                                let local_v1 = (i + 1) as f32 / n_strips as f32;

                                let y_top = sampled_slices[i].0.round() as i32;
                                let y_bottom = sampled_slices[i + 1].0.round() as i32;
                                let strip_y = y_top.min(y_bottom);
                                let strip_h = (y_bottom - y_top).abs().max(1) + 1; // Traslape de 1px anti-fisuras

                                let left_x = (sampled_slices[i].1 + sampled_slices[i + 1].1) * 0.5;
                                let width = (sampled_slices[i].2 + sampled_slices[i + 1].2) * 0.5;

                                let strip_dst = Rectangle::new(
                                    (left_x.round() as i32, strip_y).into(),
                                    (width.round().max(1.0) as i32, strip_h).into(),
                                );

                                let src_y0 = src.loc.y + (local_v0 as f64) * src.size.h;
                                let src_y1 = src.loc.y + (local_v1 as f64) * src.size.h;
                                let strip_src = Rectangle::new(
                                    (src.loc.x, src_y0).into(),
                                    (src.size.w, (src_y1 - src_y0).max(0.1)).into(),
                                );

                                match elem.texture() {
                                    WaylandSurfaceTexture::Texture(ref texture) => {
                                        if let Err(err) = frame.render_texture_from_to(
                                            texture,
                                            strip_src,
                                            strip_dst,
                                            &[Rectangle::from_size(strip_dst.size)],
                                            &[],
                                            elem.transform(),
                                            alpha,
                                            None,
                                            &[],
                                        ) {
                                            warn!("❌ [Genie Frame] render_texture_from_to falló: {:?}", err);
                                        }
                                    }
                                    WaylandSurfaceTexture::SolidColor(color) => {
                                        let solid = *color * alpha;
                                        let _ = frame.draw_solid(
                                            strip_dst,
                                            &[Rectangle::from_size(strip_dst.size)],
                                            solid,
                                        );
                                    }
                                }
                            }
                        }
                    }

                    let _ = frame.finish();

                    let mut finished = Vec::new();
                    state.active_animations.retain(|anim| {
                        if now.duration_since(anim.start_time) >= anim.duration {
                            finished.push(anim.clone());
                            false
                        } else {
                            true
                        }
                    });

                    for anim in finished {
                        if !anim.is_minimizing {
                            info!("✨ Restauración Genie completada: mapeando ventana a Space");
                            state.space.map_element(anim.window.clone(), anim.original_location, true);
                            state.space.raise_element(&anim.window, true);
                            if let Some(toplevel) = anim.window.toplevel() {
                                toplevel.send_configure();
                            }
                            if let Some(kb) = state.seat.get_keyboard() {
                                let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                                kb.set_focus(&mut state, Some(anim.surface.clone()), serial);
                            }
                            if let Some(pointer) = state.seat.get_pointer() {
                                let loc = pointer.current_location();
                                let target = state.space.element_under(loc).and_then(|(win, win_loc)| {
                                    let local_point = loc - win_loc.to_f64();
                                    win.surface_under(local_point, smithay::desktop::WindowSurfaceType::all())
                                        .map(|(s, sub_loc)| (s, (win_loc + sub_loc).to_f64()))
                                });
                                let motion_serial = smithay::utils::SERIAL_COUNTER.next_serial();
                                pointer.motion(
                                    &mut state,
                                    target,
                                    &smithay::input::pointer::MotionEvent {
                                        location: loc,
                                        serial: motion_serial,
                                        time: 0,
                                    },
                                );
                                pointer.frame(&mut state);
                            }
                            state.notify_active_app(Some(&anim.surface));
                            state.notify_running_apps();
                        }
                    }
                }
            }
        }

        let now_ms = start_time.elapsed().as_millis() as u32;
        for surface in state.xdg_shell_state.toplevel_surfaces() {
            send_frames_surface_tree(surface.wl_surface(), now_ms);
            for (popup, _) in smithay::desktop::PopupManager::popups_for_surface(surface.wl_surface()) {
                send_frames_surface_tree(popup.wl_surface(), now_ms);
            }
        }

        {
            let layer_map = smithay::desktop::layer_map_for_output(&output);
            for layer in layer_map.layers() {
                send_frames_surface_tree(layer.wl_surface(), now_ms);
                for (popup, _) in smithay::desktop::PopupManager::popups_for_surface(layer.wl_surface()) {
                    send_frames_surface_tree(popup.wl_surface(), now_ms);
                }
            }
        }

        // Aceptar nuevos clientes de forma no bloqueante
        while let Ok(Some(stream)) = listener.accept() {
            match display.handle().insert_client(
                stream,
                Arc::new(ClientState::default()),
            ) {
                Ok(client) => {
                    info!("🎉 Nuevo cliente Wayland conectado exitosamente al compositor!");
                    clients.push(client);
                }
                Err(err) => warn!("Error al registrar nuevo cliente Wayland: {:?}", err),
            }
        }

        state.process_ipc_messages();
        display.dispatch_clients(&mut state)?;
        state.space.refresh();
        for output in state.space.outputs() {
            let mut map = smithay::desktop::layer_map_for_output(output);
            map.cleanup();
        }
        state.popups.cleanup();
        state.active_popups.retain(|p| p.alive());
        display.flush_clients()?;

        let _ = backend.submit(Some(&[damage]));

        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Compositor Malus DE finalizado limpiamente.");
    Ok(())
}

pub fn toggle_spotlight(socket_name: &str) {
    let output = std::process::Command::new("pkill")
        .args(["-f", "malus-de-spotlight"])
        .status();

    if let Ok(status) = output {
        if status.success() {
            info!("Spotlight cerrado por alternancia (toggle).");
            return;
        }
    }

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("malus-de-spotlight")))
        .unwrap_or_else(|| std::path::PathBuf::from("malus-de-spotlight"));

    info!("Lanzando Spotlight desde: {:?}", exe);
    let mut command = std::process::Command::new(&exe);
    command.env("WAYLAND_DISPLAY", socket_name)
        .env("GDK_BACKEND", "wayland")
        .env("GSK_RENDERER", "cairo");
    if let Ok(cfg_home) = std::env::var("XDG_CONFIG_HOME") {
        command.env("XDG_CONFIG_HOME", cfg_home);
    } else if let Ok(home) = std::env::var("HOME") {
        command.env("XDG_CONFIG_HOME", format!("{}/.config/malus-de", home));
    }
    command.env("GSETTINGS_BACKEND", "keyfile");

    match command.spawn() {
        Ok(mut child) => {
            info!("Spotlight lanzado con éxito, PID={}", child.id());
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => tracing::error!("Error al lanzar Spotlight: {:?}", e),
    }
}
