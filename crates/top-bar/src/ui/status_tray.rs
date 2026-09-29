use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Orientation};
use tracing::info;

pub fn create_status_tray_widget() -> GtkBox {
    let tray = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    // Malus Link (KDE Connect / Integración móvil)
    let link_btn = Button::builder()
        .css_classes(["link-pill", "topbar-btn"])
        .tooltip_text("Malus Link: Dispositivo vinculado (88% batería)")
        .build();
    let link_icon = Image::from_icon_name("phone-symbolic");
    link_btn.set_child(Some(&link_icon));
    link_btn.connect_clicked(|_| {
        info!("Abriendo panel de Malus Link...");
    });
    tray.append(&link_btn);

    // WiFi / Red
    let net_btn = Button::builder()
        .css_classes(["network-pill", "topbar-btn"])
        .tooltip_text("Red conectada")
        .build();
    let net_icon = Image::from_icon_name("network-wireless-symbolic");
    net_btn.set_child(Some(&net_icon));
    tray.append(&net_btn);

    // Volumen
    let vol_btn = Button::builder()
        .css_classes(["volume-pill", "topbar-btn"])
        .tooltip_text("Volumen: 75%")
        .build();
    let vol_icon = Image::from_icon_name("audio-volume-high-symbolic");
    vol_btn.set_child(Some(&vol_icon));
    tray.append(&vol_btn);

    // Spotlight Search (Lupa macOS)
    let spotlight_btn = Button::builder()
        .css_classes(["spotlight-pill", "topbar-btn"])
        .tooltip_text("Búsqueda Spotlight (Alt + Espacio / Super + K)")
        .build();
    let spotlight_icon = Image::from_icon_name("system-search-symbolic");
    spotlight_btn.set_child(Some(&spotlight_icon));
    spotlight_btn.connect_clicked(|_| {
        let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-1".to_string());
        info!("Alternando Spotlight desde barra superior (WAYLAND_DISPLAY={})...", wayland_display);

        let output = std::process::Command::new("pkill")
            .args(["-f", "malus-de-spotlight"])
            .status();

        if let Ok(status) = output {
            if status.success() {
                info!("Spotlight existente cerrado con pkill.");
                return;
            }
        }

        let exe = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join("malus-de-spotlight")))
            .unwrap_or_else(|| std::path::PathBuf::from("malus-de-spotlight"));

        info!("Lanzando Spotlight: {:?}", exe);
        match std::process::Command::new(&exe)
            .env("WAYLAND_DISPLAY", &wayland_display)
            .env("GDK_BACKEND", "wayland")
            .env("GSK_RENDERER", "cairo")
            .spawn()
        {
            Ok(child) => info!("Spotlight lanzado con éxito, PID={}", child.id()),
            Err(e) => tracing::error!("Error al lanzar Spotlight: {:?}", e),
        }
    });
    tray.append(&spotlight_btn);

    // Control Center (Botón de dos interruptores tipo macOS)
    let cc_btn = Button::builder()
        .css_classes(["control-center-toggle", "topbar-btn"])
        .tooltip_text("Centro de Control")
        .build();
    let cc_icon = Image::from_icon_name("preferences-system-symbolic");
    cc_btn.set_child(Some(&cc_icon));
    cc_btn.connect_clicked(|_| {
        info!("Alternando Centro de Control Malus DE...");
    });
    tray.append(&cc_btn);

    tray
}
