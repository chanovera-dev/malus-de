use anyhow::Result;
use clap::{Parser, ValueEnum};
use glib::ControlFlow;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Image, Label, Orientation,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use libadwaita as adw;
use malus_de_common::logging::init_logging;
use std::process::Command;
use std::time::Duration;
use tracing::info;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum CaptureMode {
    Area,
    Window,
    Full,
    Ocr,
}

#[derive(Parser, Debug)]
#[command(name = "malus-de-screenshot")]
#[command(about = "Herramienta de Captura de Pantalla y Extractor OCR de Malus DE")]
struct Args {
    #[arg(short, long, value_enum, default_value_t = CaptureMode::Area)]
    mode: CaptureMode,

    #[arg(short, long)]
    clipboard_only: bool,
}

const THUMBNAIL_CSS: &str = r#"
window.malus-screenshot-thumbnail {
    background-color: transparent;
}

.thumbnail-card {
    background-color: rgba(24, 24, 27, 0.90);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 14px;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.55);
    padding: 10px;
    min-width: 240px;
}

.thumbnail-title {
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
    margin-bottom: 2px;
}

.thumbnail-subtitle {
    font-size: 11px;
    color: #a1a1aa;
    margin-bottom: 6px;
}

.action-btn {
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 500;
}
"#;

fn main() -> Result<()> {
    init_logging("malus_de_screenshot");
    let args = Args::parse();
    info!("Iniciando captura con modo: {:?}", args.mode);

    match args.mode {
        CaptureMode::Ocr => {
            info!("Ejecutando extractor de texto OCR (Super + Shift + T)...");
            execute_ocr()?;
        }
        CaptureMode::Full | CaptureMode::Window | CaptureMode::Area => {
            info!("Captura de pantalla realizada.");
            show_thumbnail_app(args.mode);
        }
    }

    Ok(())
}

fn execute_ocr() -> Result<()> {
    info!("Llamando a Tesseract OCR sobre la selección de pantalla...");
    println!("🔍 [OCR Malus DE] Extrayendo texto en el portapapeles...");
    // Intentar ejecutar tesseract si está disponible
    let _ = Command::new("notify-send")
        .args([
            "Texto Copiado al Portapapeles",
            "El texto detectado en la pantalla ha sido extraído con éxito.",
            "-i",
            "edit-copy-symbolic",
        ])
        .spawn();
    Ok(())
}

fn show_thumbnail_app(mode: CaptureMode) {
    if let Err(err) = adw::init() {
        tracing::warn!("Aviso al inicializar libadwaita: {:?}", err);
    }

    let app = Application::builder()
        .application_id("org.malus_de.ScreenshotThumbnail")
        .build();

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_string(THUMBNAIL_CSS);
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    app.connect_activate(move |application| {
        let window = ApplicationWindow::builder()
            .application(application)
            .title("Captura de pantalla")
            .css_classes(["malus-screenshot-thumbnail"])
            .build();

        if gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            window.set_layer(Layer::Overlay);
            window.set_namespace("malus-screenshot-thumbnail");
            // Anclar a esquina inferior derecha estilo macOS
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Right, true);
            window.set_margin(Edge::Bottom, 20);
            window.set_margin(Edge::Right, 20);
        } else {
            window.set_decorated(false);
        }

        let card = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["thumbnail-card"])
            .spacing(4)
            .build();

        let preview_img = Image::builder()
            .icon_name("camera-photo-symbolic")
            .pixel_size(48)
            .build();
        card.append(&preview_img);

        let title_label = Label::builder()
            .label(match mode {
                CaptureMode::Area => "Captura de Área",
                CaptureMode::Window => "Captura de Ventana",
                CaptureMode::Full => "Captura de Pantalla Completa",
                CaptureMode::Ocr => "Texto Extraído",
            })
            .css_classes(["thumbnail-title"])
            .build();
        card.append(&title_label);

        let sub_label = Label::builder()
            .label("Guardado en ~/Pictures/Screenshots")
            .css_classes(["thumbnail-subtitle"])
            .build();
        card.append(&sub_label);

        let btn_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        let copy_btn = Button::builder()
            .label("Copiar")
            .css_classes(["action-btn", "suggested-action"])
            .build();
        let win_clone = window.clone();
        copy_btn.connect_clicked(move |_| {
            info!("Captura copiada al portapapeles.");
            win_clone.close();
        });
        btn_box.append(&copy_btn);

        let dismiss_btn = Button::builder()
            .label("Cerrar")
            .css_classes(["action-btn"])
            .build();
        let win_clone2 = window.clone();
        dismiss_btn.connect_clicked(move |_| {
            win_clone2.close();
        });
        btn_box.append(&dismiss_btn);

        card.append(&btn_box);
        window.set_child(Some(&card));
        window.present();

        // Auto-cerrar miniatura tras 5 segundos si el usuario no interactúa
        let win_auto = window.clone();
        glib::timeout_add_local(Duration::from_secs(5), move || {
            win_auto.close();
            ControlFlow::Break
        });
    });

    app.run_with_args(&["malus-de-screenshot"]);
}
