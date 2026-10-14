use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider,
    Image, Label, Orientation, Separator,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use libadwaita as adw;
use malus_de_common::logging::init_logging;
use std::process::Command;
use tracing::info;

const DOCK_CSS: &str = r#"
window.malus-dock-window {
    background-color: transparent;
}

.dock-container {
    background-color: rgba(24, 24, 27, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 22px;
    padding: 6px 10px;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.50);
}

.dock-item-box {
    padding: 0px 4px;
}

.dock-item-btn {
    background: transparent;
    border: none;
    border-radius: 14px;
    padding: 4px;
    transition: all 180ms cubic-bezier(0.2, 0.9, 0.3, 1.0);
}

.dock-item-btn:hover {
    background-color: rgba(255, 255, 255, 0.14);
}

.dock-item-btn:active {
    background-color: rgba(255, 255, 255, 0.22);
}

.dock-indicator {
    font-size: 8px;
    color: rgba(255, 255, 255, 0.85);
    margin-top: -2px;
    margin-bottom: -2px;
}

.dock-separator {
    background-color: rgba(255, 255, 255, 0.18);
    min-width: 1px;
    margin: 8px 6px;
}

.malus-popover-menu {
    background-color: rgba(30, 30, 36, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.45);
    padding: 4px;
}

.popover-menu-item {
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 5px 14px;
    color: #e4e4e7;
    font-size: 13px;
    font-weight: 500;
}

.popover-menu-item:hover {
    background-color: #2563eb;
    color: #ffffff;
}
"#;

struct DockAppDef {
    name: &'static str,
    icon: &'static str,
    command: &'static str,
    matchers: &'static [&'static str],
}

const PINNED_APPS: &[DockAppDef] = &[
    DockAppDef {
        name: "Archivos",
        icon: "system-file-manager",
        command: "dbus-run-session nautilus --new-window",
        matchers: &["nautilus", "archivos", "org.gnome.nautilus"],
    },
    DockAppDef {
        name: "Terminal",
        icon: "utilities-terminal",
        command: "ptyxis -s",
        matchers: &["ptyxis", "terminal", "org.gnome.ptyxis"],
    },
    DockAppDef {
        name: "Editor de Texto",
        icon: "accessories-text-editor",
        command: "gnome-text-editor -s",
        matchers: &["texteditor", "text-editor", "gedit", "org.gnome.texteditor"],
    },
    DockAppDef {
        name: "Calculadora",
        icon: "org.gnome.Calculator",
        command: "gnome-calculator",
        matchers: &["calculator", "calculadora", "org.gnome.calculator"],
    },
    DockAppDef {
        name: "Configuración",
        icon: "preferences-system",
        command: "gnome-control-center",
        matchers: &["control-center", "settings", "configuración", "configuracion", "org.gnome.settings"],
    },
];

fn main() {
    init_logging("malus_de_dock");
    info!("Iniciando Malus DE Floating Dock (GTK4 / Layer Shell)...");

    if let Err(err) = adw::init() {
        tracing::warn!("Aviso al inicializar libadwaita: {:?}", err);
    }

    let app = Application::builder()
        .application_id("org.malus_de.Dock")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(build_dock_ui);

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(DOCK_CSS);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_dock_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Malus DE Dock")
        .css_classes(["malus-dock-window"])
        .build();

    if gtk4_layer_shell::is_supported() {
        info!("Layer Shell soportado. Configurando Dock flotante centrado inferior...");
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace("malus-dock");
        // Anclar únicamente al fondo (Bottom), centrado horizontal automático
        window.set_anchor(Edge::Bottom, true);
        window.set_margin(Edge::Bottom, 10);
        window.auto_exclusive_zone_enable();
    } else {
        info!("Layer Shell no detectado; ejecutando Dock en ventana flotante de desarrollo.");
        window.set_decorated(false);
    }

    let dock_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["dock-container"])
        .spacing(4)
        .build();

    let mut dock_indicators = Vec::new();

    // 1. Aplicaciones fijadas (Pinned Apps)
    for app_def in PINNED_APPS {
        let (widget, ind) = create_dock_item(app_def, true);
        dock_bar.append(&widget);
        if let Some(lbl) = ind {
            dock_indicators.push((app_def.matchers, lbl));
        }
    }

    // 2. Separador de cristal
    let separator = Separator::new(Orientation::Vertical);
    separator.add_css_class("dock-separator");
    dock_bar.append(&separator);

    // 3. Papelera (Trash)
    let trash_def = DockAppDef {
        name: "Papelera",
        icon: "user-trash-symbolic",
        command: "nautilus trash://",
        matchers: &[],
    };
    let (trash_widget, _) = create_dock_item(&trash_def, false);
    dock_bar.append(&trash_widget);

    // Escuchar eventos IPC del compositor para actualizar los indicadores de ejecución (●)
    let (tx, rx) = async_channel::unbounded::<malus_de_common::ipc::IpcMessage>();
    glib::spawn_future_local(async move {
        while let Ok(msg) = rx.recv().await {
            if let malus_de_common::ipc::IpcMessage::RunningAppsChanged { running_apps } = msg {
                for (matchers, indicator) in &dock_indicators {
                    let is_running = running_apps.iter().any(|running| {
                        let r_lower = running.to_lowercase();
                        matchers.iter().any(|m| r_lower.contains(m))
                    });
                    indicator.set_opacity(if is_running { 1.0 } else { 0.0 });
                }
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

    window.set_child(Some(&dock_bar));
    window.present();
}

fn create_dock_item(app_def: &DockAppDef, has_indicator: bool) -> (GtkBox, Option<Label>) {
    let item_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["dock-item-box"])
        .spacing(1)
        .build();

    let btn = Button::builder()
        .css_classes(["dock-item-btn"])
        .tooltip_text(app_def.name)
        .focusable(false)
        .build();

    let icon = Image::builder()
        .icon_name(app_def.icon)
        .pixel_size(48)
        .build();
    btn.set_child(Some(&icon));

    let cmd = app_def.command;
    let name = app_def.name;
    btn.connect_clicked(move |_| {
        info!("Lanzando aplicación desde Dock: {} (`{}`)", name, cmd);
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if let Some((exe, args)) = parts.split_first() {
            let mut command = Command::new(exe);
            command.args(args);
            if let Ok(wayland_disp) = std::env::var("WAYLAND_DISPLAY") {
                command.env("WAYLAND_DISPLAY", wayland_disp);
            }
            command.env("GDK_BACKEND", "wayland");
            command.env("GSK_RENDERER", "cairo");
            if let Ok(xdg_rt) = std::env::var("XDG_RUNTIME_DIR") {
                command.env("XDG_RUNTIME_DIR", xdg_rt);
            }
            if let Ok(cfg_home) = std::env::var("XDG_CONFIG_HOME") {
                command.env("XDG_CONFIG_HOME", cfg_home);
            } else if let Ok(home) = std::env::var("HOME") {
                command.env("XDG_CONFIG_HOME", format!("{}/.config/malus-de", home));
            }
            command.env("GSETTINGS_BACKEND", "keyfile");

            match command.spawn() {
                Ok(mut child) => {
                    let pid = child.id();
                    info!("🚀 Proceso iniciado para {}: PID={}", name, pid);
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                }
                Err(err) => {
                    tracing::error!("❌ Error al lanzar aplicación {} desde Dock: {:?}", name, err);
                }
            }
        }
    });

    item_container.append(&btn);

    // Indicador de app en ejecución (Punto iluminado `●`)
    let indicator = if has_indicator {
        let ind = Label::builder()
            .label("●")
            .css_classes(["dock-indicator"])
            .opacity(0.0) // inicialmente apagado
            .build();
        item_container.append(&ind);
        Some(ind)
    } else {
        let spacer = Label::builder()
            .label("●")
            .css_classes(["dock-indicator"])
            .opacity(0.0)
            .build();
        item_container.append(&spacer);
        None
    };

    (item_container, indicator)
}

