use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use anyhow::{Context, Result};
use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc,
        },
        drm::{DrmDevice, DrmDeviceFd, DrmEvent, GbmBufferedSurface},
        egl::{EGLContext, EGLDisplay},
        input::{
            AbsolutePositionEvent, Axis, ButtonState, Event, InputEvent, KeyState,
            KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
        },
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            gles::{GlesRenderer, GlesTexture},
            utils::draw_render_elements,
            Bind, Color32F, Frame, ImportDma, ImportMem, Renderer, Texture,
        },
        session::{
            libseat::LibSeatSession,
            Event as SessionEvent, Session,
        },
        udev::{all_gpus, primary_gpu},
    },
    desktop::space::space_render_elements,
    input::{
        keyboard::{keysyms, FilterResult},
        pointer::{AxisFrame, ButtonEvent, CursorImageStatus, MotionEvent},
    },
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::{
        calloop::EventLoop,
        drm::control::{connector, crtc, Device as ControlDevice},
        input::Libinput,
        wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
        wayland_server::{Display, ListeningSocket},
    },
    utils::{DeviceFd, Logical, Physical, Point, Rectangle, Transform, SERIAL_COUNTER, Size},
    wayland::{
        seat::WaylandFocus,
        shell::wlr_layer::Layer,
    },
};
use tracing::{error, info, warn};

use malus_de_common::config::DisplaysConfig;
use malus_de_common::ipc::IpcMessage;
use crate::backend::winit::{send_frames_surface_tree, toggle_spotlight};
use crate::state::{ClientState, MalusState};

struct DrmOutputSurface {
    output: Output,
    surface: GbmBufferedSurface<GbmAllocator<DrmDeviceFd>, ()>,
    crtc: crtc::Handle,
    size: Size<i32, Physical>,
    damage: Rectangle<i32, Physical>,
}

fn create_macos_cursor(renderer: &mut GlesRenderer) -> Option<GlesTexture> {
    let w = 16;
    let h = 16;
    let mut data = vec![0u8; w * h * 4];
    let pattern = [
        "X               ",
        "XX              ",
        "X.X             ",
        "X..X            ",
        "X...X           ",
        "X....X          ",
        "X.....X         ",
        "X......X        ",
        "X.......X       ",
        "X.....XXXX      ",
        "X..X..X         ",
        "X.X X..X        ",
        "XX   X..X       ",
        "X     X..X      ",
        "       XX       ",
        "                ",
    ];

    for (y, row) in pattern.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let idx = (y * w + x) * 4;
            match ch {
                'X' => {
                    data[idx] = 0;
                    data[idx + 1] = 0;
                    data[idx + 2] = 0;
                    data[idx + 3] = 255;
                }
                '.' => {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                    data[idx + 3] = 255;
                }
                _ => {}
            }
        }
    }

    let size = Size::from((w as i32, h as i32));
    renderer.import_memory(&data, Fourcc::Argb8888, size, false).ok()
}

fn find_layer_under(
    state: &MalusState,
    point: Point<f64, Logical>,
) -> Option<(
    smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    Point<f64, Logical>,
    String,
    bool,
)> {
    for output in state.space.outputs() {
        let layer_map = smithay::desktop::layer_map_for_output(output);
        let under_layer = layer_map
            .layer_under(Layer::Overlay, point)
            .or_else(|| layer_map.layer_under(Layer::Top, point));

        if let Some(layer) = under_layer {
            if let Some(geo) = layer_map.layer_geometry(layer) {
                let local_point = point - geo.loc.to_f64();
                let surface_hit = if !state.active_popups.is_empty() {
                    layer.surface_under(local_point, smithay::desktop::WindowSurfaceType::POPUP)
                } else {
                    None
                };
                let surface_hit = surface_hit.or_else(|| {
                    if layer.namespace() == "malus-top-bar" && local_point.y >= 32.0 {
                        None
                    } else {
                        layer.surface_under(
                            local_point,
                            smithay::desktop::WindowSurfaceType::TOPLEVEL,
                        )
                    }
                });

                if let Some((surf, sub_loc)) = surface_hit {
                    return Some((
                        surf,
                        (geo.loc + sub_loc).to_f64(),
                        layer.namespace().to_string(),
                        layer.can_receive_keyboard_focus(),
                    ));
                }
            }
        }
    }
    None
}

fn clamp_pointer_location(state: &MalusState, mut point: Point<f64, Logical>) -> Point<f64, Logical> {
    let mut min_x = 0.0f64;
    let mut min_y = 0.0f64;
    let mut max_x = 1920.0f64;
    let mut max_y = 1080.0f64;
    let mut has_output = false;

    for output in state.space.outputs() {
        let geo = state.space.output_geometry(output).unwrap_or_default();
        let x0 = geo.loc.x as f64;
        let y0 = geo.loc.y as f64;
        let x1 = (geo.loc.x + geo.size.w) as f64;
        let y1 = (geo.loc.y + geo.size.h) as f64;
        if !has_output {
            min_x = x0;
            min_y = y0;
            max_x = x1;
            max_y = y1;
            has_output = true;
        } else {
            min_x = min_x.min(x0);
            min_y = min_y.min(y0);
            max_x = max_x.max(x1);
            max_y = max_y.max(y1);
        }
    }

    if has_output {
        point.x = point.x.clamp(min_x, max_x - 1.0);
        point.y = point.y.clamp(min_y, max_y - 1.0);
    }
    point
}

fn resolve_target_under(
    state: &MalusState,
    point: Point<f64, Logical>,
) -> (
    Option<(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, Point<f64, Logical>)>,
    Option<(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, String, bool)>,
    Option<(smithay::desktop::Window, Point<i32, Logical>)>,
) {
    let layer_hit = find_layer_under(state, point);
    if let Some((surf, focus_loc, ns, can_kb)) = layer_hit {
        (
            Some((surf.clone(), focus_loc)),
            Some((surf, ns, can_kb)),
            None,
        )
    } else if let Some((window, loc)) = state.space.element_under(point) {
        let local_point = point - loc.to_f64();
        let target = window
            .surface_under(local_point, smithay::desktop::WindowSurfaceType::all())
            .map(|(surf, sub_loc)| (surf, (loc + sub_loc).to_f64()));
        (target, None, Some((window.clone(), loc)))
    } else {
        (None, None, None)
    }
}

pub fn run_drm() -> Result<()> {
    info!("============================================================");
    info!("🚀 MALUS DE: Iniciando en modo DRM/KMS nativo (Direct TTY/GDM)");
    info!("============================================================");

    let mut event_loop: EventLoop<MalusState> = EventLoop::try_new()
        .context("Error al inicializar EventLoop de Calloop")?;
    let mut display: Display<MalusState> = Display::new()
        .context("Error al crear Display Wayland")?;
    let mut state = MalusState::new(&display);

    // 1. Inicializar sesión de hardware (LibSeat / logind)
    let (mut session, notifier) = match LibSeatSession::new() {
        Ok(res) => res,
        Err(err) => {
            error!("❌ Error fatal: no se pudo inicializar LibSeatSession: {:?}", err);
            eprintln!("⚠️ LibSeat no pudo tomar control del asiento TTY ({err:?}).");
            eprintln!("   Si estás dentro de una sesión gráfica existente, ejecuta Malus DE en modo anidado:");
            eprintln!("   malus-de-compositor --backend winit");
            anyhow::bail!("LibSeatSession initialization failed: {:?}", err);
        }
    };
    let seat_name = session.seat();
    info!("✅ Sesión de hardware activa en asiento: '{}'", seat_name);

    // 2. Detectar y abrir nodo DRM de la GPU primaria vía LibSeat
    let gpu_path = primary_gpu(&seat_name)
        .context("Error al buscar GPU primaria")?
        .or_else(|| all_gpus(&seat_name).ok()?.into_iter().next())
        .unwrap_or_else(|| std::path::PathBuf::from("/dev/dri/card1"));
    info!("🎮 GPU seleccionada: {:?}", gpu_path);

    let raw_fd = session.open(
        &gpu_path,
        smithay::reexports::rustix::fs::OFlags::RDWR
            | smithay::reexports::rustix::fs::OFlags::CLOEXEC
            | smithay::reexports::rustix::fs::OFlags::NOCTTY,
    ).context("Error al abrir nodo DRM mediante libseat")?;
    let drm_device_fd = DrmDeviceFd::new(DeviceFd::from(raw_fd));

    // 3. Inicializar DrmDevice, GBM, EGL y GlesRenderer
    let (mut drm_device, drm_notifier) = DrmDevice::new(drm_device_fd.clone(), false)
        .context("Error al inicializar DrmDevice")?;
    let gbm_device = GbmDevice::new(drm_device_fd.clone())
        .context("Error al inicializar GbmDevice")?;
    let egl_display = unsafe { EGLDisplay::new(gbm_device.clone()) }
        .context("Error al crear EGLDisplay sobre GBM")?;
    let egl_context = EGLContext::new(&egl_display)
        .context("Error al crear EGLContext")?;
    let mut renderer = unsafe { GlesRenderer::new(egl_context) }
        .context("Error al inicializar GlesRenderer")?;

    info!("🎨 Renderer OpenGL ES acelerado por hardware listo");

    // Crear textura del cursor macOS
    let cursor_texture = create_macos_cursor(&mut renderer);

    // 4. Configurar dispositivos de entrada (teclado y puntero)
    let _keyboard = state
        .seat
        .add_keyboard(Default::default(), 200, 200)
        .map_err(|e| anyhow::anyhow!("Error al inicializar teclado: {e:?}"))?;
    let _pointer = state.seat.add_pointer();

    // 5. Crear socket de escucha de clientes Wayland (wayland-1 .. 32)
    let listener = ListeningSocket::bind_auto("wayland", 1..33)
        .map_err(|e| anyhow::anyhow!("Error al bindear socket de Wayland: {e:?}"))?;
    let socket_name = listener
        .socket_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    info!("🌐 Socket Wayland escuchando activamente en: WAYLAND_DISPLAY={}", socket_name);
    state.socket_name = socket_name.clone();

    // 6. Descubrir conectores físicos y configurar superficies de pantalla
    let res_handles = drm_device.resource_handles()
        .context("Error al consultar resource handles de DRM")?;
    let displays_cfg = DisplaysConfig::load();
    let mut used_crtcs = HashSet::new();
    let mut surfaces = Vec::new();

    for &conn_handle in res_handles.connectors() {
        let conn_info = match drm_device.get_connector(conn_handle, false) {
            Ok(info) => info,
            Err(e) => {
                warn!("Error al leer connector {:?}: {:?}", conn_handle, e);
                continue;
            }
        };

        if conn_info.state() != connector::State::Connected {
            continue;
        }

        let conn_name = format!("{}-{}", conn_info.interface().as_str(), conn_info.interface_id());
        info!("🔌 Monitor físico conectado detectado: {}", conn_name);

        let cfg_output = displays_cfg.outputs.iter().find(|o| o.connector.eq_ignore_ascii_case(&conn_name));
        let mode = if let Some(cfg) = cfg_output {
            conn_info.modes().iter().find(|m| m.size().0 == cfg.width as u16 && m.size().1 == cfg.height as u16).copied()
                .or_else(|| conn_info.modes().first().copied())
        } else {
            conn_info.modes().first().copied()
        };

        let mode = match mode {
            Some(m) => m,
            None => {
                warn!("No hay modos de vídeo para {}", conn_name);
                continue;
            }
        };

        let possible_crtcs: Vec<crtc::Handle> = conn_info
            .encoders()
            .iter()
            .filter_map(|&enc_handle| drm_device.get_encoder(enc_handle).ok())
            .flat_map(|enc| res_handles.filter_crtcs(enc.possible_crtcs()))
            .collect();

        let crtc = match possible_crtcs.into_iter().find(|c| !used_crtcs.contains(c)) {
            Some(c) => {
                used_crtcs.insert(c);
                c
            }
            None => {
                warn!("No se encontró CRTC libre para el monitor {}", conn_name);
                continue;
            }
        };

        info!("   🖥 Asignando CRTC {:?} a {} con modo {}x{} @ {}Hz",
            crtc, conn_name, mode.size().0, mode.size().1, mode.vrefresh());

        let drm_surface = match drm_device.create_surface(crtc, mode, &[conn_handle]) {
            Ok(s) => s,
            Err(e) => {
                error!("Error al crear DrmSurface para {}: {:?}", conn_name, e);
                continue;
            }
        };

        let gbm_allocator = GbmAllocator::new(
            gbm_device.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        );

        let gbm_surface = match GbmBufferedSurface::new(
            drm_surface,
            gbm_allocator,
            &[Fourcc::Argb8888, Fourcc::Xrgb8888],
            renderer.dmabuf_formats(),
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Error al inicializar GbmBufferedSurface para {}: {:?}", conn_name, e);
                continue;
            }
        };

        let (mode_w, mode_h) = mode.size();
        let (loc_x, loc_y, scale) = if let Some(cfg) = cfg_output {
            (cfg.x, cfg.y, cfg.scale)
        } else {
            (0, 0, 1.0)
        };

        let output = Output::new(
            conn_name.clone(),
            PhysicalProperties {
                size: (mode_w as i32, mode_h as i32).into(),
                subpixel: Subpixel::Unknown,
                make: "Apple/Malus".to_string(),
                model: conn_name.clone(),
            },
        );

        let smithay_mode = Mode {
            size: (mode_w as i32, mode_h as i32).into(),
            refresh: (mode.vrefresh() * 1000) as i32,
        };
        let _global = output.create_global::<MalusState>(&state.display_handle);
        output.set_preferred(smithay_mode);
        output.change_current_state(
            Some(smithay_mode),
            Some(Transform::Normal),
            Some(Scale::Fractional(scale)),
            Some((loc_x, loc_y).into()),
        );
        state.space.map_output(&output, (loc_x, loc_y));

        surfaces.push(DrmOutputSurface {
            output,
            surface: gbm_surface,
            crtc,
            size: (mode_w as i32, mode_h as i32).into(),
            damage: Rectangle::from_size((mode_w as i32, mode_h as i32).into()),
        });
    }

    if surfaces.is_empty() {
        warn!("⚠️ No se detectó ninguna pantalla activa conectada en DRM.");
    }

    // Centrar el puntero inmediatamente en el centro de la pantalla primaria para evitar coordenadas vacías
    let primary_center = state.space.outputs().next().and_then(|primary| {
        let geo = state.space.output_geometry(primary)?;
        Some(geo.loc.to_f64() + (geo.size.to_f64() / 2.0))
    });

    if let Some(center) = primary_center {
        if let Some(pointer) = state.seat.get_pointer() {
            let serial = SERIAL_COUNTER.next_serial();
            pointer.motion(
                &mut state,
                None,
                &MotionEvent {
                    location: center,
                    serial,
                    time: 0,
                },
            );
            pointer.frame(&mut state);
            info!("🎯 Puntero DRM inicializado en el centro de la pantalla: {:?}", center);
        }
    }

    let shared_surfaces = Arc::new(Mutex::new(surfaces));
    let surfaces_for_drm = shared_surfaces.clone();
    let surfaces_for_vt = shared_surfaces.clone();

    // 7. Inicializar Libinput para teclado y ratón
    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        session.clone().into(),
    );
    if let Err(e) = libinput_context.udev_assign_seat(&seat_name) {
        warn!("Aviso al asignar asiento en libinput: {:?}", e);
    }
    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    // 8. Registrar fuentes de eventos en el EventLoop de Calloop
    let loop_handle = event_loop.handle();

    // Notificador de pausa / reactivación de sesión TTY (VT Switch)
    loop_handle.insert_source(notifier, move |event, &mut (), _state: &mut MalusState| {
        match event {
            SessionEvent::PauseSession => {
                info!("⏸️ Sesión TTY pausada (VT switch away)");
                libinput_context.suspend();
            }
            SessionEvent::ActivateSession => {
                info!("▶️ Sesión TTY reactivada (VT switch back)");
                if let Err(e) = libinput_context.resume() {
                    error!("Error al reanudar libinput tras VT switch: {:?}", e);
                }
                if let Ok(mut surfs) = surfaces_for_vt.lock() {
                    for s in surfs.iter_mut() {
                        s.damage = Rectangle::from_size(s.size);
                    }
                }
            }
        }
    }).map_err(|e| anyhow::anyhow!("Error al registrar notificador de sesión: {e:?}"))?;

    // Notificador de eventos de sincronización DRM (VBlank y Page Flip)
    loop_handle.insert_source(drm_notifier, move |event, _metadata, _state: &mut MalusState| {
        match event {
            DrmEvent::VBlank(crtc) => {
                if let Ok(mut surfs) = surfaces_for_drm.lock() {
                    for s in surfs.iter_mut() {
                        if s.crtc == crtc {
                            let _ = s.surface.frame_submitted();
                        }
                    }
                }
            }
            DrmEvent::Error(err) => {
                warn!("Aviso en evento DRM: {:?}", err);
            }
        }
    }).map_err(|e| anyhow::anyhow!("Error al registrar notificador DRM: {e:?}"))?;

    // Despacho de eventos de Libinput con soporte para ratón, touchpad y atajos completos
    let mut session_vt = session.clone();

    loop_handle.insert_source(libinput_backend, move |event, _, state: &mut MalusState| {
        match event {
            InputEvent::Keyboard { event } => {
                let _key = event.key_code().raw();
                let is_pressed = event.state() == KeyState::Pressed;

                if let Some(keyboard) = state.seat.get_keyboard() {
                    keyboard.input::<(), _>(
                        state,
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

                                // 🚨 RED DE SEGURIDAD CRÍTICA: CAMBIO DE TTY CON CTRL + ALT + F1..F12
                                if has_ctrl && has_alt {
                                    let vt_target = if sym == keysyms::KEY_F1.into() { Some(1) }
                                    else if sym == keysyms::KEY_F2.into() { Some(2) }
                                    else if sym == keysyms::KEY_F3.into() { Some(3) }
                                    else if sym == keysyms::KEY_F4.into() { Some(4) }
                                    else if sym == keysyms::KEY_F5.into() { Some(5) }
                                    else if sym == keysyms::KEY_F6.into() { Some(6) }
                                    else if sym == keysyms::KEY_F7.into() { Some(7) }
                                    else if sym == keysyms::KEY_F8.into() { Some(8) }
                                    else if sym == keysyms::KEY_F9.into() { Some(9) }
                                    else if sym == keysyms::KEY_F10.into() { Some(10) }
                                    else if sym == keysyms::KEY_F11.into() { Some(11) }
                                    else if sym == keysyms::KEY_F12.into() { Some(12) }
                                    else { None };

                                    if let Some(vt) = vt_target {
                                        info!("🚨 [VT SWITCH] Solicitando cambio de terminal virtual a tty{} vía LibSeat", vt);
                                        if let Err(err) = session_vt.change_vt(vt) {
                                            error!("Error al cambiar a VT {}: {:?}", vt, err);
                                        }
                                        return FilterResult::Intercept(());
                                    }
                                }

                                let is_tab = sym == keysyms::KEY_Tab.into();
                                let is_left = sym == keysyms::KEY_Left.into();
                                let is_right = sym == keysyms::KEY_Right.into();
                                let is_q = sym == keysyms::KEY_q.into() || sym == keysyms::KEY_Q.into();
                                let is_w = sym == keysyms::KEY_w.into() || sym == keysyms::KEY_W.into();
                                let is_h = sym == keysyms::KEY_h.into() || sym == keysyms::KEY_H.into();
                                let is_m = sym == keysyms::KEY_m.into() || sym == keysyms::KEY_M.into();
                                let is_esc = sym == keysyms::KEY_Escape.into();

                                // 2. Navegación activa dentro del App Switcher (⌘Tab HUD)
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

                                let is_space = sym == keysyms::KEY_space.into();
                                let is_k = sym == keysyms::KEY_k.into() || sym == keysyms::KEY_K.into();

                                // Spotlight (Super+Espacio / Alt+Espacio / Super+K)
                                if (is_space && (has_super || has_alt || has_ctrl)) || (is_k && has_super) {
                                    info!("🔍 Spotlight alternado vía atajo de teclado");
                                    toggle_spotlight(&state.socket_name);
                                    return FilterResult::Intercept(());
                                }

                                // Cerrar ventana activa (Super+W / Super+Q / Alt+F4)
                                let is_f4 = sym == keysyms::KEY_F4.into();
                                if (has_super && (is_w || is_q)) || (has_alt && is_f4) {
                                    state.close_active_window();
                                    return FilterResult::Intercept(());
                                }

                                // Minimizar ventana activa con Genie Effect (Super+M / Super+H / Alt+F9)
                                let is_f9 = sym == keysyms::KEY_F9.into();
                                if (has_super && (is_m || is_h)) || (has_alt && is_f9) {
                                    state.minimize_active_window();
                                    return FilterResult::Intercept(());
                                }
                            } else {
                                // Liberación de modificadores (Super/Alt) -> Commit de App Switcher
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
            }
            InputEvent::PointerMotion { event } => {
                if let Some(pointer) = state.seat.get_pointer() {
                    let mut current_pos = pointer.current_location();
                    current_pos.x += event.delta_x();
                    current_pos.y += event.delta_y();
                    let point = clamp_pointer_location(state, current_pos);

                    let (under_target, _, _) = resolve_target_under(state, point);

                    let serial = SERIAL_COUNTER.next_serial();
                    pointer.motion(
                        state,
                        under_target,
                        &MotionEvent {
                            location: point,
                            serial,
                            time: event.time_msec(),
                        },
                    );
                    pointer.frame(state);
                }
            }
            InputEvent::PointerMotionAbsolute { event } => {
                if let Some(pointer) = state.seat.get_pointer() {
                    let cur = pointer.current_location();
                    let output = state
                        .space
                        .outputs()
                        .find(|o| {
                            state
                                .space
                                .output_geometry(o)
                                .map(|g| g.to_f64().contains(cur))
                                .unwrap_or(false)
                        })
                        .or_else(|| state.space.outputs().next())
                        .cloned();

                    if let Some(output) = output {
                        let output_geo = state.space.output_geometry(&output).unwrap_or_default();
                        let mut point = event.position_transformed(output_geo.size);
                        point.x += output_geo.loc.x as f64;
                        point.y += output_geo.loc.y as f64;
                        let point = clamp_pointer_location(state, point);

                        let (under_target, _, _) = resolve_target_under(state, point);

                        let serial = SERIAL_COUNTER.next_serial();
                        pointer.motion(
                            state,
                            under_target,
                            &MotionEvent {
                                location: point,
                                serial,
                                time: event.time_msec(),
                            },
                        );
                        pointer.frame(state);
                    }
                }
            }
            InputEvent::PointerButton { event } => {
                if let Some(pointer) = state.seat.get_pointer() {
                    let point = pointer.current_location();
                    let is_pressed = event.state() == ButtonState::Pressed;

                    let (under_target, layer_info, regular_win) = resolve_target_under(state, point);

                    let motion_serial = SERIAL_COUNTER.next_serial();
                    pointer.motion(
                        state,
                        under_target,
                        &MotionEvent {
                            location: point,
                            serial: motion_serial,
                            time: event.time_msec(),
                        },
                    );

                    let btn_serial = SERIAL_COUNTER.next_serial();
                    if is_pressed {
                        let clicked_on_popup = layer_info.as_ref().map(|(s, _, _)| {
                            state.active_popups.iter().any(|p| p.wl_surface() == s)
                        }).unwrap_or(false);

                        if !clicked_on_popup && !state.active_popups.is_empty() {
                            state.dismiss_all_popups();
                        }

                        if pointer.is_grabbed() {
                            pointer.unset_grab(state, btn_serial, event.time_msec());
                        }

                        if let Some(keyboard) = state.seat.get_keyboard() {
                            if keyboard.is_grabbed() {
                                keyboard.unset_grab(state);
                            }

                            if let Some((surf, _ns, can_kb)) = layer_info {
                                if can_kb {
                                    keyboard.set_focus(state, Some(surf), btn_serial);
                                }
                            } else if let Some((window, loc)) = regular_win {
                                let was_already_top = state.space.elements().last().map(|w| w == &window).unwrap_or(false);
                                if !was_already_top {
                                    state.space.raise_element(&window, true);
                                }
                                if let Some(surface) = window.wl_surface() {
                                    if keyboard.current_focus().as_ref() != Some(surface.as_ref()) {
                                        keyboard.set_focus(state, Some(surface.clone().into_owned()), btn_serial);
                                        state.notify_active_app(Some(&surface));
                                    }
                                }

                                let has_super = keyboard.modifier_state().logo;
                                let has_alt = keyboard.modifier_state().alt;
                                if (has_super || has_alt) && event.button_code() == 0x110 {
                                    let start_data = smithay::input::pointer::GrabStartData {
                                        focus: None,
                                        button: event.button_code(),
                                        location: point,
                                    };
                                    let grab = crate::grabs::MoveSurfaceGrab {
                                        start_data,
                                        window: window.clone(),
                                        initial_window_location: loc,
                                    };
                                    pointer.set_grab(state, grab, btn_serial, smithay::input::pointer::Focus::Clear);
                                }

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
                                    pointer.set_grab(state, grab, btn_serial, smithay::input::pointer::Focus::Clear);
                                }
                            }
                        }
                    }

                    pointer.button(
                        state,
                        &ButtonEvent {
                            button: event.button_code(),
                            state: event.state(),
                            serial: btn_serial,
                            time: event.time_msec(),
                        },
                    );
                    pointer.frame(state);
                }
            }
            InputEvent::PointerAxis { event } => {
                if let Some(pointer) = state.seat.get_pointer() {
                    let mut frame = AxisFrame::new(event.time_msec()).source(event.source());
                    let horiz = event.amount(Axis::Horizontal).unwrap_or(0.0);
                    let vert = event.amount(Axis::Vertical).unwrap_or(0.0);
                    if horiz != 0.0 {
                        frame = frame.value(Axis::Horizontal, horiz);
                    }
                    if vert != 0.0 {
                        frame = frame.value(Axis::Vertical, vert);
                    }
                    pointer.axis(state, frame);
                    pointer.frame(state);
                }
            }
            _ => {}
        }
    }).map_err(|e| anyhow::anyhow!("Error al registrar libinput event source: {e:?}"))?;

    info!("============================================================");
    info!("✨ MALUS DE: Bucle de renderizado DRM/KMS y GPU activo");
    info!("   WAYLAND_DISPLAY={}", socket_name);
    info!("============================================================");

    let dark_acrylic = Color32F::new(0.094, 0.094, 0.106, 1.0);
    let start_time = Instant::now();
    let mut clients = Vec::new();

    while state.is_running {
        // Si la sesión TTY no está activa (VT switch away), pausar renderizado para no saturar GPU
        if !session.is_active() {
            std::thread::sleep(Duration::from_millis(30));
            let _ = event_loop.dispatch(Some(Duration::from_millis(1)), &mut state);
            continue;
        }

        // Aceptar nuevos clientes de forma no bloqueante
        while let Ok(Some(stream)) = listener.accept() {
            match display.handle().insert_client(
                stream,
                Arc::new(ClientState::default()),
            ) {
                Ok(client) => {
                    info!("🎉 Nuevo cliente Wayland conectado al compositor DRM!");
                    clients.push(client);
                }
                Err(err) => warn!("Error al registrar cliente Wayland en DRM: {:?}", err),
            }
        }

        // Despachar eventos de hardware (libinput, sesión, vblank)
        let _ = event_loop.dispatch(Some(Duration::from_millis(1)), &mut state);

        state.process_ipc_messages();
        display.dispatch_clients(&mut state)?;
        state.space.refresh();

        for output in state.space.outputs() {
            let mut map = smithay::desktop::layer_map_for_output(output);
            map.cleanup();
        }
        state.popups.cleanup();
        state.active_popups.retain(|p| p.alive());

        // 🎨 RENDERIZADO GPU REAL A CADA MONITOR FÍSICO
        if let Ok(mut surfs) = shared_surfaces.lock() {
            for out_surface in surfs.iter_mut() {
                let scale = out_surface.output.current_scale().fractional_scale();

                let (mut dmabuf, _age) = match out_surface.surface.next_buffer() {
                    Ok(buf) => buf,
                    Err(e) => {
                        // Si no hay slots libres, la GPU aún está en escaneo de buffer anterior (esperar vblank)
                        warn!("Buffer no disponible para {}: {:?}", out_surface.output.name(), e);
                        continue;
                    }
                };

                let damage = out_surface.damage;
                let elements = match space_render_elements::<
                    GlesRenderer,
                    smithay::desktop::Window,
                    _,
                >(&mut renderer, [&state.space], &out_surface.output, 1.0) {
                    Ok(elems) => elems,
                    Err(err) => {
                        warn!("Error al obtener render elements de space: {:?}", err);
                        Vec::new()
                    }
                };

                // Preparar elementos de cursor antes de comenzar el pase de renderizado
                let mut cursor_elems = Vec::new();
                let mut cursor_draw_point = None;
                if let Some(pointer) = state.seat.get_pointer() {
                    let loc = pointer.current_location();
                    let geo = state.space.output_geometry(&out_surface.output).unwrap_or_default();
                    if geo.to_f64().contains(loc) {
                        let phys_x = ((loc.x - geo.loc.x as f64) * scale).round() as i32;
                        let phys_y = ((loc.y - geo.loc.y as f64) * scale).round() as i32;
                        cursor_draw_point = Some((phys_x, phys_y));

                        if let CursorImageStatus::Surface(ref surf) = state.cursor_status {
                            cursor_elems = render_elements_from_surface_tree::<GlesRenderer, WaylandSurfaceRenderElement<GlesRenderer>>(
                                &mut renderer,
                                surf,
                                (phys_x, phys_y),
                                scale,
                                1.0,
                                Kind::Unspecified,
                            );
                        }
                    }
                }

                if let Ok(mut target) = renderer.bind(&mut dmabuf) {
                    if let Ok(mut frame) = renderer.render(&mut target, out_surface.size, Transform::Normal) {
                        let _ = frame.clear(dark_acrylic, &[damage]);
                        let _ = draw_render_elements(&mut frame, 1.0, &elements, &[damage]);

                        // Dibujar puntero del ratón con escala física precisa
                        if !cursor_elems.is_empty() {
                            let _ = draw_render_elements(&mut frame, scale, &cursor_elems, &[damage]);
                        } else if let Some((phys_x, phys_y)) = cursor_draw_point {
                            if let Some(ref tex) = cursor_texture {
                                let cursor_size = (24.0 * scale).round() as i32;
                                let cursor_rect = Rectangle::new((phys_x, phys_y).into(), (cursor_size, cursor_size).into());
                                let _ = frame.render_texture_from_to(
                                    tex,
                                    Rectangle::from_size(tex.size()).to_f64(),
                                    cursor_rect,
                                    &[Rectangle::from_size(cursor_rect.size)],
                                    &[],
                                    Transform::Normal,
                                    1.0,
                                    None,
                                    &[],
                                );
                            }
                        }

                        let _ = frame.finish();
                    }
                }

                if let Err(e) = out_surface.surface.queue_buffer(None, None, ()) {
                    warn!("Error al encolar buffer en DRM: {:?}", e);
                }
            }
        }

        // Notificar fotogramas a los clientes
        let now_ms = start_time.elapsed().as_millis() as u32;
        for surface in state.xdg_shell_state.toplevel_surfaces() {
            send_frames_surface_tree(surface.wl_surface(), now_ms);
            for (popup, _) in smithay::desktop::PopupManager::popups_for_surface(surface.wl_surface()) {
                send_frames_surface_tree(popup.wl_surface(), now_ms);
            }
        }

        for output in state.space.outputs() {
            let layer_map = smithay::desktop::layer_map_for_output(output);
            for layer in layer_map.layers() {
                send_frames_surface_tree(layer.wl_surface(), now_ms);
                for (popup, _) in smithay::desktop::PopupManager::popups_for_surface(layer.wl_surface()) {
                    send_frames_surface_tree(popup.wl_surface(), now_ms);
                }
            }
        }

        display.flush_clients()?;
        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Compositor Malus DE DRM finalizado limpiamente.");
    Ok(())
}
