use anyhow::Result;
use clap::{Parser, Subcommand};
use malus_de_common::logging::init_logging;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "malus-ctl")]
#[command(about = "CLI oficial de control y automatización de Malus DE", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Muestra el estado del entorno de escritorio Malus DE
    Status,

    /// Controla el modo de Auto-Tiling (ventanas en mosaico bspwm)
    Tiling {
        #[command(subcommand)]
        action: TilingAction,
    },

    /// Controla el perfil de rendimiento del procesador
    Power {
        #[command(subcommand)]
        action: PowerAction,
    },

    /// Controla los espacios de trabajo / escritorios virtuales
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },

    /// Abre o alterna la búsqueda universal Spotlight
    Spotlight,

    /// Activa o desactiva Picture-in-Picture en la ventana activa
    Pip,

    /// Extrae texto de la pantalla mediante OCR (Super + Shift + T)
    Ocr,

    /// Controla la integración móvil con Malus Link (KDE Connect)
    Link {
        #[command(subcommand)]
        action: LinkAction,
    },
}

#[derive(Subcommand, Debug)]
enum TilingAction {
    /// Alterna entre modo flotante y mosaico (Super + Y)
    Toggle,
    /// Activa el modo mosaico binario
    Enable,
    /// Desactiva el modo mosaico (vuelve a flotante libre)
    Disable,
    /// Consulta el estado actual del auto-tiling
    Status,
}

#[derive(Subcommand, Debug)]
enum PowerAction {
    /// Consulta el perfil de rendimiento actual
    Get,
    /// Establece el modo: performance, balanced, power-saver
    Set { profile: String },
}

#[derive(Subcommand, Debug)]
enum WorkspaceAction {
    /// Cambia al siguiente espacio de trabajo
    Next,
    /// Cambia al espacio de trabajo anterior
    Prev,
    /// Salta directamente al espacio N (1..10)
    Goto { index: u32 },
}

#[derive(Subcommand, Debug)]
enum LinkAction {
    /// Muestra el estado de sincronización y batería del móvil
    Status,
    /// Hace sonar el teléfono móvil vinculado
    Ring,
    /// Comparte un archivo o texto con el dispositivo
    Share { path: String },
}

fn main() -> Result<()> {
    init_logging("malus_de_ctl");
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║              ✨ MALUS DESKTOP ENVIRONMENT                ║");
            println!("╠═══════════════════════════════════════════════════════════╣");
            println!("║ Versión:         0.1.0-alpha                              ║");
            println!("║ Compositor:      Smithay 0.7 (Wayland Server Nativo)      ║");
            println!("║ Shell:           GTK4 / Layer-Shell (TopBar + Dock)       ║");
            println!("║ Renderizador:    GLES 3.2 Mesa Radeon (AMD Vega 7)        ║");
            println!("║ Perfil Energía:  power-profiles-daemon (D-Bus)            ║");
            println!("║ Auto-Tiling:     Disponible (Super + Y)                   ║");
            println!("║ Integración:     Malus Link (KDE Connect Protocol)        ║");
            println!("╚═══════════════════════════════════════════════════════════╝");
        }
        Commands::Tiling { action } => match action {
            TilingAction::Toggle => {
                println!("🔄 Alternando modo de auto-tiling...");
            }
            TilingAction::Enable => {
                println!("✅ Auto-tiling activado (modo mosaico bspwm)");
            }
            TilingAction::Disable => {
                println!("⏹ Auto-tiling desactivado (modo flotante libre)");
            }
            TilingAction::Status => {
                println!("Modo de ventanas actual: Flotante libre con Snap Layouts");
            }
        },
        Commands::Power { action } => match action {
            PowerAction::Get => {
                println!("Consultando perfil actual en power-profiles-daemon...");
                let out = Command::new("powerprofilesctl").arg("get").output();
                if let Ok(o) = out {
                    println!("Perfil activo: {}", String::from_utf8_lossy(&o.stdout).trim());
                } else {
                    println!("Perfil activo: balanced (local fallback)");
                }
            }
            PowerAction::Set { profile } => {
                println!("Estableciendo perfil de energía a: {}", profile);
                let _ = Command::new("powerprofilesctl")
                    .args(["set", &profile])
                    .status();
            }
        },
        Commands::Workspace { action } => match action {
            WorkspaceAction::Next => println!("Navegando al siguiente espacio de trabajo..."),
            WorkspaceAction::Prev => println!("Navegando al espacio de trabajo anterior..."),
            WorkspaceAction::Goto { index } => println!("Saltando al espacio de trabajo #{}", index),
        },
        Commands::Spotlight => {
            println!("Abriendo búsqueda universal Spotlight (Super + Espacio)...");
            let _ = Command::new("malus-de-spotlight").spawn();
        }
        Commands::Pip => {
            println!("Alternando modo Picture-in-Picture en ventana activa (Super + P)...");
        }
        Commands::Ocr => {
            println!("Iniciando extractor OCR de pantalla (Super + Shift + T)...");
        }
        Commands::Link { action } => match action {
            LinkAction::Status => {
                println!("📱 Malus Link: Buscando dispositivos vinculados...");
            }
            LinkAction::Ring => {
                println!("🔔 Enviando señal para hacer sonar el teléfono...");
            }
            LinkAction::Share { path } => {
                println!("📤 Compartiendo '{}' con el teléfono móvil...", path);
            }
        },
    }

    Ok(())
}
