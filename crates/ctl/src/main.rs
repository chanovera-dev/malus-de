use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_ctl");
    info!("Malus DE Control CLI");
}
