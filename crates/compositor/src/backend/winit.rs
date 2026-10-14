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
    utils::{Rectangle, Transform, SERIAL_COUNTER},
    wayland::{
        compositor::{with_surface_tree_downward, SurfaceAttributes, TraversalAction},
        seat::WaylandFocus,
        shell::wlr_layer::Layer,
    },
};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge;
use tracing::{info, warn};

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
            model: "Virtual Display".to_string(),
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
                            if is_pressed {
                                let sym = keysym_handle.modified_sym();
                                let has_super = modifiers.logo;
                                let has_alt = modifiers.alt;
                                let has_ctrl = modifiers.ctrl;

                                let is_space = sym == keysyms::KEY_space.into();
                                let is_k = sym == keysyms::KEY_k.into() || sym == keysyms::KEY_K.into();

                                // Atajos para alternar Spotlight:
                                // 1. Super + Espacio (macOS / Malus DE nativo)
                                // 2. Alt + Espacio (estándar PC, evita conflicto con switch-input-source de GNOME)
                                // 3. Ctrl + Espacio (alternativa clásica de launchers)
                                // 4. Super + K (Command Palette moderno)
                                if (is_space && (has_super || has_alt || has_ctrl)) || (is_k && has_super) {
                                    info!("🔍 Atajo de Spotlight detectado (Super={}, Alt={}, Ctrl={}, sym={:?}): alternando...",
                                        has_super, has_alt, has_ctrl, sym);
                                    toggle_spotlight(&state.socket_name);
                                    return FilterResult::Intercept(());
                                }

                                // Atajo Alt + Tab / Super + Tab: conmutador rápido de ventanas (adelante y atrás con Shift)
                                let is_tab = sym == keysyms::KEY_Tab.into() || sym == keysyms::KEY_ISO_Left_Tab.into() || key == 23;
                                if (has_alt || has_super) && is_tab {
                                    let has_shift = modifiers.shift;
                                    let windows: Vec<_> = state.space.elements().cloned().collect();
                                    if windows.len() > 1 {
                                        let target_window = if has_shift {
                                            windows.first().cloned()
                                        } else {
                                            windows.iter().rev().nth(1).cloned()
                                        };

                                        if let Some(win) = target_window {
                                            state.space.raise_element(&win, true);
                                            let serial = SERIAL_COUNTER.next_serial();
                                            if let Some(surf) = win.wl_surface() {
                                                if let Some(kb) = state.seat.get_keyboard() {
                                                    kb.set_focus(state, Some(surf.into_owned()), serial);
                                                }
                                            }
                                            info!("🔀 Alt+Tab: conmutado foco a siguiente ventana (geo={:?})", win.geometry());
                                            return FilterResult::Intercept(());
                                        }
                                    }
                                }

                                // Atajos de Aero Snap / Tiling estilo Windows 11 (Super o Alt + Flechas)
                                if has_super || has_alt {
                                    let is_left = sym == keysyms::KEY_Left.into() || key == 105 || key == 113;
                                    let is_right = sym == keysyms::KEY_Right.into() || key == 106 || key == 114;
                                    let is_up = sym == keysyms::KEY_Up.into() || key == 103 || key == 111;
                                    let is_down = sym == keysyms::KEY_Down.into() || key == 108 || key == 116;

                                    if is_left || is_right || is_up || is_down {
                                        let active_window = state.space.elements().rev().next().cloned();
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
                                layer
                                    .surface_under(local_point, smithay::desktop::WindowSurfaceType::all())
                                    .map(|(surf, sub_loc)| (surf, (geo.loc + sub_loc).to_f64()))
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
                                layer
                                    .surface_under(local_point, smithay::desktop::WindowSurfaceType::all())
                                    .map(|(surf, sub_loc)| {
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
                        if let Some((surf, ns, can_kb)) = layer_info {
                            info!("🎯 Clic de ratón sobre layer surface: namespace={}, pos={:?}", ns, point);
                            if can_kb {
                                keyboard.set_focus(&mut state, Some(surf), btn_serial);
                            }
                        } else if let Some((window, loc)) = regular_win {
                            info!("🖱️ Clic en ventana regular: button={:#x}, pos={:?}, win_loc={:?}, geo={:?}", event.button_code(), point, loc, window.geometry());
                            state.space.raise_element(&window, true);
                            if let Some(surface) = window.wl_surface() {
                                keyboard.set_focus(&mut state, Some(surface.into_owned()), btn_serial);
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

                    let _ = frame.finish();
                }
            }
        }

        for surface in state.xdg_shell_state.toplevel_surfaces() {
            send_frames_surface_tree(surface.wl_surface(), start_time.elapsed().as_millis() as u32);
        }

        {
            let layer_map = smithay::desktop::layer_map_for_output(&output);
            for layer in layer_map.layers() {
                send_frames_surface_tree(layer.wl_surface(), start_time.elapsed().as_millis() as u32);
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

        display.dispatch_clients(&mut state)?;
        state.space.refresh();
        for output in state.space.outputs() {
            let mut map = smithay::desktop::layer_map_for_output(output);
            map.cleanup();
        }
        display.flush_clients()?;

        let _ = backend.submit(Some(&[damage]));

        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Compositor Malus DE finalizado limpiamente.");
    Ok(())
}

fn toggle_spotlight(socket_name: &str) {
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
    match std::process::Command::new(&exe)
        .env("WAYLAND_DISPLAY", socket_name)
        .env("GDK_BACKEND", "wayland")
        .env("GSK_RENDERER", "cairo")
        .spawn()
    {
        Ok(mut child) => {
            info!("Spotlight lanzado con éxito, PID={}", child.id());
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => tracing::error!("Error al lanzar Spotlight: {:?}", e),
    }
}
