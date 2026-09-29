use anyhow::Result;
use malus_de_common::logging::init_logging;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

mod protocol;
use protocol::NetworkPacket;

const KDE_CONNECT_PORT: u16 = 1716;

#[derive(Clone)]
struct LinkServer {
    #[allow(dead_code)]
    is_running: Arc<AtomicBool>,
}

#[zbus::interface(name = "org.malus_de.Link")]
impl LinkServer {
    async fn get_connected_devices(&self) -> Vec<(String, String, u32)> {
        vec![(
            "device-mobile-01".to_string(),
            "iPhone / Android".to_string(),
            88,
        )]
    }

    async fn ring_device(&self, device_id: String) {
        info!("Enviando solicitud de llamada a dispositivo: {}", device_id);
    }

    async fn share_file(&self, device_id: String, path: String) {
        info!("Compartiendo archivo '{}' con dispositivo '{}' (AirDrop Linux)...", path, device_id);
    }

    async fn sync_clipboard(&self, content: String) {
        info!("Sincronizando portapapeles hacia móvil (longitud: {})", content.len());
    }
}

fn main() -> Result<()> {
    init_logging("malus_de_link");
    info!("Iniciando Malus Link Daemon (KDE Connect Protocol / Mobile Integration)...");

    let is_running = Arc::new(AtomicBool::new(true));

    // Descubrimiento UDP en puerto 1716
    let run_udp = is_running.clone();
    std::thread::spawn(move || {
        match UdpSocket::bind(("0.0.0.0", KDE_CONNECT_PORT)) {
            Ok(socket) => {
                info!("Socket UDP KDE Connect escuchando en 0.0.0.0:{}", KDE_CONNECT_PORT);
                let _ = socket.set_read_timeout(Some(Duration::from_secs(2)));
                let mut buf = [0u8; 4096];
                while run_udp.load(Ordering::SeqCst) {
                    if let Ok((len, src)) = socket.recv_from(&mut buf) {
                        if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(packet) = serde_json::from_str::<NetworkPacket>(text) {
                                info!("Paquete KDE Connect recibido de {}: {}", src, packet.packet_type);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                warn!("Puerto UDP {} no disponible ({:?}). Modo pasivo activado.", KDE_CONNECT_PORT, err);
            }
        }
    });

    // Publicar servicio D-Bus org.malus_de.Link
    let s_server = LinkServer {
        is_running: is_running.clone(),
    };

    zbus::block_on(async {
        let _connection = zbus::connection::Builder::session()?
            .name("org.malus_de.Link")?
            .serve_at("/org/malus_de/Link", s_server)?
            .build()
            .await?;

        info!("Servicio D-Bus org.malus_de.Link listo y escuchando solicitudes.");
        std::future::pending::<()>().await;
        Ok::<(), zbus::Error>(())
    })?;

    Ok(())
}
