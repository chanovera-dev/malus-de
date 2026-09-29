//! Agente de Autenticación Polkit (`org.freedesktop.PolicyKit1.AuthenticationAgent`)
//!
//! Despliega un diálogo modal squircle acrílico con avatar de usuario, campo de contraseña
//! con animación de sacudida (*shake animation*) y soporte para sensor de huellas `fprintd`.

use tracing::info;

pub struct PolkitAgent;

impl PolkitAgent {
    pub fn new() -> Self {
        info!("Inicializando Polkit Authentication Agent para Malus DE...");
        Self
    }
}
