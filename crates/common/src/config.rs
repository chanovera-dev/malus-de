use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
            theme: "default-light".to_string(),
            dark_mode: false,
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

/// Reglas CSS de controles de ventana macOS (Traffic Lights) para tema Claro
pub const MALUS_GTK_CSS_WINDOWCONTROLS_LIGHT: &str = r#"/* Controles de ventana estilo macOS (Claro) */
windowcontrols {
    border-spacing: 8px;
    padding: 0;
    margin-top: 10px;
    margin-bottom: 10px;
}

windowcontrols.start {
    margin-left: 10px;
    margin-right: 12px;
}

windowcontrols.end {
    margin-right: 10px;
    margin-left: 12px;
}

windowcontrols > button,
headerbar button.titlebutton {
    min-width: 13px;
    min-height: 13px;
    padding: 0;
    margin: 0;
    border-radius: 9999px;
    border-style: solid;
    border-width: 0.5px;
    box-shadow: none;
    outline: none;
    background-clip: padding-box;
    transition: background-color 100ms ease, border-color 100ms ease, opacity 100ms ease;
}

windowcontrols > button > image,
headerbar button.titlebutton > image {
    padding: 0;
    margin: 0;
    min-width: 13px;
    min-height: 13px;
    border-radius: 9999px;
    opacity: 0;
    transition: opacity 120ms ease;
    background-color: transparent;
    box-shadow: none;
    -gtk-icon-size: 8px;
}

windowcontrols:hover > button > image,
windowcontrols > button:hover > image,
headerbar:hover button.titlebutton > image,
headerbar button.titlebutton:hover > image {
    opacity: 0.85;
}

windowcontrols > button.close,
headerbar button.titlebutton.close {
    background-color: #ff5f56;
    border-color: #e0443e;
}
windowcontrols > button.close:hover,
headerbar button.titlebutton.close:hover {
    background-color: #ff473d;
    border-color: #c93630;
}
windowcontrols > button.close:active,
headerbar button.titlebutton.close:active {
    background-color: #bf3933;
    border-color: #a82f2a;
}
windowcontrols > button.close > image,
headerbar button.titlebutton.close > image {
    color: #4c0000;
}

windowcontrols > button.minimize,
headerbar button.titlebutton.minimize {
    background-color: #ffbd2e;
    border-color: #dea123;
}
windowcontrols > button.minimize:hover,
headerbar button.titlebutton.minimize:hover {
    background-color: #ffb114;
    border-color: #c48c18;
}
windowcontrols > button.minimize:active,
headerbar button.titlebutton.minimize:active {
    background-color: #bf8516;
    border-color: #9e6d0f;
}
windowcontrols > button.minimize > image,
headerbar button.titlebutton.minimize > image {
    color: #593b00;
}

windowcontrols > button.maximize,
headerbar button.titlebutton.maximize {
    background-color: #27c93f;
    border-color: #1aab29;
}
windowcontrols > button.maximize:hover,
headerbar button.titlebutton.maximize:hover {
    background-color: #1fbd36;
    border-color: #159322;
}
windowcontrols > button.maximize:active,
headerbar button.titlebutton.maximize:active {
    background-color: #158a25;
    border-color: #0f6c1a;
}
windowcontrols > button.maximize > image,
headerbar button.titlebutton.maximize > image {
    color: #004d11;
}

window:backdrop windowcontrols > button,
.backdrop windowcontrols > button,
window:backdrop headerbar button.titlebutton,
.backdrop headerbar button.titlebutton {
    background-color: rgba(0, 0, 0, 0.12);
    border-color: rgba(0, 0, 0, 0.08);
}
window:backdrop windowcontrols > button > image,
.backdrop windowcontrols > button > image,
window:backdrop headerbar button.titlebutton > image,
.backdrop headerbar button.titlebutton > image {
    opacity: 0;
}
"#;

/// Reglas CSS de controles de ventana macOS (Traffic Lights) para tema Oscuro
pub const MALUS_GTK_CSS_WINDOWCONTROLS_DARK: &str = r#"/* Controles de ventana estilo macOS (Oscuro) */
windowcontrols {
    border-spacing: 8px;
    padding: 0;
    margin-top: 10px;
    margin-bottom: 10px;
}

windowcontrols.start {
    margin-left: 10px;
    margin-right: 12px;
}

windowcontrols.end {
    margin-right: 10px;
    margin-left: 12px;
}

windowcontrols > button,
headerbar button.titlebutton {
    min-width: 13px;
    min-height: 13px;
    padding: 0;
    margin: 0;
    border-radius: 9999px;
    border-style: solid;
    border-width: 0.5px;
    box-shadow: none;
    outline: none;
    background-clip: padding-box;
    transition: background-color 100ms ease, border-color 100ms ease, opacity 100ms ease;
}

windowcontrols > button > image,
headerbar button.titlebutton > image {
    padding: 0;
    margin: 0;
    min-width: 13px;
    min-height: 13px;
    border-radius: 9999px;
    opacity: 0;
    transition: opacity 120ms ease;
    background-color: transparent;
    box-shadow: none;
    -gtk-icon-size: 8px;
}

windowcontrols:hover > button > image,
windowcontrols > button:hover > image,
headerbar:hover button.titlebutton > image,
headerbar button.titlebutton:hover > image {
    opacity: 0.85;
}

windowcontrols > button.close,
headerbar button.titlebutton.close {
    background-color: #ff5f56;
    border-color: #e0443e;
}
windowcontrols > button.close:hover,
headerbar button.titlebutton.close:hover {
    background-color: #ff473d;
    border-color: #c93630;
}
windowcontrols > button.close:active,
headerbar button.titlebutton.close:active {
    background-color: #bf3933;
    border-color: #a82f2a;
}
windowcontrols > button.close > image,
headerbar button.titlebutton.close > image {
    color: #4c0000;
}

windowcontrols > button.minimize,
headerbar button.titlebutton.minimize {
    background-color: #ffbd2e;
    border-color: #dea123;
}
windowcontrols > button.minimize:hover,
headerbar button.titlebutton.minimize:hover {
    background-color: #ffb114;
    border-color: #c48c18;
}
windowcontrols > button.minimize:active,
headerbar button.titlebutton.minimize:active {
    background-color: #bf8516;
    border-color: #9e6d0f;
}
windowcontrols > button.minimize > image,
headerbar button.titlebutton.minimize > image {
    color: #593b00;
}

windowcontrols > button.maximize,
headerbar button.titlebutton.maximize {
    background-color: #27c93f;
    border-color: #1aab29;
}
windowcontrols > button.maximize:hover,
headerbar button.titlebutton.maximize:hover {
    background-color: #1fbd36;
    border-color: #159322;
}
windowcontrols > button.maximize:active,
headerbar button.titlebutton.maximize:active {
    background-color: #158a25;
    border-color: #0f6c1a;
}
windowcontrols > button.maximize > image,
headerbar button.titlebutton.maximize > image {
    color: #004d11;
}

window:backdrop windowcontrols > button,
.backdrop windowcontrols > button,
window:backdrop headerbar button.titlebutton,
.backdrop headerbar button.titlebutton {
    background-color: rgba(255, 255, 255, 0.18);
    border-color: rgba(255, 255, 255, 0.10);
}
window:backdrop windowcontrols > button > image,
.backdrop windowcontrols > button > image,
window:backdrop headerbar button.titlebutton > image,
.backdrop headerbar button.titlebutton > image {
    opacity: 0;
}
"#;

/// Reglas CSS de Malus DE para controles de ventana estilo macOS y recortes squircle
pub const MALUS_GTK_CSS_SQUIRCLE_RULES: &str = r#"/* Malus DE - Estilo macOS aislado para aplicaciones GTK3 y GTK4 */
windowtitle .title,
headerbar windowtitle .title {
    opacity: 0;
    font-size: 0px;
    min-height: 0px;
    margin: 0px;
    padding: 0px;
}

/* Ventanas GTK3, GTK4 y Libadwaita: esquinas rectas a nivel de widget para permitir
   que el compositor GPU de Malus DE aplique el recorte Squircle superelíptico continuo (n=4.2)
   sin doble recorte circular previo ni márgenes de sombra CSD */
window,
window.background,
window.csd,
window.unified,
window.solid-csd,
window.messagedialog,
window.dialog,
decoration,
.solid-csd,
headerbar,
headerbar.flat,
.titlebar,
tabbar,
tab-bar,
navigation-view,
navigation-view > widget,
leaflet,
flap,
toast-overlay,
adw-toolbar-view,
adw-window,
adw-application-window {
    border-radius: 0px;
    box-shadow: none;
    margin: 0px;
}

headerbar,
headerbar.flat,
.titlebar,
tabbar,
tab-bar {
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
    border-radius: 0px;
}

/* Ventanas maximizadas o en pantalla completa: esquinas rectas (0px) */
window.maximized,
window.fullscreen,
window.maximized.background,
window.fullscreen.background,
window.maximized headerbar,
window.fullscreen headerbar {
    border-radius: 0px;
}

/* Menús emergentes, popovers y context menus estilo Squircle macOS */
popover > contents,
popover.menu > contents,
popover.background > contents,
menu,
.menu {
    border-radius: 14px;
    padding: 6px;
}

/* Elementos de menú con squircle highlight */
popover modelbutton,
popover .popover-menu-item,
menu menuitem,
.menu menuitem {
    border-radius: 8px;
    min-height: 26px;
    padding: 4px 10px;
    margin: 1px 2px;
}

/* Enlaces estilo macOS con squircle hover */
link,
link:hover,
link:active,
link:visited,
button.link,
button.link:hover {
    border-radius: 7px;
    padding: 2px 6px;
}
"#;

/// Genera la hoja de estilos CSS completa de Malus DE (controles macOS + recortes squircle)
pub fn get_malus_gtk_css(dark: bool) -> String {
    let controls = if dark {
        MALUS_GTK_CSS_WINDOWCONTROLS_DARK
    } else {
        MALUS_GTK_CSS_WINDOWCONTROLS_LIGHT
    };
    format!("{}\n{}", controls, MALUS_GTK_CSS_SQUIRCLE_RULES)
}

/// Inicializa el entorno de configuración aislado para Malus DE en `$HOME/.config/malus-de`.
/// Configura el backend `keyfile` de GSettings y los archivos `settings.ini` de GTK3/GTK4
/// con el layout de botones a la macOS (`close,minimize,maximize:`) sin alterar dconf del host.
pub fn setup_isolated_session_environment() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let malus_config_dir = PathBuf::from(&home).join(".config").join("malus-de");

    // 1. Directorio glib-2.0/settings para el backend GSettings keyfile (preservar si ya existe)
    let keyfile_dir = malus_config_dir.join("glib-2.0").join("settings");
    std::fs::create_dir_all(&keyfile_dir)?;
    let keyfile_path = keyfile_dir.join("keyfile");
    if !keyfile_path.exists() {
        let keyfile_content = r#"[org/gnome/desktop/wm/preferences]
button-layout='close,minimize,maximize:'

[org/gnome/desktop/interface]
color-scheme='prefer-light'
cursor-theme='macOS'
cursor-size=24
icon-theme='Adwaita'
gtk-theme='Adwaita'

[org/gnome/settings-daemon/plugins/xsettings]
overrides={'Gtk/ShellShowsAppMenu': <1>, 'Gtk/ShellShowsMenubar': <1>}
"#;
        std::fs::write(&keyfile_path, keyfile_content)?;
    }

    // 2. Archivos settings.ini para GTK3 y GTK4 (preservar configuraciones existentes del usuario)
    let gtk3_dir = malus_config_dir.join("gtk-3.0");
    std::fs::create_dir_all(&gtk3_dir)?;
    let gtk3_ini = gtk3_dir.join("settings.ini");
    if !gtk3_ini.exists() {
        let gtk_ini_content = r#"[Settings]
gtk-decoration-layout=close,minimize,maximize:
gtk-application-prefer-dark-theme=0
gtk-cursor-theme-name=macOS
gtk-cursor-theme-size=24
gtk-theme-name=Adwaita
"#;
        std::fs::write(&gtk3_ini, gtk_ini_content)?;
    }

    let gtk4_dir = malus_config_dir.join("gtk-4.0");
    std::fs::create_dir_all(&gtk4_dir)?;
    let gtk4_ini = gtk4_dir.join("settings.ini");
    if !gtk4_ini.exists() {
        let gtk_ini_content = r#"[Settings]
gtk-decoration-layout=close,minimize,maximize:
gtk-application-prefer-dark-theme=0
gtk-cursor-theme-name=macOS
gtk-cursor-theme-size=24
gtk-theme-name=Adwaita
"#;
        std::fs::write(&gtk4_ini, gtk_ini_content)?;
    }

    // 2.1. Archivos gtk.css y gtk-dark.css nativos estilo macOS (GTK3 y GTK4)
    let user_gtk3 = PathBuf::from(&home).join(".config/gtk-3.0");
    let user_gtk4 = PathBuf::from(&home).join(".config/gtk-4.0");
    let _ = std::fs::create_dir_all(&user_gtk3);
    let _ = std::fs::create_dir_all(&user_gtk4);

    let light_css = get_malus_gtk_css(false);
    let dark_css = get_malus_gtk_css(true);

    // GTK 3
    let _ = std::fs::write(gtk3_dir.join("gtk.css"), &light_css);
    let _ = std::fs::write(user_gtk3.join("gtk.css"), &light_css);

    // GTK 4 (sesión Malus DE y usuario)
    let _ = std::fs::write(gtk4_dir.join("gtk.css"), &light_css);
    let _ = std::fs::write(gtk4_dir.join("gtk-dark.css"), &dark_css);
    let _ = std::fs::write(user_gtk4.join("gtk.css"), &light_css);
    let _ = std::fs::write(user_gtk4.join("gtk-dark.css"), &dark_css);

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

    // 6. Configuración de Teclado y Entrada (~/.config/malus-de/input.toml)
    let input_path = malus_config_dir.join("input.toml");
    if !input_path.exists() {
        let input_cfg = InputConfig::load();
        let _ = input_cfg.save();
    }

    // 7. Configuración automática de controles semáforo macOS para Firefox y Brave
    configure_browsers_traffic_lights(&home);

    // Exportar variables en el entorno del proceso actual
    std::env::set_var("XDG_CONFIG_HOME", &malus_config_dir);
    std::env::set_var("GSETTINGS_BACKEND", "keyfile");
    std::env::set_var("XCURSOR_THEME", "macOS");
    std::env::set_var("XCURSOR_SIZE", "24");

    Ok(malus_config_dir)
}

/// Configura los controles de ventana estilo semáforo macOS para Firefox y Brave
pub fn configure_browsers_traffic_lights(home: &str) {
    let firefox_css = r#"/* Malus DE - Controles de ventana macOS (Traffic Lights) para Firefox */
.titlebar-buttonbox-container {
    -moz-box-ordinal-group: 0 !important;
    order: -1 !important;
}

.titlebar-buttonbox {
    margin-left: 8px !important;
    margin-right: 8px !important;
}

.titlebar-button {
    padding: 0 !important;
    margin: auto 3.5px !important;
    width: 13px !important;
    height: 13px !important;
    min-width: 13px !important;
    min-height: 13px !important;
    max-width: 13px !important;
    max-height: 13px !important;
    border-radius: 9999px !important;
    appearance: none !important;
    -moz-appearance: none !important;
    background-color: transparent !important;
    box-shadow: none !important;
    border-style: solid !important;
    border-width: 0.5px !important;
    transition: background-color 100ms ease, border-color 100ms ease !important;
}

.titlebar-button > .toolbarbutton-icon {
    display: none !important;
}

.titlebar-close {
    background-color: #ff5f56 !important;
    border-color: #e0443e !important;
}
.titlebar-close:hover {
    background-color: #ff473d !important;
    border-color: #c93630 !important;
}
.titlebar-close:active {
    background-color: #bf3933 !important;
    border-color: #a82f2a !important;
}

.titlebar-min {
    background-color: #ffbd2e !important;
    border-color: #dea123 !important;
}
.titlebar-min:hover {
    background-color: #ffb114 !important;
    border-color: #c48c18 !important;
}
.titlebar-min:active {
    background-color: #bf8516 !important;
    border-color: #9e6d0f !important;
}

.titlebar-max {
    background-color: #27c93f !important;
    border-color: #1aab29 !important;
}
.titlebar-max:hover {
    background-color: #1fbd36 !important;
    border-color: #159322 !important;
}
.titlebar-max:active {
    background-color: #158a25 !important;
    border-color: #0f6c1a !important;
}

:root[unfocused] .titlebar-button {
    background-color: rgba(0, 0, 0, 0.12) !important;
    border-color: rgba(0, 0, 0, 0.08) !important;
}
@media (prefers-color-scheme: dark) {
    :root[unfocused] .titlebar-button {
        background-color: rgba(255, 255, 255, 0.15) !important;
        border-color: rgba(255, 255, 255, 0.1) !important;
    }
}
"#;

    // 1. Firefox: Configurar user.js y chrome/userChrome.css para todos los perfiles
    let ff_bases = [
        format!("{}/.config/mozilla/firefox", home),
        format!("{}/.config/malus-de/mozilla/firefox", home),
    ];
    for base_str in &ff_bases {
        let base = Path::new(base_str);
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let is_profile = path.extension().is_some()
                        || path.join("prefs.js").exists()
                        || path.join("compatibility.ini").exists();
                    if is_profile {
                        // user.js
                        let user_js = path.join("user.js");
                        let pref_line = "user_pref(\"toolkit.legacyUserProfileCustomizations.stylesheets\", true);\n";
                        let existing = std::fs::read_to_string(&user_js).unwrap_or_default();
                        if !existing.contains("toolkit.legacyUserProfileCustomizations.stylesheets") {
                            let _ = std::fs::write(&user_js, format!("{}\n{}", existing, pref_line));
                        }
                        // userChrome.css
                        let chrome_dir = path.join("chrome");
                        let _ = std::fs::create_dir_all(&chrome_dir);
                        let _ = std::fs::write(chrome_dir.join("userChrome.css"), firefox_css);
                    }
                }
            }
        }
    }

    // 2. Brave: Forzar system_theme: 1 en las preferencias de Brave
    let brave_prefs = [
        format!("{}/.config/BraveSoftware/Brave-Origin/Default/Preferences", home),
        format!("{}/.config/malus-de/BraveSoftware/Brave-Origin/Default/Preferences", home),
    ];
    for bp in &brave_prefs {
        if let Ok(content) = std::fs::read_to_string(bp) {
            if let Ok(mut json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json_val.as_object_mut() {
                    let extensions = obj.entry("extensions").or_insert_with(|| serde_json::json!({}));
                    if let Some(ext_obj) = extensions.as_object_mut() {
                        let theme = ext_obj.entry("theme").or_insert_with(|| serde_json::json!({}));
                        if let Some(theme_obj) = theme.as_object_mut() {
                            theme_obj.insert("system_theme".to_string(), serde_json::json!(1));
                            if let Ok(new_content) = serde_json::to_string_pretty(&json_val) {
                                let _ = std::fs::write(bp, new_content);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Brave Launcher Wrapper con soporte GTK4
    let bin_dir = Path::new(home).join(".local/bin");
    let _ = std::fs::create_dir_all(&bin_dir);
    let brave_wrapper = r#"#!/usr/bin/env bash
# Malus DE - Brave Launcher con controles de ventana GTK4 nativos macOS
REAL_BRAVE="/opt/brave.com/brave-origin/brave-origin"
if [ ! -x "$REAL_BRAVE" ]; then
    REAL_BRAVE="/usr/bin/brave-origin-stable"
fi
exec "$REAL_BRAVE" --gtk-version=4 "$@"
"#;
    for bname in &["brave", "brave-browser"] {
        let bpath = bin_dir.join(bname);
        let _ = std::fs::remove_file(&bpath);
        if std::fs::write(&bpath, brave_wrapper).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&bpath, std::fs::Permissions::from_mode(0o755));
            }
        }
    }

    // 4. Desktop entry override para Brave
    let apps_dir = Path::new(home).join(".local/share/applications");
    let _ = std::fs::create_dir_all(&apps_dir);
    let desktop_source = "/usr/share/applications/brave-origin.desktop";
    let desktop_dest = apps_dir.join("brave-origin.desktop");
    if let Ok(dcontent) = std::fs::read_to_string(desktop_source) {
        let modified = dcontent.replace(
            "Exec=/usr/bin/brave-origin-stable",
            "Exec=/usr/bin/brave-origin-stable --gtk-version=4",
        );
        let _ = std::fs::write(desktop_dest, modified);
    }
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
                    name: "Ajustes del Sistema".to_string(),
                    icon: "preferences-system".to_string(),
                    command: "malus-de-settings".to_string(),
                    matchers: vec!["malus-de-settings".into(), "dev.malus.settings".into(), "ajustes".into(), "settings".into(), "configuración".into(), "configuracion".into()],
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

fn default_keyboard_layout() -> String {
    "latam".to_string()
}

fn default_keyboard_model() -> String {
    "pc105".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    /// Distribución de teclado XKB activa ("latam" = Español Latinoamericano, "es" = Español España, "us" = Inglés EE. UU.)
    #[serde(default = "default_keyboard_layout")]
    pub keyboard_layout: String,
    /// Variante XKB opcional (ej. "" o "deadtilde")
    #[serde(default)]
    pub keyboard_variant: String,
    /// Modelo físico de teclado XKB (por defecto "pc105")
    #[serde(default = "default_keyboard_model")]
    pub keyboard_model: String,
    /// Opciones XKB adicionales (ej. Some("grp:alt_shift_toggle"))
    #[serde(default)]
    pub keyboard_options: Option<String>,
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
            keyboard_layout: default_keyboard_layout(),
            keyboard_variant: String::new(),
            keyboard_model: default_keyboard_model(),
            keyboard_options: None,
            natural_scrolling: true,
            shake_to_find_cursor: true,
            show_desktop_gesture: true,
            hot_corners: HotCornersConfig::default(),
        }
    }
}

impl InputConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/malus-de/input.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut cfg) = toml::from_str::<InputConfig>(&content) {
                    if cfg.keyboard_layout.trim().is_empty() {
                        cfg.keyboard_layout = "latam".to_string();
                    }
                    return cfg;
                }
            }
        }
        let cfg = Self::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let header = "# Configuración de Teclado y Entrada para Malus DE\n# Soporte nativo para Español Latinoamericano (latam), Español (es) e Inglés (us).\n\n";
        std::fs::write(path, format!("{}{}", header, toml_str))
    }

    pub fn layout_display_name(&self) -> &'static str {
        match self.keyboard_layout.as_str() {
            "latam" => "🇲🇽 Español (Latinoamérica)",
            "es" => "🇪🇸 Español (España)",
            "us" => "🇺🇸 Inglés (EE. UU.)",
            _ => "⌨️ Personalizado",
        }
    }

    pub fn layout_short_badge(&self) -> &'static str {
        match self.keyboard_layout.as_str() {
            "latam" => "LAT",
            "es" => "ES",
            "us" => "US",
            _ => "KB",
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

fn default_screen_corner_radius() -> f64 {
    32.0
}

fn default_window_corner_radius() -> f64 {
    32.0
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
    /// Radio de curvatura squircle de las esquinas del escritorio (en puntos lógicos, ej. 24.0 o 32.0). 0.0 desactiva el recorte.
    #[serde(default = "default_screen_corner_radius")]
    pub screen_corner_radius: f64,
    /// Radio de curvatura squircle de las ventanas (en puntos lógicos, ej. 32.0). 0.0 desactiva el recorte.
    #[serde(default = "default_window_corner_radius")]
    pub window_corner_radius: f64,
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
            master_scale: 1.71,
            brightness: 80.0,
            screen_corner_radius: 32.0,
            window_corner_radius: 32.0,
            outputs: vec![
                DisplayOutputConfig {
                    connector: "DP-1".to_string(),
                    name: "LG Ultra HD 24\"".to_string(),
                    width: 3840,
                    height: 2160,
                    refresh_rate: 59.997,
                    x: 0,
                    y: 169,
                    scale: 1.71,
                    primary: true,
                    diagonal_inches: Some(24.0),
                },
                DisplayOutputConfig {
                    connector: "HDMI-A-1".to_string(),
                    name: "Mi Monitor 27\"".to_string(),
                    width: 3840,
                    height: 2160,
                    refresh_rate: 60.0,
                    x: 2246,
                    y: 0,
                    scale: 1.5,
                    primary: false,
                    diagonal_inches: Some(27.0),
                },
            ],
        };
        cfg.apply_linked_preset(1);
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
                    config.recalculate_layout();
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
                    } else {
                        config.recalculate_layout();
                    }
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Recalcula las coordenadas lógicas X e Y de las pantallas para mantenerlas adyacentes
    /// sin huecos muertos ni solapamientos cuando cambia la escala o el orden horizontal.
    pub fn recalculate_layout(&mut self) {
        if self.outputs.is_empty() {
            return;
        }

        // Ordenar índices de izquierda a derecha según su coordenada X actual
        let mut order: Vec<usize> = (0..self.outputs.len()).collect();
        order.sort_by_key(|&idx| (self.outputs[idx].x, !self.outputs[idx].primary));

        let mut cursor_x: i32 = 0;
        for idx in order {
            self.outputs[idx].x = cursor_x;
            let (lw, _) = self.outputs[idx].logical_size();
            cursor_x += lw.max(100);
        }

        // Normalizar coordenadas verticales Y para que al menos un monitor toque el borde superior (y = 0)
        if let Some(min_y) = self.outputs.iter().map(|o| o.y).min() {
            for o in &mut self.outputs {
                o.y = (o.y - min_y).max(0);
            }
        }
    }

    /// Devuelve el preset enlazado activo (Some(1), Some(2), o None)
    /// Preset 1: LG 24" a 171% (1.71) y Mi 27" a 150% (1.50)
    /// Preset 2: LG 24" a 152.5% (1.525) y Mi 27" a 133.33% (1.333333)
    pub fn current_linked_preset(&self) -> Option<u8> {
        if !self.linked_physical_1to1 || self.outputs.len() < 2 {
            return None;
        }
        let o24 = self.outputs.iter().find(|o| {
            let c = o.connector.to_lowercase();
            let n = o.name.to_lowercase();
            c.contains("dp-1") || n.contains("24") || n.contains("lg")
        })?;
        let o27 = self.outputs.iter().find(|o| {
            let c = o.connector.to_lowercase();
            let n = o.name.to_lowercase();
            c.contains("hdmi") || n.contains("27") || n.contains("mi")
        })?;

        if (o24.scale - 1.71).abs() < 0.015 && (o27.scale - 1.50).abs() < 0.015 {
            Some(1)
        } else if (o24.scale - 1.525).abs() < 0.015 && (o27.scale - 1.333333).abs() < 0.015 {
            Some(2)
        } else {
            None
        }
    }

    /// Aplica un preset específico de resolución enlazada:
    /// Preset 1: LG 24" a 171% (1.71) y Mi 27" a 150% (1.50) [Recomendado / Texto Nítido]
    /// Preset 2: LG 24" a 152.5% (1.525) y Mi 27" a 133.33% (1.333333) [Más Espacio de Trabajo]
    pub fn apply_linked_preset(&mut self, preset: u8) {
        self.linked_physical_1to1 = true;
        for o in &mut self.outputs {
            let n = o.name.to_lowercase();
            let c = o.connector.to_lowercase();
            let is_24 = c.contains("dp-1") || n.contains("24") || n.contains("lg");
            let is_27 = c.contains("hdmi") || n.contains("27") || n.contains("mi");

            match preset {
                1 => {
                    if is_24 {
                        o.scale = 1.71;
                    } else if is_27 {
                        o.scale = 1.50;
                    }
                }
                2 => {
                    if is_24 {
                        o.scale = 1.525;
                    } else if is_27 {
                        o.scale = 1.333333;
                    }
                }
                _ => {}
            }
        }

        if let Some(p) = self.outputs.iter().find(|o| o.primary) {
            self.master_scale = p.scale;
        }

        self.recalculate_layout();
    }

    /// Aplica el enlace físico 1:1 sincronizando la escala de todos los monitores
    /// según su densidad de píxeles (PPI) real respecto al monitor primario.
    pub fn apply_1to1_link(&mut self) {
        if !self.linked_physical_1to1 || self.outputs.is_empty() {
            self.recalculate_layout();
            return;
        }

        // Si ya coincide con un preset, mantenerlo exacto
        if let Some(preset) = self.current_linked_preset() {
            self.apply_linked_preset(preset);
            return;
        }

        if (self.master_scale - 1.71).abs() < 0.03 {
            self.apply_linked_preset(1);
            return;
        }
        if (self.master_scale - 1.525).abs() < 0.03 {
            self.apply_linked_preset(2);
            return;
        }

        let primary_idx = self.outputs.iter().position(|o| o.primary).unwrap_or(0);
        let primary_ppi = self.outputs[primary_idx].ppi();
        let primary_scale = self.master_scale.clamp(0.75, 3.0);
        self.master_scale = (primary_scale * 10000.0).round() / 10000.0;
        self.outputs[primary_idx].scale = self.master_scale;

        for (i, output) in self.outputs.iter_mut().enumerate() {
            if i == primary_idx {
                continue;
            }
            let out_ppi = output.ppi();
            // Escala 1:1 exacta en base a PPI: Escala_i = Escala_primaria * (PPI_i / PPI_primaria)
            let linked_scale = (primary_scale * (out_ppi / primary_ppi)).clamp(0.75, 3.0);
            output.scale = (linked_scale * 10000.0).round() / 10000.0;
        }

        self.recalculate_layout();
    }

    /// Actualiza la escala maestra unificada y recalcula ambas pantallas 1:1
    pub fn set_master_scale(&mut self, new_scale: f64) {
        let clamped = (new_scale.clamp(0.75, 3.0) * 10000.0).round() / 10000.0;
        self.master_scale = clamped;
        if self.linked_physical_1to1 {
            self.apply_1to1_link();
        } else {
            if let Some(p) = self.outputs.iter_mut().find(|o| o.primary) {
                p.scale = clamped;
            }
            self.recalculate_layout();
        }
    }

    /// Ajusta la escala de una pantalla específica por su índice.
    /// Si `linked_physical_1to1` está activo, ajusta proporcionalmente todas las demás pantallas según su PPI.
    pub fn set_output_scale(&mut self, output_idx: usize, new_scale: f64) {
        if output_idx >= self.outputs.len() {
            return;
        }
        let clamped = (new_scale.clamp(0.75, 3.0) * 10000.0).round() / 10000.0;
        self.outputs[output_idx].scale = clamped;

        if self.linked_physical_1to1 {
            let o = &self.outputs[output_idx];
            let c = o.connector.to_lowercase();
            let n = o.name.to_lowercase();
            let is_24 = c.contains("dp-1") || n.contains("24") || n.contains("lg");
            let is_27 = c.contains("hdmi") || n.contains("27") || n.contains("mi");

            if (is_24 && (clamped - 1.71).abs() < 0.015) || (is_27 && (clamped - 1.50).abs() < 0.015) {
                self.apply_linked_preset(1);
                return;
            } else if (is_24 && (clamped - 1.525).abs() < 0.015) || (is_27 && (clamped - 1.333333).abs() < 0.015) {
                self.apply_linked_preset(2);
                return;
            }

            let ref_ppi = self.outputs[output_idx].ppi();
            for i in 0..self.outputs.len() {
                if i != output_idx {
                    let i_ppi = self.outputs[i].ppi();
                    let linked = (clamped * (i_ppi / ref_ppi)).clamp(0.75, 3.0);
                    self.outputs[i].scale = (linked * 10000.0).round() / 10000.0;
                }
            }
            let primary_idx = self.outputs.iter().position(|o| o.primary).unwrap_or(0);
            self.master_scale = self.outputs[primary_idx].scale;
        } else if self.outputs[output_idx].primary {
            self.master_scale = clamped;
        }

        self.recalculate_layout();
    }

    /// Intercambia el orden horizontal (Izquierda <-> Derecha) de los monitores
    pub fn swap_horizontal_order(&mut self) {
        if self.outputs.len() < 2 {
            return;
        }
        let mut order: Vec<usize> = (0..self.outputs.len()).collect();
        order.sort_by_key(|&idx| (self.outputs[idx].x, !self.outputs[idx].primary));
        order.reverse();

        let mut cursor_x = 0;
        for idx in order {
            self.outputs[idx].x = cursor_x;
            let (lw, _) = self.outputs[idx].logical_size();
            cursor_x += lw.max(100);
        }
    }

    /// Aplica un modo de alineación vertical entre las pantallas ("top", "center", "bottom", "desk")
    pub fn set_vertical_alignment(&mut self, mode: &str) {
        if self.outputs.is_empty() {
            return;
        }
        let max_h = self
            .outputs
            .iter()
            .map(|o| o.logical_size().1)
            .max()
            .unwrap_or(1080);

        match mode {
            "top" => {
                for o in &mut self.outputs {
                    o.y = 0;
                }
            }
            "center" => {
                for o in &mut self.outputs {
                    let (_, lh) = o.logical_size();
                    o.y = ((max_h - lh) / 2).max(0);
                }
            }
            "bottom" => {
                for o in &mut self.outputs {
                    let (_, lh) = o.logical_size();
                    o.y = (max_h - lh).max(0);
                }
            }
            "desk" => {
                for (i, o) in self.outputs.iter_mut().enumerate() {
                    if o.primary || i == 0 {
                        o.y = 519;
                    } else {
                        o.y = 0;
                    }
                }
            }
            _ => {}
        }
        self.recalculate_layout();
    }

    /// Desplaza verticalmente un monitor específico en `delta_y` píxeles lógicos
    pub fn adjust_vertical_offset(&mut self, output_idx: usize, delta_y: i32) {
        if output_idx >= self.outputs.len() {
            return;
        }
        self.outputs[output_idx].y += delta_y;
        if let Some(min_y) = self.outputs.iter().map(|o| o.y).min() {
            for o in &mut self.outputs {
                o.y = (o.y - min_y).clamp(0, 1200);
            }
        }
    }

    /// Define qué monitor es la pantalla principal
    pub fn set_primary_output(&mut self, primary_idx: usize) {
        if primary_idx >= self.outputs.len() {
            return;
        }
        for (i, o) in self.outputs.iter_mut().enumerate() {
            o.primary = i == primary_idx;
        }
        self.master_scale = self.outputs[primary_idx].scale;
        if self.linked_physical_1to1 {
            self.apply_1to1_link();
        } else {
            self.recalculate_layout();
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
                screen_corner_radius: 32.0,
                window_corner_radius: 32.0,
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
        assert!(cfg.icon_size > 0);
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

    #[test]
    fn test_displays_config_1to1_and_arrangement() {
        let mut cfg = DisplaysConfig::default();
        cfg.outputs[0].diagonal_inches = Some(27.0);
        cfg.outputs[1].diagonal_inches = Some(24.0);
        assert!(cfg.linked_physical_1to1);
        cfg.set_output_scale(0, 1.5);
        // Monitor 0 is 27" 4K (163.18 PPI) -> scale 1.50 (logical width 2560)
        // Monitor 1 is 24" 4K (183.58 PPI) -> scale 1.50 * (27/24) = 1.6875 (logical width 2276)
        assert!((cfg.outputs[0].scale - 1.5).abs() < 0.001);
        assert!((cfg.outputs[1].scale - 1.6875).abs() < 0.001);
        assert_eq!(cfg.outputs[0].x, 0);
        assert_eq!(cfg.outputs[1].x, 2560);

        // Swap horizontal order
        cfg.swap_horizontal_order();
        assert_eq!(cfg.outputs[1].x, 0);
        assert_eq!(cfg.outputs[0].x, 2276);

        // Unlink and set independent scale
        cfg.linked_physical_1to1 = false;
        cfg.set_output_scale(1, 2.0);
        assert!((cfg.outputs[1].scale - 2.0).abs() < 0.001);
        assert!((cfg.outputs[0].scale - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_input_config_latam_default() {
        let cfg = InputConfig::default();
        assert_eq!(cfg.keyboard_layout, "latam");
        assert_eq!(cfg.layout_short_badge(), "LAT");
    }

    #[test]
    fn test_displays_config_screen_corner_radius_default() {
        let cfg = DisplaysConfig::default();
        assert_eq!(cfg.screen_corner_radius, 32.0);
        assert_eq!(cfg.window_corner_radius, 32.0);
    }
}


