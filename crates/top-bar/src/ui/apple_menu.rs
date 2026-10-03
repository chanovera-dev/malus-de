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
        .build();

    let logo_label = Label::builder()
        .label("🍎")
        .css_classes(["apple-icon"])
        .build();
    menu_btn.set_child(Some(&logo_label));

    let popover = Popover::builder()
        .css_classes(["malus-popover-menu"])
        .has_arrow(false)
        .build();

    let content_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    // Elementos del Menú
    let item_about = create_menu_item("Acerca de Malus DE", false);
    let popover_clone = popover.clone();
    item_about.connect_clicked(move |_| {
        popover_clone.popdown();
        show_about_dialog();
    });
    content_box.append(&item_about);

    content_box.append(&Separator::new(Orientation::Horizontal));

    let item_settings = create_menu_item("Preferencias del Sistema...", false);
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

    let item_lock = create_menu_item("Bloquear Pantalla        ⌃⌘L", false);
    let popover_clone = popover.clone();
    item_lock.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Bloqueando pantalla...");
    });
    content_box.append(&item_lock);

    content_box.append(&Separator::new(Orientation::Horizontal));

    let item_reboot = create_menu_item("Reiniciar...", false);
    let popover_clone = popover.clone();
    item_reboot.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Solicitando reinicio del sistema...");
    });
    content_box.append(&item_reboot);

    let item_shutdown = create_menu_item("Apagar el Equipo...", false);
    let popover_clone = popover.clone();
    item_shutdown.connect_clicked(move |_| {
        popover_clone.popdown();
        info!("Solicitando apagado del sistema...");
    });
    content_box.append(&item_shutdown);

    popover.set_child(Some(&content_box));
    menu_btn.set_popover(Some(&popover));

    menu_btn
}

fn create_menu_item(label_text: &str, is_bold: bool) -> Button {
    let btn = Button::builder()
        .css_classes(["popover-menu-item"])
        .has_frame(false)
        .build();

    let label = Label::builder()
        .label(label_text)
        .xalign(0.0)
        .build();

    if is_bold {
        label.add_css_class("menu-item-bold");
    }

    btn.set_child(Some(&label));
    btn
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
