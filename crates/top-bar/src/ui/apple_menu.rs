use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Label, MenuButton, Orientation, Popover, Separator,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing::info;

pub fn create_apple_menu() -> MenuButton {
    let menu_btn = MenuButton::builder()
        .css_classes(["apple-logo-btn", "topbar-btn"])
        .tooltip_text("Menú Malus DE")
        .has_frame(false)
        .always_show_arrow(false)
        .build();

    let logo_label = Label::builder()
        .label("🍎")
        .css_classes(["apple-icon"])
        .build();
    menu_btn.set_child(Some(&logo_label));

    let popover = Popover::builder()
        .css_classes(["malus-popover-menu"])
        .has_arrow(false)
        .position(gtk4::PositionType::Bottom)
        .halign(gtk4::Align::Start)
        .build();
    popover.set_offset(0, 4);

    let content_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();

    // Elementos del Menú Apple
    let item_about = create_menu_item("Acerca de Malus DE", None, true);
    let popover_clone = popover.clone();
    item_about.connect_clicked(move |_| {
        popover_clone.popdown();
        show_about_dialog();
    });
    content_box.append(&item_about);

    content_box.append(&create_divider());

    let item_settings = create_menu_item("Preferencias del Sistema...", Some("⌘,"), false);
    let popover_clone = popover.clone();
    item_settings.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Abriendo Preferencias del Sistema...");
        if let Ok(mut child) = std::process::Command::new("gnome-control-center").spawn() {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
    });
    content_box.append(&item_settings);

    let item_monitor = create_menu_item("Forzar salida de aplicaciones...", Some("⌥⌘⎋"), false);
    let popover_clone = popover.clone();
    item_monitor.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Abriendo Monitor del Sistema...");
        let _ = std::process::Command::new("gnome-system-monitor").spawn();
    });
    content_box.append(&item_monitor);

    let item_reload = create_menu_item("Recargar Shell (Hot-Reload)", Some("⌥⌘R"), false);
    let popover_clone = popover.clone();
    item_reload.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Recargando componentes de interfaz de Malus DE (Hot-Reload)...");
        let bin_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("/usr/bin"));
        let ctl_path = if bin_dir.join("malus-de-ctl").exists() {
            bin_dir.join("malus-de-ctl")
        } else {
            bin_dir.join("malus-ctl")
        };
        let cmd = if ctl_path.exists() {
            ctl_path.to_str().unwrap().to_string()
        } else {
            "malus-de-ctl".to_string()
        };
        let _ = std::process::Command::new(&cmd).arg("reload").spawn();
    });
    content_box.append(&item_reload);

    let item_lock = create_menu_item("Bloquear Pantalla", Some("⌃⌘L"), false);
    let popover_clone = popover.clone();
    item_lock.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Bloqueando pantalla con malus-de-session-lock...");
        let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-1".to_string());
        let exe = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join("malus-de-session-lock")))
            .unwrap_or_else(|| std::path::PathBuf::from("malus-de-session-lock"));

        let _ = std::process::Command::new(&exe)
            .env("WAYLAND_DISPLAY", &wayland_display)
            .env("GDK_BACKEND", "wayland")
            .spawn();
    });
    content_box.append(&item_lock);

    content_box.append(&create_divider());

    let item_reboot = create_menu_item("Reiniciar...", None, false);
    let popover_clone = popover.clone();
    item_reboot.connect_clicked(move |_| {
        popover_clone.popdown();
        show_confirm_dialog(
            "¿Reiniciar el equipo?",
            "El sistema cerrará todas las aplicaciones y reiniciará el equipo. Cualquier cambio no guardado se perderá.",
            "Reiniciar",
            || {
                info!("Reiniciando el sistema...");
                let _ = std::process::Command::new("systemctl").arg("reboot").spawn();
            },
        );
    });
    content_box.append(&item_reboot);

    let item_shutdown = create_menu_item("Apagar el Equipo...", None, false);
    let popover_clone = popover.clone();
    item_shutdown.connect_clicked(move |_| {
        popover_clone.popdown();
        show_confirm_dialog(
            "¿Apagar el equipo?",
            "El sistema cerrará todas las aplicaciones y apagará el equipo de forma segura.",
            "Apagar",
            || {
                info!("Apagando el sistema...");
                let _ = std::process::Command::new("systemctl").arg("poweroff").spawn();
            },
        );
    });
    content_box.append(&item_shutdown);

    popover.set_child(Some(&content_box));
    menu_btn.set_popover(Some(&popover));

    menu_btn
}

fn create_menu_item(label_text: &str, shortcut: Option<&str>, is_bold: bool) -> Button {
    let btn = Button::builder()
        .css_classes(["popover-menu-item"])
        .has_frame(false)
        .build();

    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let label = Label::builder()
        .label(label_text)
        .xalign(0.0)
        .hexpand(true)
        .css_classes(["menu-row-label"])
        .build();

    if is_bold {
        label.add_css_class("menu-item-bold");
    }
    row.append(&label);

    if let Some(sc) = shortcut {
        let sc_label = Label::builder()
            .label(sc)
            .xalign(1.0)
            .halign(gtk4::Align::End)
            .css_classes(["menu-shortcut-hint"])
            .build();
        row.append(&sc_label);
    }

    btn.set_child(Some(&row));
    btn
}

fn create_divider() -> Separator {
    let sep = Separator::new(Orientation::Horizontal);
    sep.add_css_class("menu-separator");
    sep
}

fn show_about_dialog() {
    let about_window = adw::AboutDialog::builder()
        .application_name("Malus Desktop Environment")
        .version("0.1.0-alpha")
        .developer_name("Equipo Malus DE")
        .comments("Entorno de Escritorio Wayland Autónomo y Ultraligero escrito en Rust y Smithay. Fusión de diseño macOS y gestión de ventanas avanzada.")
        .website("https://github.com/cristian/malus-de")
        .issue_url("https://github.com/cristian/malus-de/issues")
        .license_type(gtk4::License::MitX11)
        .build();

    about_window.present(None as Option<&gtk4::Window>);
}

fn show_confirm_dialog(title: &str, body: &str, confirm_label: &str, on_confirm: impl FnOnce() + 'static) {
    let dialog = adw::MessageDialog::builder()
        .heading(title)
        .body(body)
        .build();
    dialog.add_response("cancel", "Cancelar");
    dialog.add_response("confirm", confirm_label);
    dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let on_confirm = std::cell::RefCell::new(Some(on_confirm));
    dialog.connect_response(None, move |d, resp| {
        if resp == "confirm" {
            if let Some(cb) = on_confirm.borrow_mut().take() {
                cb();
            }
        }
        d.close();
    });
    dialog.present();
}
