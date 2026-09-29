//! Demonio de Integración Móvil Malus Link (`malus-de-link`)
//!
//! Implementa el protocolo abierto de KDE Connect sobre TLS/UDP/TCP para
//! sincronización de portapapeles, transferencia de archivos (AirDrop Linux),
//! recepción de notificaciones móviles y respuesta de SMS.

use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_link");
    info!("Iniciando Malus Link Daemon (Protocolo KDE Connect / Enlace Móvil)...");

    // TODO: Descubrimiento UDP en puerto 1716 y canal TCP TLS con pares autorizados
    info!("Escuchando en red local por dispositivos móviles Android / iOS emparejados...");
}
