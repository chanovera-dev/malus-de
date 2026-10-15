use anyhow::Result;
use malus_de_common::logging::init_logging;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};
use zbus::interface;

mod protocol;
use protocol::NetworkPacket;

const KDE_CONNECT_PORT: u16 = 1716;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirDropMode {
    Off,
    ContactsOnly,
    Everyone,
}

impl AirDropMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AirDropMode::Off => "off",
            AirDropMode::ContactsOnly => "contacts",
            AirDropMode::Everyone => "everyone",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "everyone" | "todos" => AirDropMode::Everyone,
            "contacts" | "contactos" => AirDropMode::ContactsOnly,
            _ => AirDropMode::Off,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AirDropMode::Off => "Desactivado",
            AirDropMode::ContactsOnly => "Solo contactos",
            AirDropMode::Everyone => "Todos (10 min)",
        }
    }
}

pub struct LinkState {
    pub airdrop_mode: AirDropMode,
    pub everyone_until: Option<Instant>,
}

#[derive(Clone)]
pub struct LinkServer {
    pub state: Arc<Mutex<LinkState>>,
    #[allow(dead_code)]
    pub is_running: Arc<AtomicBool>,
}

#[interface(name = "org.malus_de.Link")]
impl LinkServer {
    async fn get_airdrop_mode(&self) -> String {
        let mut st = self.state.lock().unwrap();
        // Verificar si la visibilidad "Todos" de 10 minutos expiró
        if st.airdrop_mode == AirDropMode::Everyone {
            if let Some(until) = st.everyone_until {
                if Instant::now() >= until {
                    info!("⏳ Visibilidad temporal 'Todos' expirada. Revertiendo a 'Solo contactos'.");
                    st.airdrop_mode = AirDropMode::ContactsOnly;
                    st.everyone_until = None;
                }
            }
        }
        st.airdrop_mode.as_str().to_string()
    }

    async fn set_airdrop_mode(&self, mode: String) -> bool {
        let mut st = self.state.lock().unwrap();
        let parsed = AirDropMode::from_str(&mode);
        st.airdrop_mode = parsed;
        if parsed == AirDropMode::Everyone {
            st.everyone_until = Some(Instant::now() + Duration::from_secs(600)); // 10 minutos
            info!("📡 AirDrop configurado a: Todos (activo durante 10 minutos)");
        } else {
            st.everyone_until = None;
            info!("📡 AirDrop configurado a: {:?}", parsed);
        }
        true
    }

    async fn get_airdrop_remaining_seconds(&self) -> u32 {
        let st = self.state.lock().unwrap();
        if st.airdrop_mode == AirDropMode::Everyone {
            if let Some(until) = st.everyone_until {
                let now = Instant::now();
                if until > now {
                    return until.duration_since(now).as_secs() as u32;
                }
            }
        }
        0
    }

    async fn simulate_airdrop_receive(&self, sender_name: String, file_name: String) -> bool {
        info!("📲 [AirDrop] Recibiendo archivo simulado '{}' desde '{}'...", file_name, sender_name);
        let summary = format!("AirDrop de {}", sender_name);
        let body = format!("Compartió el archivo '{}'\nGuardado en ~/Descargas", file_name);

        let _ = std::process::Command::new("notify-send")
            .args([
                "--app-name=AirDrop",
                "--icon=network-wireless-hotspot-symbolic",
                &summary,
                &body,
            ])
            .spawn();

        true
    }

    async fn get_connected_devices(&self) -> Vec<(String, String, u32)> {
        vec![(
            "device-apple-01".to_string(),
            "iPhone de Cristian".to_string(),
            92,
        )]
    }

    async fn ring_device(&self, device_id: String) {
        info!("Enviando solicitud de llamada a dispositivo: {}", device_id);
    }

    async fn share_file(&self, device_id: String, path: String) {
        info!("Compartiendo archivo '{}' con dispositivo '{}' vía AirDrop/AWDL...", path, device_id);
    }

    async fn sync_clipboard(&self, content: String) {
        info!("Sincronizando portapapeles hacia dispositivo móvil (longitud: {})", content.len());
    }
}

fn main() -> Result<()> {
    init_logging("malus_de_link");
    info!("Iniciando Malus Link Daemon (AirDrop Engine & Mobile Integration)...");

    let is_running = Arc::new(AtomicBool::new(true));
    let state = Arc::new(Mutex::new(LinkState {
        airdrop_mode: AirDropMode::Everyone,
        everyone_until: Some(Instant::now() + Duration::from_secs(600)),
    }));

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
                                info!("Paquete recibido de {}: {}", src, packet.packet_type);
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
        state: state.clone(),
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
