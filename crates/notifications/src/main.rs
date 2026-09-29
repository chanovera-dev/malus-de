use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_notifications");
    info!("Iniciando Malus DE Notifications & Widget Drawer...");
}
