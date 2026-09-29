use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use anyhow::{Context, Result};
use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Event, InputEvent, KeyboardKeyEvent, PointerButtonEvent,
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
        keyboard::FilterResult,
        pointer::{ButtonEvent, MotionEvent},
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
                    keyboard.input::<(), _>(
                        &mut state,
                        event.key_code(),
                        event.state(),
                        0.into(),
                        0,
                        |_, _, _| FilterResult::Forward,
                    );
                }
                InputEvent::PointerMotionAbsolute { event } => {
                    let output_geo = state.space.output_geometry(&output).unwrap_or_default();
                    let point = event.position_transformed(output_geo.size);
                    let serial = SERIAL_COUNTER.next_serial();

                    let layer_map = smithay::desktop::layer_map_for_output(&output);
                    let under_layer = layer_map.layer_under(Layer::Top, point)
                        .or_else(|| layer_map.layer_under(Layer::Overlay, point));

                    if let Some(layer) = under_layer {
                        if let Some(geo) = layer_map.layer_geometry(layer) {
                            let local_point = point - geo.loc.to_f64();
                            pointer.motion(
                                &mut state,
                                Some((layer.wl_surface().clone(), local_point)),
                                &MotionEvent {
                                    location: point,
                                    serial,
                                    time: event.time_msec(),
                                },
                            );
                        }
                    } else if let Some((window, loc)) = state.space.element_under(point) {
                        let local_point = point - loc.to_f64();
                        if let Some(surface) = window.wl_surface() {
                            let surface = surface.into_owned();
                            keyboard.set_focus(&mut state, Some(surface.clone()), serial);
                            pointer.motion(
                                &mut state,
                                Some((surface, local_point)),
                                &MotionEvent {
                                    location: point,
                                    serial,
                                    time: event.time_msec(),
                                },
                            );
                        }
                    } else {
                        pointer.motion(
                            &mut state,
                            None,
                            &MotionEvent {
                                location: point,
                                serial,
                                time: event.time_msec(),
                            },
                        );
                    }
                }
                InputEvent::PointerButton { event } => {
                    let serial = SERIAL_COUNTER.next_serial();
                    pointer.button(
                        &mut state,
                        &ButtonEvent {
                            serial,
                            time: event.time_msec(),
                            button: event.button_code(),
                            state: event.state(),
                        },
                    );
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
        display.flush_clients()?;

        let _ = backend.submit(Some(&[damage]));

        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Compositor Malus DE finalizado limpiamente.");
    Ok(())
}
