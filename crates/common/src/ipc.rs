use serde::{Deserialize, Serialize};
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// Mensajes IPC transmitidos por Unix Domain Socket entre el compositor y los clientes shell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Invocación de Spotlight (abrir / cerrar modal)
    ToggleSpotlight,
    /// Notificación de cambio de ventana activa / enfocada
    ActiveAppChanged {
        app_id: Option<String>,
        title: Option<String>,
    },
    /// Notificación de lista de aplicaciones en ejecución (app_ids o títulos)
    RunningAppsChanged {
        running_apps: Vec<String>,
    },
    /// Notificación de coordenadas de slot en el Dock para animación Genie Effect
    RegisterDockSlot {
        window_id: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    /// Solicitud de minimización con efecto Genie
    AnimateMinimize {
        window_id: u64,
    },
    /// Solicitud de restauración con efecto Genie inverso
    AnimateRestore {
        window_id: u64,
    },
    /// Cambio de espacio de trabajo virtual
    SwitchWorkspace {
        index: usize,
    },
    /// Toggle Mission Control (Exposé)
    ToggleMissionControl,
    /// Toggle Dispersión de Ventanas para Mostrar / Ocultar Escritorio (Super + D)
    ToggleShowDesktop,
    /// Invocación del Historial Visual del Portapapeles (Super + V)
    ToggleClipboardHistory,
    /// Invocación del Selector de Emojis y Símbolos (Super + .)
    ToggleEmojiPicker,
    /// Invocación del Menú flotante Snap Layouts (Super + Z)
    ToggleSnapLayouts,
    /// Invocación de la herramienta de captura / grabación de pantalla
    TriggerScreenshot {
        interactive: bool,
        record_video: bool,
    },
    /// Invocación del Extractor de Texto en Pantalla OCR (Super + Shift + T)
    TriggerOcrCapture,
    /// Conmutar entre Modo Flotante y Modo Mosaico Automático (Super + Y)
    ToggleAutoTiling,
    /// Conmutar Picture-in-Picture universal para la ventana activa (Super + P)
    TogglePictureInPicture,
    /// Filtrado difuso dinámico de ventanas en Mission Control al teclear
    MissionControlFilter {
        query: String,
    },
    /// Disparar localizador acústico del teléfono (Hacer sonar teléfono)
    MalusLinkRingPhone,
    /// Enviar archivo al teléfono móvil emparejado (AirDrop Linux)
    MalusLinkSendFile {
        file_path: String,
    },
    /// Solicitud de cambio de modo de rendimiento (power-profiles-daemon)
    SetPowerProfile {
        profile: crate::config::PowerProfile,
    },
    /// Notificación de cambio de modo de rendimiento activo
    PowerProfileChanged {
        profile: crate::config::PowerProfile,
    },
    /// Notificación de recarga de tema
    ThemeChanged {
        theme_name: String,
        is_dark: bool,
    },
}

/// Obtiene la ruta canónica del socket IPC Unix de Malus DE
pub fn ipc_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("malus-de-ipc.sock")
    } else {
        PathBuf::from("/tmp/malus-de-ipc.sock")
    }
}

/// Servidor y difusor de mensajes IPC para el compositor
#[derive(Clone, Default)]
pub struct IpcBroadcaster {
    clients: Arc<Mutex<Vec<UnixStream>>>,
    last_active: Arc<Mutex<Option<String>>>,
    last_running: Arc<Mutex<Option<String>>>,
}

impl IpcBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(Vec::new())),
            last_active: Arc::new(Mutex::new(None)),
            last_running: Arc::new(Mutex::new(None)),
        }
    }

    /// Inicializa el servidor Unix Listener en un hilo dedicado en segundo plano
    pub fn start_server(&self) -> std::io::Result<()> {
        let path = ipc_socket_path();
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        let clients = self.clients.clone();
        let last_active = self.last_active.clone();
        let last_running = self.last_running.clone();

        thread::spawn(move || {
            tracing::info!("📡 Servidor IPC Malus DE escuchando en: {:?}", path);
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    // Enviar los últimos estados conocidos al nuevo cliente conectado
                    if let Some(ref act) = *last_active.lock().unwrap() {
                        let _ = stream.write_all(act.as_bytes());
                    }
                    if let Some(ref run) = *last_running.lock().unwrap() {
                        let _ = stream.write_all(run.as_bytes());
                    }
                    let mut lock = clients.lock().unwrap();
                    lock.push(stream);
                }
            }
        });

        Ok(())
    }

    /// Difunde un mensaje JSON a todos los clientes shell conectados (Top Bar, Dock, etc.)
    pub fn broadcast(&self, msg: &IpcMessage) {
        let Ok(json) = serde_json::to_string(msg) else { return };
        let line = format!("{}\n", json);
        let bytes = line.as_bytes();

        match msg {
            IpcMessage::ActiveAppChanged { .. } => {
                *self.last_active.lock().unwrap() = Some(line.clone());
            }
            IpcMessage::RunningAppsChanged { .. } => {
                *self.last_running.lock().unwrap() = Some(line.clone());
            }
            _ => {}
        }

        let mut clients = self.clients.lock().unwrap();
        clients.retain_mut(|client| {
            client.write_all(bytes).is_ok()
        });
    }
}

