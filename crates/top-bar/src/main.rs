use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, CssProvider, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use libadwaita as adw;
use malus_de_common::logging::init_logging;
use tracing::info;

mod ui;

const STYLE_CSS: &str = r#"
window.malus-topbar {
    background-color: rgba(24, 24, 27, 0.78);
    color: #f4f4f5;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 13px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-weight: 500;
}

.topbar-content {
    min-height: 32px;
    padding-left: 10px;
    padding-right: 12px;
}

.topbar-btn {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 2px 7px;
    color: #f4f4f5;
    box-shadow: none;
}

.topbar-btn:hover {
    background-color: rgba(255, 255, 255, 0.10);
}

.topbar-btn:active {
    background-color: rgba(255, 255, 255, 0.18);
}

.apple-icon {
    font-size: 14px;
}

.active-app-name {
    font-weight: 700;
    color: #ffffff;
    padding-left: 2px;
    padding-right: 4px;
}

.global-menu-item {
    font-weight: 500;
    color: #e4e4e7;
}

.clock-label {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: #ffffff;
}

.active-pill {
    background-color: rgba(59, 130, 246, 0.35);
    border: 1px solid rgba(96, 165, 250, 0.5);
}

.malus-popover-menu {
    background-color: rgba(30, 30, 36, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.45);
    padding: 4px;
}

.popover-menu-item {
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 5px 12px;
    color: #e4e4e7;
    font-size: 13px;
    font-weight: 500;
}

.popover-menu-item:hover {
    background-color: #2563eb;
    color: #ffffff;
}

.menu-item-bold {
    font-weight: 700;
}
"#;

fn main() {
    init_logging("malus_de_top_bar");
    info!("Iniciando Malus DE Top Bar (GTK4 / Layer Shell)...");

    if let Err(err) = adw::init() {
        tracing::warn!("Aviso al inicializar libadwaita: {:?}", err);
    }

    let app = Application::builder()
        .application_id("org.malus_de.TopBar")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(build_ui);

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(STYLE_CSS);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Malus DE Top Bar")
        .default_height(32)
        .css_classes(["malus-topbar"])
        .build();

    if gtk4_layer_shell::is_supported() {
        info!("Protocolo Layer Shell soportado. Anclando Top Bar en capa Top...");
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace("malus-top-bar");
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.auto_exclusive_zone_enable();
    } else {
        info!("Layer Shell no detectado en este display; ejecutando ventana flotante.");
        window.set_decorated(false);
    }

    let root_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["topbar-content"])
        .spacing(8)
        .build();

    // 1. Zona Izquierda: Apple Menu + Global Menu
    let left_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();
    let (global_menu, app_title_label) = ui::global_menu::create_global_menu_widget();
    left_box.append(&ui::apple_menu::create_apple_menu());
    left_box.append(&global_menu);

    // Escuchar eventos IPC del compositor para actualizar el nombre de la aplicación activa
    let (tx, rx) = async_channel::unbounded::<malus_de_common::ipc::IpcMessage>();
    let title_label = app_title_label.clone();
    glib::spawn_future_local(async move {
        while let Ok(msg) = rx.recv().await {
            if let malus_de_common::ipc::IpcMessage::ActiveAppChanged { app_id, title } = msg {
                let display = format_app_name(app_id.as_deref(), title.as_deref());
                title_label.set_label(&display);
            }
        }
    });

    std::thread::spawn(move || {
        let sock_path = malus_de_common::ipc::ipc_socket_path();
        loop {
            if let Ok(stream) = std::os::unix::net::UnixStream::connect(&sock_path) {
                let reader = std::io::BufReader::new(stream);
                for line in std::io::BufRead::lines(reader) {
                    if let Ok(line) = line {
                        if let Ok(msg) = serde_json::from_str::<malus_de_common::ipc::IpcMessage>(&line) {
                            let _ = tx.send_blocking(msg);
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    // 2. Zona Central: Espaciador expansible
    let center_spacer = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    // 3. Zona Derecha: Auto-Tiling + Power Mode + Tray + Clock
    let right_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    right_box.append(&ui::auto_tiling::create_auto_tiling_widget());
    right_box.append(&ui::power_profile::create_power_profile_widget());
    right_box.append(&ui::status_tray::create_status_tray_widget());
    right_box.append(&ui::clock::create_clock_widget());

    root_box.append(&left_box);
    root_box.append(&center_spacer);
    root_box.append(&right_box);

    window.set_child(Some(&root_box));
    window.present();
}

fn format_app_name(app_id: Option<&str>, title: Option<&str>) -> String {
    if let Some(id) = app_id {
        let lower = id.to_lowercase();
        if lower.contains("nautilus") {
            return "Archivos".to_string();
        } else if lower.contains("calculator") {
            return "Calculadora".to_string();
        } else if lower.contains("ptyxis") {
            return "Terminal".to_string();
        } else if lower.contains("texteditor") || lower.contains("text-editor") {
            return "Editor de Texto".to_string();
        } else if lower.contains("control-center") || lower.contains("settings") {
            return "Configuración".to_string();
        }
    }

    if let Some(t) = title {
        let trimmed = t.trim();
        if !trimmed.is_empty() {
            if let Some((_, last)) = trimmed.rsplit_once(" — ") {
                return last.trim().to_string();
            }
            if let Some((_, last)) = trimmed.rsplit_once(" - ") {
                return last.trim().to_string();
            }
            return trimmed.to_string();
        }
    }

    if let Some(id) = app_id {
        if let Some(last) = id.rsplit('.').next() {
            if !last.is_empty() {
                return last.to_string();
            }
        }
    }

    "Malus DE".to_string()
}
