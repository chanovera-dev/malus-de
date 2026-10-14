use serde::{Deserialize, Serialize};

/// Configuración global de Malus DE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalusConfig {
    pub appearance: AppearanceConfig,
    pub dock: DockConfig,
    pub top_bar: TopBarConfig,
    pub input: InputConfig,
    pub tiling: TilingConfig,
    pub link: LinkConfig,
    pub power: PowerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String,
    pub dark_mode: bool,
    pub button_layout_left: bool,
    pub acrylic_blur: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "default-dark".to_string(),
            dark_mode: true,
            button_layout_left: true,
            acrylic_blur: true,
        }
    }
}

impl AppearanceConfig {
    /// Formato de botones de ventana para GNOME / GTK:
    /// - Izquierda (macOS): `close,minimize,maximize:`
    /// - Derecha (Windows/GNOME): `:minimize,maximize,close`
    pub fn button_layout_string(&self) -> &'static str {
        if self.button_layout_left {
            "close,minimize,maximize:"
        } else {
            ":minimize,maximize,close"
        }
    }
}

use std::path::PathBuf;

/// Inicializa el entorno de configuración aislado para Malus DE en `$HOME/.config/malus-de`.
/// Configura el backend `keyfile` de GSettings y los archivos `settings.ini` de GTK3/GTK4
/// con el layout de botones a la macOS (`close,minimize,maximize:`) sin alterar dconf del host.
pub fn setup_isolated_session_environment() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let malus_config_dir = PathBuf::from(home).join(".config").join("malus-de");

    // 1. Directorio glib-2.0/settings para el backend GSettings keyfile
    let keyfile_dir = malus_config_dir.join("glib-2.0").join("settings");
    std::fs::create_dir_all(&keyfile_dir)?;
    let keyfile_path = keyfile_dir.join("keyfile");
    let keyfile_content = r#"[org/gnome/desktop/wm/preferences]
button-layout='close,minimize,maximize:'

[org/gnome/desktop/interface]
color-scheme='prefer-dark'
"#;
    std::fs::write(keyfile_path, keyfile_content)?;

    // 2. Archivos settings.ini para GTK3 y GTK4
    let gtk3_dir = malus_config_dir.join("gtk-3.0");
    std::fs::create_dir_all(&gtk3_dir)?;
    let gtk_ini_content = r#"[Settings]
gtk-decoration-layout=close,minimize,maximize:
gtk-application-prefer-dark-theme=1
"#;
    std::fs::write(gtk3_dir.join("settings.ini"), gtk_ini_content)?;

    let gtk4_dir = malus_config_dir.join("gtk-4.0");
    std::fs::create_dir_all(&gtk4_dir)?;
    std::fs::write(gtk4_dir.join("settings.ini"), gtk_ini_content)?;

    // Exportar variables en el entorno del proceso actual
    std::env::set_var("XDG_CONFIG_HOME", &malus_config_dir);
    std::env::set_var("GSETTINGS_BACKEND", "keyfile");

    Ok(malus_config_dir)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockConfig {
    pub position: DockPosition,
    pub style: DockStyle,
    pub icon_size: u32,
    pub magnification_enabled: bool,
    pub magnification_max_size: u32,
    pub floating_gap: u32,
    pub corner_radius: u32,
    pub background_opacity: f32,
    pub blur_strength: u32,
    pub genie_effect: bool,
    pub indicator_style: DockIndicatorStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockPosition {
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockStyle {
    Floating,
    Panel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockIndicatorStyle {
    Dot,
    Dash,
    Glow,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            position: DockPosition::Bottom,
            style: DockStyle::Floating,
            icon_size: 48,
            magnification_enabled: true,
            magnification_max_size: 96,
            floating_gap: 12,
            corner_radius: 24,
            background_opacity: 0.70,
            blur_strength: 24,
            genie_effect: true,
            indicator_style: DockIndicatorStyle::Dot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopBarConfig {
    pub height: u32,
    pub global_menu: bool,
    pub clock_show_seconds: bool,
}

impl Default for TopBarConfig {
    fn default() -> Self {
        Self {
            height: 28,
            global_menu: true,
            clock_show_seconds: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub natural_scrolling: bool,
    pub shake_to_find_cursor: bool,
    pub show_desktop_gesture: bool,
    pub hot_corners: HotCornersConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotCornersConfig {
    pub enabled: bool,
    pub delay_ms: u32,
    pub top_left: HotCornerAction,
    pub top_right: HotCornerAction,
    pub bottom_left: HotCornerAction,
    pub bottom_right: HotCornerAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotCornerAction {
    None,
    MissionControl,
    NotificationCenter,
    ShowDesktop,
    LockScreen,
}

impl Default for HotCornersConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_ms: 150,
            top_left: HotCornerAction::MissionControl,
            top_right: HotCornerAction::NotificationCenter,
            bottom_left: HotCornerAction::LockScreen,
            bottom_right: HotCornerAction::ShowDesktop,
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            natural_scrolling: true,
            shake_to_find_cursor: true,
            show_desktop_gesture: true,
            hot_corners: HotCornersConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilingConfig {
    pub enabled: bool,
    pub gaps: u32,
    pub default_layout: TilingLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TilingLayout {
    BinaryTree,
    Columns,
}

impl Default for TilingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            gaps: 12,
            default_layout: TilingLayout::BinaryTree,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    pub device_name: String,
    pub auto_clipboard: bool,
    pub pause_on_call: bool,
    pub sync_notifications: bool,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            device_name: "Malus Linux".to_string(),
            auto_clipboard: true,
            pause_on_call: true,
            sync_notifications: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    pub default_profile: PowerProfile,
    pub auto_power_saver_on_low_battery: bool,
    pub low_battery_threshold: u8,
    pub reduce_effects_on_power_saver: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSaver,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            default_profile: PowerProfile::Balanced,
            auto_power_saver_on_low_battery: true,
            low_battery_threshold: 20,
            reduce_effects_on_power_saver: true,
        }
    }
}

impl Default for MalusConfig {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig::default(),
            dock: DockConfig::default(),
            top_bar: TopBarConfig::default(),
            input: InputConfig::default(),
            tiling: TilingConfig::default(),
            link: LinkConfig::default(),
            power: PowerConfig::default(),
        }
    }
}

