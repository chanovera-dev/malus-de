use serde::{Deserialize, Serialize};

/// Mensajes IPC transmitidos por Unix Domain Socket entre el compositor y los clientes shell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Invocación de Spotlight (abrir / cerrar modal)
    ToggleSpotlight,
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
