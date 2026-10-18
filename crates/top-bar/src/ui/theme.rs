use gtk4::prelude::*;
use gtk4::Popover;
use std::cell::RefCell;
use std::process::Command;

thread_local! {
    static REGISTERED_POPOVERS: RefCell<Vec<glib::WeakRef<Popover>>> = RefCell::new(Vec::new());
}

pub fn is_dark_mode_enabled() -> bool {
    if let Ok(out) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout);
        return s.contains("prefer-dark");
    }
    false
}

pub fn apply_theme_to_popover(popover: &Popover, is_dark: bool) {
    if is_dark {
        popover.remove_css_class("light-popover");
    } else {
        popover.add_css_class("light-popover");
    }
}

pub fn register_popover(popover: &Popover) {
    let is_dark = is_dark_mode_enabled();
    apply_theme_to_popover(popover, is_dark);

    let pop_weak = popover.downgrade();
    popover.connect_show(move |_| {
        if let Some(pop) = pop_weak.upgrade() {
            apply_theme_to_popover(&pop, is_dark_mode_enabled());
        }
    });

    REGISTERED_POPOVERS.with(|list| {
        let mut l = list.borrow_mut();
        l.retain(|w| w.upgrade().is_some());
        l.push(popover.downgrade());
    });
}

pub fn update_all_popovers_theme(is_dark: bool) {
    REGISTERED_POPOVERS.with(|list| {
        let mut l = list.borrow_mut();
        l.retain(|w| {
            if let Some(pop) = w.upgrade() {
                apply_theme_to_popover(&pop, is_dark);
                true
            } else {
                false
            }
        });
    });
}
