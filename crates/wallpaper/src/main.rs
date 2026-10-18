use anyhow::Result;
use clap::Parser;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, Picture};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use malus_de_common::logging::init_logging;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "malus-de-wallpaper")]
#[command(about = "Demonio de Fondo de Pantalla de Malus DE (Layer Shell Background)")]
struct Args {
    /// Ruta a la imagen de fondo de pantalla
    #[arg(short, long)]
    image: Option<PathBuf>,
}

const DEFAULT_GRADIENT_CSS: &str = r#"
window.malus-wallpaper-window,
window.malus-wallpaper-window.background,
window.malus-wallpaper-window decoration {
    margin: 0;
    padding: 0;
    border: none;
    box-shadow: none;
    background: radial-gradient(circle at 35% 25%, #2e1065 0%, #172554 35%, #0f172a 70%, #0d1526 100%);
}

window.malus-wallpaper-window picture {
    margin: 0;
    padding: 0;
    border: none;
}
"#;

fn create_wallpaper_window(
    application: &Application,
    monitor: Option<&gtk4::gdk::Monitor>,
    resolved_path: &Option<PathBuf>,
) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("Fondo de Pantalla Malus DE")
        .decorated(false)
        .css_classes(["malus-wallpaper-window"])
        .build();

    window.set_decorated(false);

    if gtk4_layer_shell::is_supported() {
        info!("Anclando Fondo de Pantalla en capa Background a pantalla completa...");
        window.init_layer_shell();
        if let Some(mon) = monitor {
            window.set_monitor(mon);
        }
        window.set_layer(Layer::Background);
        window.set_namespace("malus-wallpaper");
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_exclusive_zone(-1);
    } else {
        window.maximize();
    }

    if let Some(ref img_path) = resolved_path {
        if img_path.exists() {
            info!("Cargando imagen de fondo: {:?}", img_path);
            let picture = Picture::for_filename(img_path);
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.set_hexpand(true);
            picture.set_vexpand(true);
            picture.set_halign(gtk4::Align::Fill);
            picture.set_valign(gtk4::Align::Fill);
            window.set_child(Some(&picture));
        }
    }

    window.present();
}

fn main() -> Result<()> {
    init_logging("malus_de_wallpaper");
    let args = Args::parse();
    info!("Iniciando Malus DE Wallpaper Daemon...");

    // Esperar de forma activa a que el compositor esté listo y aceptando conexiones
    malus_de_common::wayland::wait_for_wayland(10);

    let app = Application::builder()
        .application_id("org.malus_de.Wallpaper")
        .build();

    let image_path = args.image;

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_string(DEFAULT_GRADIENT_CSS);
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    app.connect_activate(move |application| {
        let resolved_path = image_path.clone().or_else(|| {
            let config_dir = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    std::env::var("HOME")
                        .map(|h| PathBuf::from(h).join(".config/malus-de"))
                        .unwrap_or_else(|_| PathBuf::from("/etc/malus-de"))
                });
            let candidates = [
                config_dir.join("wallpaper.png"),
                config_dir.join("wallpaper.jpg"),
                config_dir.join("wallpaper.jpeg"),
                config_dir.join("wallpaper.webp"),
                config_dir.join("wallpaper.svg"),
            ];
            candidates.into_iter().find(|p| p.exists())
        });

        let mut created = 0;
        if let Some(display) = gtk4::gdk::Display::default() {
            let monitors = display.monitors();
            for i in 0..monitors.n_items() {
                if let Some(obj) = monitors.item(i) {
                    if let Ok(monitor) = obj.downcast::<gtk4::gdk::Monitor>() {
                        create_wallpaper_window(application, Some(&monitor), &resolved_path);
                        created += 1;
                    }
                }
            }
        }

        if created == 0 {
            create_wallpaper_window(application, None, &resolved_path);
        }
    });

    app.run_with_args(&["malus-de-wallpaper"]);
    Ok(())
}
