use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_top_bar");
    info!("Iniciando Malus DE Top Bar (GTK4 / Layer Shell)...");
}
