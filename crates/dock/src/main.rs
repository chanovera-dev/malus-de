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
    is_running: bool,
}

const PINNED_APPS: &[DockAppDef] = &[
    DockAppDef {
        name: "Archivos",
        icon: "system-file-manager",
        command: "nautilus",
        is_running: false,
    },
    DockAppDef {
        name: "Terminal",
        icon: "utilities-terminal",
        command: "ptyxis",
        is_running: false,
    },
    DockAppDef {
        name: "Editor de Texto",
        icon: "accessories-text-editor",
        command: "gnome-text-editor",
        is_running: false,
    },
    DockAppDef {
        name: "Calculadora",
        icon: "org.gnome.Calculator",
        command: "gnome-calculator",
        is_running: false,
    },
    DockAppDef {
        name: "Configuración",
        icon: "preferences-system",
        command: "gnome-control-center",
        is_running: false,
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

    // 1. Aplicaciones fijadas (Pinned Apps)
    for app_def in PINNED_APPS {
        dock_bar.append(&create_dock_item(app_def));
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
        is_running: false,
    };
    dock_bar.append(&create_dock_item(&trash_def));

    window.set_child(Some(&dock_bar));
    window.present();
}

fn create_dock_item(app_def: &DockAppDef) -> GtkBox {
    let item_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["dock-item-box"])
        .spacing(1)
        .build();

    let btn = Button::builder()
        .css_classes(["dock-item-btn"])
        .tooltip_text(app_def.name)
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
            let _ = command.spawn();
        }
    });

    item_container.append(&btn);

    // Indicador de app en ejecución (Punto iluminado `•`)
    if app_def.is_running {
        let indicator = Label::builder()
            .label("●")
            .css_classes(["dock-indicator"])
            .build();
        item_container.append(&indicator);
    } else {
        // Espaciador transparente invisible para mantener alineación exacta
        let spacer = Label::builder()
            .label(" ")
            .css_classes(["dock-indicator"])
            .opacity(0.0)
            .build();
        item_container.append(&spacer);
    }

    item_container
}
