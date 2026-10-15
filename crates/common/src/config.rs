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
    pub displays: DisplaysConfig,
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

    // 2.1. Archivo gtk.css aislado (oculta títulos de ventana redundantes en la cabecera estilo macOS)
    let gtk_css_content = r#"/* Malus DE - Estilo macOS aislado para aplicaciones GTK3 y GTK4 */
windowtitle .title,
headerbar windowtitle .title {
    opacity: 0;
    font-size: 0px;
    min-height: 0px;
    margin: 0px;
    padding: 0px;
}

windowcontrols.start {
    margin-right: 6px;
    margin-left: 2px;
}
"#;
    let _ = std::fs::write(gtk3_dir.join("gtk.css"), gtk_css_content);
    let _ = std::fs::write(gtk4_dir.join("gtk.css"), gtk_css_content);

    // 3. Biblioteca compartida para garantizar botones consistentes en GTK3/GTK4 (macOS style)
    let lib_dir = malus_config_dir.join("lib");
    let _ = std::fs::create_dir_all(&lib_dir);
    let so_path = lib_dir.join("libmalus_gtk_layout.so");
    let c_source = r#"#define _GNU_SOURCE
#include <dlfcn.h>
#include <glib-object.h>
#include <string.h>

typedef void (*orig_get_prop)(GObject*, const char*, GValue*);
typedef void (*orig_pack)(void*, void*);
typedef const char* (*get_action_fn)(void*);
typedef void (*orig_set_title)(void*, const char*);

void g_object_get_property(GObject *object, const char *property_name, GValue *value) {
    static orig_get_prop real = NULL;
    if (!real) {
        real = (orig_get_prop)dlsym(RTLD_NEXT, "g_object_get_property");
    }
    if (property_name && strcmp(property_name, "gtk-decoration-layout") == 0) {
        if (!G_IS_VALUE(value)) {
            g_value_init(value, G_TYPE_STRING);
        }
        g_value_set_string(value, "close,minimize,maximize:");
        return;
    }
    real(object, property_name, value);
}

void g_object_get(gpointer object, const gchar *first_property_name, ...) {
    static void (*real_valist)(gpointer, const gchar*, va_list) = NULL;
    if (!real_valist) {
        real_valist = (void (*)(gpointer, const gchar*, va_list))dlsym(RTLD_NEXT, "g_object_get_valist");
    }
    va_list var_args;
    va_start(var_args, first_property_name);
    if (first_property_name && strcmp(first_property_name, "gtk-decoration-layout") == 0) {
        gchar **str_ptr = va_arg(var_args, gchar**);
        if (str_ptr) {
            *str_ptr = g_strdup("close,minimize,maximize:");
        }
    } else if (real_valist) {
        real_valist(object, first_property_name, var_args);
    }
    va_end(var_args);
}

void adw_header_bar_pack_start(void *self, void *child) {
    static orig_pack real_start = NULL;
    static orig_pack real_end = NULL;
    static get_action_fn get_action = NULL;
    if (!real_start) {
        real_start = (orig_pack)dlsym(RTLD_NEXT, "adw_header_bar_pack_start");
        real_end = (orig_pack)dlsym(RTLD_NEXT, "adw_header_bar_pack_end");
        get_action = (get_action_fn)dlsym(RTLD_DEFAULT, "gtk_actionable_get_action_name");
    }

    // Mover el botón de búsqueda global a la derecha (pack_end) para no colisionar con los semáforos macOS
    if (child && real_end && get_action) {
        const char *act = get_action(child);
        if (act && (strstr(act, "search-global") != NULL || strstr(act, "search") != NULL)) {
            real_end(self, child);
            return;
        }
    }

    if (real_start) {
        real_start(self, child);
    }
}

void adw_window_title_set_title(void *self, const char *title) {
    static orig_set_title real_title = NULL;
    if (!real_title) {
        real_title = (orig_set_title)dlsym(RTLD_NEXT, "adw_window_title_set_title");
    }
    if (real_title) {
        // En Malus DE los programas se identifican en el menú global superior estilo macOS
        if (title && (strcmp(title, "Archivos") == 0 || strcmp(title, "Files") == 0)) {
            real_title(self, "");
            return;
        }
        real_title(self, title);
    }
}
"#;
    let c_path = lib_dir.join("malus_gtk_layout.c");
    let needs_compile = !so_path.exists() || std::fs::read_to_string(&c_path).map(|c| c != c_source).unwrap_or(true);
    if needs_compile && std::fs::write(&c_path, c_source).is_ok() {
        let _ = std::process::Command::new("gcc")
            .args([
                "-O2", "-shared", "-fPIC",
                c_path.to_str().unwrap(),
                "-o", so_path.to_str().unwrap(),
                "-I/usr/include/glib-2.0",
                "-I/usr/lib64/glib-2.0/include",
            ])
            .status();
    }

    if so_path.exists() {
        std::env::set_var("LD_PRELOAD", &so_path);
    }

    // 4. Configuración de monitores y escalado fraccionario (~/.config/malus-de/monitors.toml)
    let monitors_path = malus_config_dir.join("monitors.toml");
    if !monitors_path.exists() {
        let displays_cfg = DisplaysConfig::load();
        if let Ok(toml_str) = toml::to_string_pretty(&displays_cfg) {
            let header = "# Configuración de Pantallas y Monitores para Malus DE\n# Soporte nativo para múltiples pantallas 4K con escalado fraccionario independiente.\n\n";
            let _ = std::fs::write(&monitors_path, format!("{}{}", header, toml_str));
        }
    }

    // 5. Configuración del Dock (~/.config/malus-de/dock.toml)
    let dock_path = malus_config_dir.join("dock.toml");
    if !dock_path.exists() {
        let dock_cfg = DockConfig::load_or_create_default();
        let _ = dock_cfg.save();
    }

    // Exportar variables en el entorno del proceso actual
    std::env::set_var("XDG_CONFIG_HOME", &malus_config_dir);
    std::env::set_var("GSETTINGS_BACKEND", "keyfile");

    Ok(malus_config_dir)
}


/// Configuración de una aplicación anclada en el Dock
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DockAppConfig {
    pub name: String,
    pub icon: String,
    pub command: String,
    #[serde(default)]
    pub matchers: Vec<String>,
}

impl DockAppConfig {
    pub fn matches(&self, identifier: &str) -> bool {
        let id_lower = identifier.to_lowercase();
        if self.name.to_lowercase() == id_lower {
            return true;
        }
        for m in &self.matchers {
            if id_lower.contains(&m.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
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
    pub apps: Vec<DockAppConfig>,
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
            magnification_max_size: 62,
            floating_gap: 10,
            corner_radius: 24,
            background_opacity: 0.70,
            blur_strength: 24,
            genie_effect: true,
            indicator_style: DockIndicatorStyle::Dot,
            apps: vec![
                DockAppConfig {
                    name: "Archivos".to_string(),
                    icon: "system-file-manager".to_string(),
                    command: "nautilus --new-window".to_string(),
                    matchers: vec!["nautilus".into(), "archivos".into(), "org.gnome.nautilus".into()],
                },
                DockAppConfig {
                    name: "Terminal".to_string(),
                    icon: "utilities-terminal".to_string(),
                    command: "ptyxis -s".to_string(),
                    matchers: vec!["ptyxis".into(), "terminal".into(), "org.gnome.ptyxis".into()],
                },
                DockAppConfig {
                    name: "Editor de Texto".to_string(),
                    icon: "accessories-text-editor".to_string(),
                    command: "gnome-text-editor -s".to_string(),
                    matchers: vec!["texteditor".into(), "text-editor".into(), "gedit".into(), "org.gnome.texteditor".into()],
                },
                DockAppConfig {
                    name: "Calculadora".to_string(),
                    icon: "org.gnome.Calculator".to_string(),
                    command: "gnome-calculator".to_string(),
                    matchers: vec!["calculator".into(), "calculadora".into(), "org.gnome.calculator".into()],
                },
                DockAppConfig {
                    name: "Configuración".to_string(),
                    icon: "preferences-system".to_string(),
                    command: "gnome-control-center".to_string(),
                    matchers: vec!["control-center".into(), "settings".into(), "configuración".into(), "configuracion".into(), "org.gnome.settings".into()],
                },
            ],
        }
    }
}

impl DockConfig {
    /// Determina la ruta canónica del archivo `dock.toml`
    pub fn config_path() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let p1 = PathBuf::from(&xdg).join("dock.toml");
            let p2 = PathBuf::from(&xdg).join("malus-de").join("dock.toml");
            if p1.exists() {
                return p1;
            }
            if p2.exists() {
                return p2;
            }
            if PathBuf::from(&xdg).ends_with("malus-de") {
                return p1;
            } else {
                return p2;
            }
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("malus-de").join("dock.toml")
    }

    /// Carga la configuración desde `dock.toml`, o crea el archivo con la configuración predeterminada si no existe
    pub fn load_or_create_default() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<DockConfig>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    /// Guarda la configuración actual en el archivo `dock.toml`
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let header = "# Configuración del Dock de Malus DE\n# Personalización de tamaño de iconos, magnificación, gap flotante y aplicaciones ancladas.\n\n";
        std::fs::write(path, format!("{}{}", header, toml_str))
    }

    /// Ancla una aplicación al final del dock si no está ya presente
    pub fn pin_app(&mut self, app: DockAppConfig) -> bool {
        if self.apps.iter().any(|a| a.name.eq_ignore_ascii_case(&app.name) || (!a.command.is_empty() && a.command == app.command)) {
            return false;
        }
        self.apps.push(app);
        true
    }

    /// Desancla una aplicación del dock por su nombre
    pub fn unpin_app(&mut self, name: &str) -> bool {
        let prev_len = self.apps.len();
        self.apps.retain(|a| !a.name.eq_ignore_ascii_case(name));
        self.apps.len() < prev_len
    }

    /// Mueve una aplicación de posición (reordenar)
    pub fn move_app(&mut self, from_idx: usize, to_idx: usize) -> bool {
        if from_idx < self.apps.len() && to_idx < self.apps.len() && from_idx != to_idx {
            let item = self.apps.remove(from_idx);
            self.apps.insert(to_idx, item);
            true
        } else {
            false
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
            displays: DisplaysConfig::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_master_scale() -> f64 {
    1.666667
}

fn default_brightness() -> f64 {
    80.0
}

/// Configuración multi-monitor con escalado fraccionario independiente y Enlace Físico 1:1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaysConfig {
    /// Activa el enlace matemático 1:1 para que ventanas y textos midan los mismos milímetros físicos en todos los monitores
    #[serde(default = "default_true")]
    pub linked_physical_1to1: bool,
    /// Escala maestra aplicada al monitor principal (1.0 = 100%, 1.666667 = 166.7%, 2.0 = 200%)
    #[serde(default = "default_master_scale")]
    pub master_scale: f64,
    /// Nivel de brillo de pantalla configurado (5.0 a 100.0)
    #[serde(default = "default_brightness")]
    pub brightness: f64,
    /// Lista de pantallas configuradas
    pub outputs: Vec<DisplayOutputConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOutputConfig {
    pub connector: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f64,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub primary: bool,
    /// Diagonal física real del panel en pulgadas (permite corregir datos genéricos de EDID)
    #[serde(default)]
    pub diagonal_inches: Option<f64>,
}

impl DisplayOutputConfig {
    /// Obtiene la diagonal real o estimada en pulgadas
    pub fn effective_diagonal(&self) -> f64 {
        if let Some(d) = self.diagonal_inches {
            return d;
        }
        let n = self.name.to_lowercase();
        if n.contains("27") {
            27.0
        } else if n.contains("24") || n.contains("mi monitor") {
            24.0
        } else if n.contains("32") {
            32.0
        } else {
            27.0
        }
    }

    /// Calcula la densidad física de píxeles por pulgada (PPI)
    pub fn ppi(&self) -> f64 {
        let diag = self.effective_diagonal();
        if diag <= 0.0 {
            return 163.0;
        }
        let diag_px = ((self.width as f64).powi(2) + (self.height as f64).powi(2)).sqrt();
        diag_px / diag
    }

    /// Dimensiones físicas aproximadas del panel en milímetros
    pub fn physical_size_mm(&self) -> (f64, f64) {
        let ppi = self.ppi();
        let w_mm = (self.width as f64 / ppi) * 25.4;
        let h_mm = (self.height as f64 / ppi) * 25.4;
        (w_mm, h_mm)
    }

    /// Resolución lógica en el espacio virtual del compositor
    pub fn logical_size(&self) -> (i32, i32) {
        let s = if self.scale > 0.0 { self.scale } else { 1.0 };
        (
            (self.width as f64 / s).round() as i32,
            (self.height as f64 / s).round() as i32,
        )
    }
}

impl Default for DisplaysConfig {
    fn default() -> Self {
        let mut cfg = Self {
            linked_physical_1to1: true,
            master_scale: 1.666667,
            brightness: 80.0,
            outputs: vec![
                DisplayOutputConfig {
                    connector: "DP-1".to_string(),
                    name: "LG Ultra HD 27\"".to_string(),
                    width: 3840,
                    height: 2160,
                    refresh_rate: 59.997,
                    x: 0,
                    y: 519,
                    scale: 1.666667,
                    primary: true,
                    diagonal_inches: Some(27.0),
                },
                DisplayOutputConfig {
                    connector: "HDMI-1".to_string(),
                    name: "Mi Monitor 24\"".to_string(),
                    width: 3840,
                    height: 2160,
                    refresh_rate: 60.0,
                    x: 2304,
                    y: 0,
                    scale: 1.875,
                    primary: false,
                    diagonal_inches: Some(24.0),
                },
            ],
        };
        cfg.apply_1to1_link();
        cfg
    }
}

impl DisplaysConfig {
    /// Carga la configuración de monitores desde `~/.config/malus-de/monitors.toml`,
    /// o como fallback parsea `~/.config/monitors.xml` de GNOME.
    pub fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let toml_path = PathBuf::from(&home).join(".config/malus-de/monitors.toml");
        if toml_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&toml_path) {
                if let Ok(mut config) = toml::from_str::<DisplaysConfig>(&content) {
                    if config.linked_physical_1to1 {
                        config.apply_1to1_link();
                    }
                    return config;
                }
            }
        }

        // Fallback: intentar parsear ~/.config/monitors.xml
        let xml_path = PathBuf::from(&home).join(".config/monitors.xml");
        if xml_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&xml_path) {
                if let Some(mut config) = Self::from_gnome_xml(&content) {
                    if config.linked_physical_1to1 {
                        config.apply_1to1_link();
                    }
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Aplica el enlace físico 1:1 sincronizando la escala de todos los monitores
    /// según su densidad de píxeles (PPI) real respecto al monitor primario.
    pub fn apply_1to1_link(&mut self) {
        if !self.linked_physical_1to1 || self.outputs.is_empty() {
            return;
        }

        let primary_idx = self.outputs.iter().position(|o| o.primary).unwrap_or(0);
        let primary_ppi = self.outputs[primary_idx].ppi();
        let primary_scale = self.master_scale;
        self.outputs[primary_idx].scale = primary_scale;

        let primary_logical_w = (self.outputs[primary_idx].width as f64 / primary_scale).round() as i32;

        for (i, output) in self.outputs.iter_mut().enumerate() {
            if i == primary_idx {
                continue;
            }
            let out_ppi = output.ppi();
            // Escala 1:1 exacta en base a PPI: Escala_i = Escala_primaria * (PPI_i / PPI_primaria)
            let linked_scale = primary_scale * (out_ppi / primary_ppi);
            // Redondear a 4 decimales para consistencia numérica en Wayland
            output.scale = (linked_scale * 10000.0).round() / 10000.0;

            // Si el monitor secundario está adyacente a la derecha, ajustar su coordenada X lógica continua
            if output.x >= primary_logical_w - 100 {
                output.x = primary_logical_w;
            }
        }
    }

    /// Actualiza la escala maestra unificada y recalcula ambas pantallas 1:1
    pub fn set_master_scale(&mut self, new_scale: f64) {
        self.master_scale = new_scale;
        if self.linked_physical_1to1 {
            self.apply_1to1_link();
        } else if let Some(p) = self.outputs.iter_mut().find(|o| o.primary) {
            p.scale = new_scale;
        }
    }

    /// Guarda la configuración actual en `$HOME/.config/malus-de/monitors.toml`
    pub fn save(&self) -> std::io::Result<()> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let toml_path = PathBuf::from(&home).join(".config/malus-de/monitors.toml");
        if let Some(parent) = toml_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let header = "# Configuración de Pantallas y Monitores para Malus DE\n# Soporte nativo para múltiples pantallas 4K con Enlace Físico 1:1.\n\n";
        std::fs::write(toml_path, format!("{}{}", header, toml_str))
    }

    /// Parsea la configuración de pantallas desde el formato XML de GNOME
    pub fn from_gnome_xml(xml: &str) -> Option<Self> {
        let config_blocks: Vec<&str> = xml.split("<configuration>").collect();
        let active_block = config_blocks.last()?;
        let parts: Vec<&str> = active_block.split("<logicalmonitor>").collect();
        if parts.len() <= 1 {
            return None;
        }

        let mut outputs = Vec::new();
        let mut master_scale = 1.666667;

        for part in &parts[1..] {
            let x = extract_tag_value(part, "x").and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
            let y = extract_tag_value(part, "y").and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
            let scale = extract_tag_value(part, "scale").and_then(|v| v.parse::<f64>().ok()).unwrap_or(1.0);
            let primary = extract_tag_value(part, "primary").map(|v| v == "yes").unwrap_or(false);
            let connector = extract_tag_value(part, "connector").unwrap_or_else(|| "default".to_string());
            let product = extract_tag_value(part, "product").unwrap_or_else(|| connector.clone());
            let width = extract_tag_value(part, "width").and_then(|v| v.parse::<u32>().ok()).unwrap_or(3840);
            let height = extract_tag_value(part, "height").and_then(|v| v.parse::<u32>().ok()).unwrap_or(2160);
            let rate = extract_tag_value(part, "rate").and_then(|v| v.parse::<f64>().ok()).unwrap_or(60.0);

            let diagonal_inches = if product.contains("27") || product.contains("LG") {
                Some(27.0)
            } else if product.contains("24") || product.contains("Mi") {
                Some(24.0)
            } else {
                None
            };

            if primary {
                master_scale = scale;
            }

            outputs.push(DisplayOutputConfig {
                connector,
                name: product,
                width,
                height,
                refresh_rate: rate,
                x,
                y,
                scale,
                primary,
                diagonal_inches,
            });
        }

        if outputs.is_empty() {
            None
        } else {
            let mut cfg = DisplaysConfig {
                linked_physical_1to1: true,
                master_scale,
                brightness: 80.0,
                outputs,
            };
            cfg.apply_1to1_link();
            Some(cfg)
        }
    }

    /// Busca la configuración de un monitor por nombre de conector (p. ej. "DP-1" o "HDMI-A-1")
    pub fn find_for_connector(&self, connector: &str) -> Option<&DisplayOutputConfig> {
        let norm = connector.trim_start_matches("card1-").trim_start_matches("card0-");
        self.outputs.iter().find(|o| {
            let o_norm = o.connector.trim_start_matches("card1-").trim_start_matches("card0-");
            o_norm.eq_ignore_ascii_case(norm)
                || (norm.starts_with("HDMI") && o_norm.starts_with("HDMI"))
                || (norm.starts_with("DP") && o_norm.starts_with("DP"))
        })
    }
}

fn extract_tag_value(xml_snippet: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);
    let start = xml_snippet.find(&open_tag)? + open_tag.len();
    let end = xml_snippet[start..].find(&close_tag)? + start;
    Some(xml_snippet[start..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_config_default_serialization() {
        let default_cfg = DockConfig::default();
        let toml_str = toml::to_string_pretty(&default_cfg).expect("Serialización falló");
        let parsed: DockConfig = toml::from_str(&toml_str).expect("Deserialización falló");
        assert_eq!(parsed.apps.len(), 5);
        assert_eq!(parsed.icon_size, 48);
        assert_eq!(parsed.magnification_max_size, 62);
        assert_eq!(parsed.floating_gap, 10);
        assert_eq!(parsed.position, DockPosition::Bottom);
    }

    #[test]
    fn test_dock_config_partial_deserialization() {
        let partial_toml = r#"
            icon_size = 56
            magnification_enabled = false
            floating_gap = 14

            [[apps]]
            name = "Test App"
            icon = "test-icon"
            command = "test-cmd"
            matchers = ["testapp"]
        "#;
        let parsed: DockConfig = toml::from_str(partial_toml).expect("Deserialización parcial falló");
        assert_eq!(parsed.icon_size, 56);
        assert!(!parsed.magnification_enabled);
        assert_eq!(parsed.floating_gap, 14);
        assert_eq!(parsed.corner_radius, 24); // default
        assert_eq!(parsed.apps.len(), 1);
        assert_eq!(parsed.apps[0].name, "Test App");
    }

    #[test]
    fn test_dock_config_pin_unpin_move() {
        let mut cfg = DockConfig::default();
        let initial_count = cfg.apps.len();

        // Pin new app
        let new_app = DockAppConfig {
            name: "Spotify".to_string(),
            icon: "spotify".to_string(),
            command: "spotify".to_string(),
            matchers: vec!["spotify".into()],
        };
        assert!(cfg.pin_app(new_app.clone()));
        assert_eq!(cfg.apps.len(), initial_count + 1);
        // Duplicate pin should return false
        assert!(!cfg.pin_app(new_app));

        // Move app
        let last_idx = cfg.apps.len() - 1;
        assert!(cfg.move_app(last_idx, 0));
        assert_eq!(cfg.apps[0].name, "Spotify");

        // Unpin app
        assert!(cfg.unpin_app("Spotify"));
        assert_eq!(cfg.apps.len(), initial_count);
        assert!(!cfg.unpin_app("NonExistentApp"));
    }

    #[test]
    fn test_dock_app_config_matches() {
        let app = DockAppConfig {
            name: "Terminal".to_string(),
            icon: "utilities-terminal".to_string(),
            command: "ptyxis -s".to_string(),
            matchers: vec!["ptyxis".into(), "terminal".into(), "org.gnome.ptyxis".into()],
        };

        assert!(app.matches("Terminal"));
        assert!(app.matches("ptyxis"));
        assert!(app.matches("org.gnome.ptyxis.something"));
        assert!(!app.matches("firefox"));
    }

    #[test]
    fn test_dock_config_load_or_create_default() {
        let cfg = DockConfig::load_or_create_default();
        let path = DockConfig::config_path();
        assert!(path.exists());
        assert_eq!(cfg.apps.len(), 5);
        assert_eq!(cfg.icon_size, 48);
    }

    #[test]
    fn test_displays_config_brightness() {
        let mut cfg = DisplaysConfig::default();
        assert_eq!(cfg.brightness, 80.0);
        cfg.brightness = 95.0;
        let toml_str = toml::to_string(&cfg).unwrap();
        let parsed: DisplaysConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.brightness, 95.0);
    }
}


