use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Paquete base del protocolo KDE Connect (transmitido como JSON delimitado por salto de línea)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPacket {
    pub id: i64,
    #[serde(rename = "type")]
    pub packet_type: String,
    pub body: Value,
}

#[allow(dead_code)]
impl NetworkPacket {
    pub fn new_identity(device_id: &str, device_name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            id: now,
            packet_type: "kdeconnect.identity".to_string(),
            body: serde_json::json!({
                "deviceId": device_id,
                "deviceName": device_name,
                "protocolVersion": 7,
                "deviceType": "desktop",
                "incomingCapabilities": [
                    "kdeconnect.clipboard",
                    "kdeconnect.notification",
                    "kdeconnect.battery",
                    "kdeconnect.findmyphone",
                    "kdeconnect.share.request"
                ],
                "outgoingCapabilities": [
                    "kdeconnect.clipboard",
                    "kdeconnect.notification",
                    "kdeconnect.battery",
                    "kdeconnect.findmyphone",
                    "kdeconnect.share.request"
                ]
            }),
        }
    }

    pub fn new_ping() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            id: now,
            packet_type: "kdeconnect.ping".to_string(),
            body: serde_json::json!({}),
        }
    }

    pub fn new_ring_phone() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            id: now,
            packet_type: "kdeconnect.findmyphone.request".to_string(),
            body: serde_json::json!({}),
        }
    }

    pub fn new_clipboard(content: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            id: now,
            packet_type: "kdeconnect.clipboard".to_string(),
            body: serde_json::json!({
                "content": content
            }),
        }
    }
}
