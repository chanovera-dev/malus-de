use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_session_lock");
    info!("Iniciando Malus DE Session Lock (ext-session-lock-v1 / PAM / fprintd)...");
}
