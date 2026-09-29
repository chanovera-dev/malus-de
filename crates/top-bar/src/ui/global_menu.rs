use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, MenuButton, Orientation, Popover};

pub fn create_global_menu_widget() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .build();

    // Nombre de la App activa (ej. "Malus DE" o nombre de ventana activa)
    let app_title_btn = MenuButton::builder()
        .css_classes(["app-title-btn", "topbar-btn"])
        .build();
    let app_title_label = Label::builder()
        .label("Malus DE")
        .css_classes(["active-app-name"])
        .build();
    app_title_btn.set_child(Some(&app_title_label));

    let popover = Popover::builder()
        .css_classes(["malus-popover-menu"])
        .has_arrow(false)
        .build();
    let content = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let hide_btn = Button::builder()
        .label("Ocultar Malus DE          ⌘H")
        .has_frame(false)
        .css_classes(["popover-menu-item"])
        .build();
    let quit_btn = Button::builder()
        .label("Salir de Malus DE        ⌘Q")
        .has_frame(false)
        .css_classes(["popover-menu-item"])
        .build();
    content.append(&hide_btn);
    content.append(&quit_btn);
    popover.set_child(Some(&content));
    app_title_btn.set_popover(Some(&popover));

    container.append(&app_title_btn);

    // Menús estándar: Archivo, Edición, Ver, Ventana, Ayuda
    let standard_menus = ["Archivo", "Edición", "Ver", "Ventana", "Ayuda"];
    for menu_name in standard_menus {
        let menu_btn = MenuButton::builder()
            .css_classes(["global-menu-item", "topbar-btn"])
            .label(menu_name)
            .build();

        let menu_popover = Popover::builder()
            .css_classes(["malus-popover-menu"])
            .has_arrow(false)
            .build();
        let placeholder_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();
        let placeholder_item = Button::builder()
            .label(&format!("Acción de {}", menu_name))
            .has_frame(false)
            .css_classes(["popover-menu-item"])
            .build();
        placeholder_box.append(&placeholder_item);
        menu_popover.set_child(Some(&placeholder_box));
        menu_btn.set_popover(Some(&menu_popover));

        container.append(&menu_btn);
    }

    container
}
