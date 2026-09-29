use glib::ControlFlow;
use gtk4::Label;
use std::time::Duration;

pub fn create_clock_widget() -> Label {
    let mut builder = Label::builder().css_classes(["clock-label", "topbar-btn"]);

    if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
        builder = builder.cursor(&cursor);
    }

    let clock_label = builder.build();

    update_clock(&clock_label);

    let label_clone = clock_label.clone();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        update_clock(&label_clone);
        ControlFlow::Continue
    });

    clock_label
}

fn update_clock(label: &Label) {
    if let Ok(now) = glib::DateTime::now_local() {
        if let Ok(formatted) = now.format("%a %-d %b  %H:%M:%S") {
            label.set_text(&formatted);
        }
    }
}
