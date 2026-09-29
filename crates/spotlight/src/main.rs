use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_spotlight");
    info!("Iniciando Malus DE Spotlight (GTK4 / Tracker Miner FS / Quick Look)...");
}
