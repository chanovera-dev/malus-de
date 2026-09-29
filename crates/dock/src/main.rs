use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_dock");
    info!("Iniciando Malus DE Dock (GTK4 / Layer Shell / Magnification)...");
}
