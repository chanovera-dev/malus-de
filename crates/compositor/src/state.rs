use std::collections::HashMap;
use std::os::unix::io::OwnedFd;
use smithay::{
    delegate_compositor, delegate_data_device, delegate_layer_shell, delegate_output,
    delegate_seat, delegate_shm, delegate_xdg_shell,
    desktop::{layer_map_for_output, Space},
    input::{Seat, SeatHandler, SeatState},
    output::Output,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::State as XdgState,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{
                wl_buffer::WlBuffer, wl_output::WlOutput, wl_seat::WlSeat, wl_surface::WlSurface,
            },
            Client, Display, DisplayHandle,
        },
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        output::OutputHandler,
        selection::{
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
            SelectionHandler,
        },
        shell::{
            wlr_layer::{
                Layer, LayerSurface, WlrLayerShellHandler, WlrLayerShellState,
            },
            xdg::{
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            },
        },
        seat::WaylandFocus,
        shm::{ShmHandler, ShmState},
    },
};

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub struct MalusState {
    pub display_handle: DisplayHandle,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<MalusState>,
    pub data_device_state: DataDeviceState,
    pub seat: Seat<MalusState>,
    pub space: Space<smithay::desktop::Window>,
    pub window_restore_geometries: HashMap<WlSurface, (Point<i32, Logical>, Size<i32, Logical>)>,
    pub socket_name: String,
    pub is_running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapMode {
    LeftHalf,
    RightHalf,
    Maximize,
    CenterRestore,
}

impl MalusState {
    pub fn new(display: &Display<MalusState>) -> Self {
        let dh = display.handle();
        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let layer_shell_state = WlrLayerShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&dh, "malus-seat-0");
        let data_device_state = DataDeviceState::new::<Self>(&dh);

        Self {
            display_handle: dh,
            compositor_state,
            xdg_shell_state,
            layer_shell_state,
            shm_state,
            seat_state,
            data_device_state,
            seat,
            space: Space::default(),
            window_restore_geometries: HashMap::new(),
            socket_name: String::new(),
            is_running: true,
        }
    }

    pub fn window_for_surface(&self, surface: &WlSurface) -> Option<smithay::desktop::Window> {
        self.space
            .elements()
            .find(|window| window.wl_surface().as_deref() == Some(surface))
            .cloned()
    }
}

/// Aplica un modo de Aero Snap / Tiling a una ventana de manera atómica
pub fn snap_window(
    state: &mut MalusState,
    window: &smithay::desktop::Window,
    mode: SnapMode,
    usable_zone: Rectangle<i32, Logical>,
) {
    let Some(surface) = window.wl_surface() else { return };
    let surface = surface.into_owned();

    let cur_loc = state.space.element_location(window).unwrap_or_default();
    let cur_geo = window.geometry();

    // Guardar geometría previa flotante para poder restaurarla con Super + Abajo
    if mode != SnapMode::CenterRestore && !state.window_restore_geometries.contains_key(&surface) {
        state.window_restore_geometries.insert(surface.clone(), (cur_loc, cur_geo.size));
    }

    let (target_loc, target_size, xdg_states) = match mode {
        SnapMode::LeftHalf => (
            Point::from((usable_zone.loc.x, usable_zone.loc.y)),
            Size::from((usable_zone.size.w / 2, usable_zone.size.h)),
            vec![XdgState::TiledLeft],
        ),
        SnapMode::RightHalf => (
            Point::from((usable_zone.loc.x + usable_zone.size.w / 2, usable_zone.loc.y)),
            Size::from((usable_zone.size.w - usable_zone.size.w / 2, usable_zone.size.h)),
            vec![XdgState::TiledRight],
        ),
        SnapMode::Maximize => (
            Point::from((usable_zone.loc.x, usable_zone.loc.y)),
            Size::from((usable_zone.size.w, usable_zone.size.h)),
            vec![XdgState::Maximized],
        ),
        SnapMode::CenterRestore => {
            if let Some((saved_loc, saved_size)) = state.window_restore_geometries.remove(&surface) {
                (saved_loc, saved_size, vec![])
            } else {
                (cur_loc, cur_geo.size, vec![])
            }
        }
    };

    tracing::info!(
        "🪟 Aero Snap aplicado: modo={:?}, loc={:?}, size={:?}",
        mode,
        target_loc,
        target_size
    );

    if let Some(toplevel) = window.toplevel() {
        toplevel.with_pending_state(|top_state| {
            top_state.size = Some(target_size);
            top_state.states.unset(XdgState::Maximized);
            top_state.states.unset(XdgState::TiledLeft);
            top_state.states.unset(XdgState::TiledRight);
            for s in xdg_states {
                top_state.states.set(s);
            }
        });
        toplevel.send_configure();
    }

    state.space.map_element(window.clone(), target_loc, true);
    state.space.raise_element(window, true);
}

// Delegaciones de protocolos de Smithay
delegate_compositor!(MalusState);
delegate_shm!(MalusState);
delegate_seat!(MalusState);
delegate_xdg_shell!(MalusState);
delegate_layer_shell!(MalusState);
delegate_output!(MalusState);
delegate_data_device!(MalusState);

impl CompositorHandler for MalusState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);

        // Si es una ventana regular en Space, actualizamos sus dimensiones internas
        if let Some(window) = self.space.elements().find(|w| w.wl_surface().as_deref() == Some(surface)) {
            window.on_commit();
        }

        // Si la superficie pertenece a una Layer Surface (Top Bar, Dock, Notificaciones, etc.)
        let outputs: Vec<_> = self.space.outputs().cloned().collect();
        let target_output = outputs.into_iter().find(|o| {
            let map = smithay::desktop::layer_map_for_output(o);
            map.layer_for_surface(surface, smithay::desktop::WindowSurfaceType::all()).is_some()
        });

        if let Some(output) = target_output {
            let mut should_focus_overlay = false;
            {
                let mut map = smithay::desktop::layer_map_for_output(&output);
                map.arrange();

                let initial_configure_sent = smithay::wayland::compositor::with_states(surface, |states| {
                    states
                        .data_map
                        .get::<smithay::wayland::shell::wlr_layer::LayerSurfaceData>()
                        .map(|d| d.lock().unwrap().initial_configure_sent)
                        .unwrap_or(false)
                });

                tracing::info!("🔄 Commit de Layer Surface: initial_configure_sent={}", initial_configure_sent);

                if let Some(layer) = map.layer_for_surface(surface, smithay::desktop::WindowSurfaceType::all()) {
                    if !initial_configure_sent {
                        tracing::info!("📤 Enviando initial configure a layer surface");
                        layer.layer_surface().send_configure();
                    }
                    if layer.layer() == smithay::wayland::shell::wlr_layer::Layer::Overlay {
                        should_focus_overlay = true;
                    }
                }
            }

            if should_focus_overlay {
                let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                if let Some(kb) = self.seat.get_keyboard() {
                    kb.set_focus(self, Some(surface.clone()), serial);
                }
            }
        }
    }
}

impl BufferHandler for MalusState {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for MalusState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for MalusState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {}
}

impl SelectionHandler for MalusState {
    type SelectionUserData = ();
}

impl DataDeviceHandler for MalusState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for MalusState {}

impl ServerDndGrabHandler for MalusState {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}

impl OutputHandler for MalusState {}

impl XdgShellHandler for MalusState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        tracing::info!("✨ Nueva ventana XDG Toplevel recibida y mapeada en Space!");
        let window = smithay::desktop::Window::new_wayland_window(surface.clone());
        surface.send_configure();
        self.space.map_element(window, (50, 50), true);
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {}

    fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {
        tracing::info!("🖱️ [XdgShell] move_request recibido del cliente: serial={:?}", serial);
        let seat: Seat<Self> = Seat::from_resource(&seat).unwrap();
        let pointer = seat.get_pointer().unwrap();

        if !pointer.has_grab(serial) {
            tracing::warn!("⚠️ [XdgShell] move_request ignorado: pointer no tiene grab activo con serial {:?}", serial);
            return;
        }

        let start_data = pointer.grab_start_data().unwrap();
        let Some(window) = self.window_for_surface(surface.wl_surface()) else {
            tracing::warn!("⚠️ [XdgShell] move_request: no se encontró ventana para la superficie");
            return;
        };

        let initial_window_location = self.space.element_location(&window).unwrap_or_default();

        let grab = crate::grabs::MoveSurfaceGrab {
            start_data,
            window: window.clone(),
            initial_window_location,
        };

        self.space.raise_element(&window, true);
        pointer.set_grab(self, grab, serial, smithay::input::pointer::Focus::Clear);
        tracing::info!("✅ [XdgShell] MoveSurfaceGrab activado exitosamente para arrastre por título!");
    }

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        tracing::info!("🪟 [XdgShell] maximize_request recibido del cliente");
        let Some(window) = self.window_for_surface(surface.wl_surface()) else { return };
        let usable_zone = self.space.outputs().next().map(|output| {
            let layer_map = smithay::desktop::layer_map_for_output(output);
            let zone = layer_map.non_exclusive_zone();
            let output_geo = self.space.output_geometry(output).unwrap_or_default();
            if zone.size.w > 0 && zone.size.h > 0 {
                zone
            } else {
                Rectangle::new((0, 32).into(), (output_geo.size.w, output_geo.size.h - 32).into())
            }
        });

        if let Some(usable_zone) = usable_zone {
            snap_window(self, &window, SnapMode::Maximize, usable_zone);
        }
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        tracing::info!("🪟 [XdgShell] unmaximize_request recibido del cliente");
        let surf = surface.wl_surface();
        if !self.window_restore_geometries.contains_key(surf) {
            tracing::info!("ℹ️ [XdgShell] unmaximize_request ignorado: la ventana no estaba en estado maximizado.");
            return;
        }

        let Some(window) = self.window_for_surface(surf) else { return };
        let usable_zone = self.space.outputs().next().map(|output| {
            let layer_map = smithay::desktop::layer_map_for_output(output);
            let zone = layer_map.non_exclusive_zone();
            let output_geo = self.space.output_geometry(output).unwrap_or_default();
            if zone.size.w > 0 && zone.size.h > 0 {
                zone
            } else {
                Rectangle::new((0, 32).into(), (output_geo.size.w, output_geo.size.h - 32).into())
            }
        });

        if let Some(usable_zone) = usable_zone {
            snap_window(self, &window, SnapMode::CenterRestore, usable_zone);
        }
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        tracing::info!("🗑️ [XdgShell] Ventana toplevel destruida");
        self.window_restore_geometries.remove(surface.wl_surface());
    }
}

impl WlrLayerShellHandler for MalusState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        tracing::info!("✨ Nueva Layer Surface recibida: namespace={}", namespace);
        let output = output
            .as_ref()
            .and_then(Output::from_resource)
            .or_else(|| self.space.outputs().next().cloned());

        if let Some(output) = output {
            let desktop_surface = smithay::desktop::LayerSurface::new(surface, namespace);
            let mut map = layer_map_for_output(&output);
            let _ = map.map_layer(&desktop_surface);
        }
    }

    fn layer_destroyed(&mut self, surface: LayerSurface) {
        if let Some((mut map, layer)) = self.space.outputs().find_map(|o| {
            let map = layer_map_for_output(o);
            let layer = map
                .layers()
                .find(|&layer| layer.layer_surface() == &surface)
                .cloned();
            layer.map(|layer| (map, layer))
        }) {
            map.unmap_layer(&layer);
        }

        let serial = smithay::utils::SERIAL_COUNTER.next_serial();
        if let Some(kb) = self.seat.get_keyboard() {
            let top_win_surface = self.space.elements().next().and_then(|w| w.wl_surface().map(|s| s.into_owned()));
            kb.set_focus(self, top_win_surface, serial);
        }
    }
}
