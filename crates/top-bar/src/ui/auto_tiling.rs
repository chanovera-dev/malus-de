use gtk4::prelude::*;
use gtk4::{Button, Image};
use std::cell::Cell;
use std::rc::Rc;
use tracing::info;

pub fn create_auto_tiling_widget() -> Button {
    let is_tiling = Rc::new(Cell::new(false));

    let btn = Button::builder()
        .css_classes(["tiling-pill", "topbar-btn"])
        .tooltip_text("Modo Ventanas: Flotante (Click para Auto-Tiling Super+Y)")
        .has_frame(false)
        .build();

    let icon = Image::from_icon_name("view-paged-symbolic");
    btn.set_child(Some(&icon));

    let btn_clone = btn.clone();
    let state_clone = is_tiling.clone();

    btn.connect_clicked(move |_| {
        let currently_tiling = state_clone.get();
        let new_state = !currently_tiling;
        state_clone.set(new_state);

        if new_state {
            let img = Image::from_icon_name("view-grid-symbolic");
            btn_clone.set_child(Some(&img));
            btn_clone.add_css_class("active-pill");
            btn_clone.set_tooltip_text(Some("Modo Ventanas: Auto-Tiling Activo (Super+Y)"));
            info!("Auto-Tiling Activado");
        } else {
            let img = Image::from_icon_name("view-paged-symbolic");
            btn_clone.set_child(Some(&img));
            btn_clone.remove_css_class("active-pill");
            btn_clone.set_tooltip_text(Some("Modo Ventanas: Flotante Libre (Super+Y)"));
            info!("Auto-Tiling Desactivado (Modo Flotante)");
        }
    });

    btn
}
