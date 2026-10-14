pub mod backend;
pub mod grabs;
pub mod state;

use malus_de_common::logging::init_logging;
use tracing::info;

fn main() -> anyhow::Result<()> {
    init_logging("malus_de_compositor");
    info!("Iniciando Malus DE Compositor (Smithay Wayland)...");

    if let Err(e) = malus_de_common::config::setup_isolated_session_environment() {
        tracing::warn!("Aviso al inicializar entorno aislado de configuración: {:?}", e);
    }

    let args: Vec<String> = std::env::args().collect();
    let backend = args.iter().position(|r| r == "--backend")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("winit");

    info!("Modo de backend seleccionado: {}", backend);

    match backend {
        "winit" => {
            info!("Iniciando backend Winit para pruebas en ventana anidada...");
            crate::backend::winit::run_winit()?;
        }
        "drm" => {
            info!("Iniciando backend DRM/KMS nativo con soporte libinput...");
            // TODO: Inicialización de sesión directa TTY en Fase 2
        }
        other => {
            eprintln!("Backend no reconocido: {other}. Usa 'winit' o 'drm'.");
            std::process::exit(1);
        }
    }

    Ok(())
}
