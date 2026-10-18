use clap::Parser;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, CssProvider,
    DrawingArea, Entry, FileChooserAction, FileChooserNative, FileFilter, FlowBox, GestureClick,
    GestureDrag, Grid, Label, Orientation, PackType, Picture, PolicyType, Scale, ScrolledWindow,
    SearchEntry, Stack, StackTransitionType, Switch, WindowControls, WindowHandle,
};
use malus_de_common::config::{DisplaysConfig, DockConfig, InputConfig};
use malus_de_common::ipc::{IpcClient, IpcMessage};
use malus_de_common::logging::init_logging;
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

const APP_ID: &str = "dev.malus.Settings";

#[derive(Parser, Debug, Clone)]
#[command(name = "malus-de-settings")]
#[command(about = "Ajustes del Sistema de Malus DE (macOS System Settings)")]
struct Args {
    /// Panel inicial a mostrar: displays, wallpaper, keyboard, appearance, dock, spotlight, sound, about
    #[arg(short, long, default_value = "spotlight")]
    panel: String,
}

fn main() {
    init_logging("malus_de_settings");
    let _ = malus_de_common::config::setup_isolated_session_environment();

    if let Err(err) = libadwaita::init() {
        tracing::warn!("Aviso al inicializar libadwaita: {:?}", err);
    }

    libadwaita::StyleManager::default().set_color_scheme(
        if is_dark_mode_active() {
            libadwaita::ColorScheme::ForceDark
        } else {
            libadwaita::ColorScheme::ForceLight
        }
    );

    let args = Args::parse();
    let initial_panel = args.panel.clone();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        load_macos_settings_css();
    });

    app.connect_activate(move |app| {
        if let Some(win) = app.active_window().or_else(|| app.windows().into_iter().next()) {
            win.present();
            return;
        }
        build_settings_window(app, &initial_panel);
    });

    app.run_with_args::<&str>(&[]);
}

fn load_macos_settings_css() {
    let provider = CssProvider::new();
    let css = r#"
        window.macos-settings-window {
            background-color: @view_bg_color;
            color: @window_fg_color;
            border-radius: 0px;
            box-shadow: none;
        }

        .macos-sidebar {
            background-color: @sidebar_bg_color;
            border-right: 1px solid @borders;
            padding: 0px;
        }

        .sidebar-windowcontrols {
            min-height: 47px;
            padding: 0px 9px;
            background-color: @sidebar_bg_color;
        }

        .sidebar-body {
            padding: 0px 10px 12px 10px;
        }

        .sidebar-search {
            background-color: alpha(currentColor, 0.06);
            border: 1px solid @borders;
            border-radius: 8px;
            color: @window_fg_color;
            padding: 4px 8px;
            margin-bottom: 8px;
        }
        .sidebar-search image {
            color: alpha(currentColor, 0.60);
        }

        .profile-card {
            background-color: alpha(currentColor, 0.05);
            border: 1px solid @borders;
            border-radius: 12px;
            padding: 8px 10px;
            margin-bottom: 10px;
        }

        .avatar-circle {
            background: linear-gradient(135deg, @accent_bg_color, #5e5ce6);
            border-radius: 999px;
            min-width: 32px;
            min-height: 32px;
            color: @accent_fg_color;
            font-weight: 700;
            font-size: 14px;
        }

        .sidebar-btn {
            background: transparent;
            border: none;
            border-radius: 9px;
            padding: 7px 10px;
            color: @window_fg_color;
            font-weight: 500;
            font-size: 13.5px;
        }
        .sidebar-btn:hover {
            background-color: alpha(currentColor, 0.07);
        }
        .sidebar-btn-active {
            background-color: @accent_bg_color;
            color: @accent_fg_color;
            font-weight: 600;
        }
        .sidebar-btn-active:hover {
            filter: brightness(1.08);
        }

        .sidebar-icon-badge {
            border-radius: 7px;
            min-width: 24px;
            min-height: 24px;
            font-size: 13px;
            color: white;
        }
        .badge-blue { background: linear-gradient(180deg, #32ade6, #007aff); }
        .badge-teal { background: linear-gradient(180deg, #30b0c7, #008299); }
        .badge-indigo { background: linear-gradient(180deg, #5e5ce6, #3634a3); }
        .badge-purple { background: linear-gradient(180deg, #bf5af2, #8944ab); }
        .badge-pink { background: linear-gradient(180deg, #ff375f, #d70015); }
        .badge-gray { background: linear-gradient(180deg, #8e8e93, #636366); }
        .badge-dark { background: linear-gradient(180deg, #545458, #2c2c2e); }

        .content-header-toolbar {
            min-height: 47px;
            padding: 0px 24px;
            background-color: @view_bg_color;
            border-bottom: 1px solid alpha(currentColor, 0.08);
        }

        .nav-chevron-btn {
            color: alpha(currentColor, 0.60);
            font-size: 18px;
            font-weight: 700;
            padding: 0 4px;
            border-radius: 6px;
            background: transparent;
            min-width: 24px;
            min-height: 24px;
        }
        .nav-chevron-btn:hover {
            background-color: alpha(currentColor, 0.08);
            color: @window_fg_color;
        }

        .content-header-title {
            font-size: 15px;
            font-weight: 700;
            color: @window_fg_color;
            margin: 0px;
            padding: 0px;
        }

        .spotlight-banner-icon {
            border-radius: 10px;
            min-width: 44px;
            min-height: 44px;
            font-size: 22px;
            color: white;
        }

        .section-header-title {
            font-size: 13px;
            font-weight: 700;
            color: alpha(currentColor, 0.65);
            margin-top: 6px;
        }

        .spotlight-list-card {
            padding: 2px 0px;
            border-radius: 12px;
        }

        .list-option-row {
            padding: 8px 14px;
            min-height: 38px;
        }

        .list-row-separator {
            min-height: 1px;
            background-color: @borders;
            margin: 0 14px;
        }

        .macos-checkbox check {
            border-radius: 4px;
            border: 1.5px solid @borders;
            background: transparent;
            min-width: 16px;
            min-height: 16px;
            margin-right: 8px;
        }
        .macos-checkbox:checked check {
            background-color: @accent_bg_color;
            border-color: @accent_bg_color;
            color: @accent_fg_color;
        }

        .content-pane {
            background-color: @view_bg_color;
            padding: 16px 28px 24px 28px;
        }

        .pane-header-title {
            font-size: 20px;
            font-weight: 700;
            color: @window_fg_color;
        }

        .pane-header-subtitle {
            font-size: 12.5px;
            color: alpha(currentColor, 0.65);
        }

        .macos-group-card {
            background-color: @card_bg_color;
            border: 1px solid @borders;
            border-radius: 14px;
            padding: 16px;
            color: @card_fg_color;
        }

        .macos-canvas-card {
            background-color: alpha(currentColor, 0.04);
            border: 1px solid @borders;
            border-radius: 14px;
            padding: 12px;
        }

        .card-section-title {
            font-size: 14px;
            font-weight: 600;
            color: @card_fg_color;
        }

        .card-section-sub {
            font-size: 12px;
            color: alpha(currentColor, 0.65);
        }

        .macos-action-btn {
            background-color: alpha(currentColor, 0.08);
            border: 1px solid @borders;
            border-radius: 9px;
            padding: 6px 12px;
            color: @window_fg_color;
            font-size: 12.5px;
            font-weight: 500;
        }
        .macos-action-btn:hover {
            background-color: alpha(currentColor, 0.14);
        }

        .macos-action-btn-primary {
            background-color: @accent_bg_color;
            border: 1px solid alpha(@accent_fg_color, 0.20);
            border-radius: 9px;
            padding: 7px 14px;
            color: @accent_fg_color;
            font-size: 13px;
            font-weight: 600;
        }
        .macos-action-btn-primary:hover {
            filter: brightness(1.10);
        }


        .scale-input-entry {
            background-color: alpha(currentColor, 0.07);
            border: 1px solid @borders;
            border-radius: 6px;
            color: @window_fg_color;
            font-size: 12px;
            font-weight: 600;
            padding: 4px 8px;
            min-height: 24px;
        }
        .scale-input-entry:focus {
            border-color: @accent_bg_color;
            background-color: alpha(currentColor, 0.10);
        }

        .res-thumb-btn {
            background-color: @card_bg_color;
            border: 1.5px solid @borders;
            border-radius: 10px;
            padding: 10px 8px;
            color: @card_fg_color;
        }
        .res-thumb-btn:hover {
            background-color: alpha(currentColor, 0.08);
            border-color: alpha(currentColor, 0.20);
        }
        .res-thumb-btn-active {
            background-color: alpha(@accent_bg_color, 0.20);
            border: 2px solid @accent_bg_color;
            color: @card_fg_color;
        }

        .res-aa-label {
            font-size: 16px;
            font-weight: 700;
            color: @card_fg_color;
        }
        .res-dim-label {
            font-size: 12px;
            font-weight: 600;
            color: @card_fg_color;
        }
        .res-pct-label {
            font-size: 11px;
            color: alpha(currentColor, 0.65);
        }

        .truesize-banner-active {
            background: linear-gradient(90deg, alpha(@accent_bg_color, 0.20), alpha(#5e5ce6, 0.20));
            border: 1.5px solid @accent_bg_color;
            border-radius: 12px;
            padding: 14px 16px;
            color: @card_fg_color;
        }

        .truesize-banner-inactive {
            background-color: @card_bg_color;
            border: 1px solid @borders;
            border-radius: 12px;
            padding: 14px 16px;
            color: @card_fg_color;
        }

        .truesize-preset-btn {
            background-color: alpha(currentColor, 0.07);
            border: 1px solid @borders;
            border-radius: 10px;
            padding: 8px 12px;
            transition: all 150ms ease-in-out;
            color: @card_fg_color;
        }
        .truesize-preset-btn:hover {
            background-color: alpha(currentColor, 0.12);
        }
        .truesize-preset-btn-active {
            background-color: alpha(@accent_bg_color, 0.25);
            border: 1.5px solid @accent_bg_color;
            border-radius: 10px;
            padding: 8px 12px;
            color: @card_fg_color;
        }

        .cursor-theme-btn {
            background-color: alpha(currentColor, 0.07);
            border: 1px solid @borders;
            border-radius: 10px;
            padding: 10px 14px;
            color: @card_fg_color;
            transition: all 150ms ease-in-out;
        }
        .cursor-theme-btn:hover {
            background-color: alpha(currentColor, 0.12);
        }
        .cursor-theme-btn-active {
            background-color: alpha(@accent_bg_color, 0.25);
            border: 1.5px solid @accent_bg_color;
            border-radius: 10px;
            padding: 10px 14px;
            color: @card_fg_color;
        }

        .cursor-theme-name,
        .cursor-preview-icon {
            color: @card_fg_color;
        }

        .appearance-mode-btn {
            background-color: alpha(currentColor, 0.06);
            border: 1.5px solid @borders;
            border-radius: 12px;
            padding: 12px 16px;
            color: @card_fg_color;
            transition: all 150ms ease-in-out;
        }
        .appearance-mode-btn:hover {
            background-color: alpha(currentColor, 0.10);
        }
        .appearance-mode-btn-active {
            background-color: alpha(@accent_bg_color, 0.18);
            border: 2px solid @accent_bg_color;
            color: @card_fg_color;
            box-shadow: 0 0 10px alpha(@accent_bg_color, 0.30);
        }

        .theme-select-btn {
            background-color: alpha(currentColor, 0.05);
            border: 1.5px solid @borders;
            border-radius: 12px;
            padding: 8px 12px;
            color: @card_fg_color;
            transition: all 150ms ease-in-out;
        }
        .theme-select-btn:hover {
            background-color: alpha(currentColor, 0.10);
            border-color: alpha(currentColor, 0.20);
        }
        .theme-select-btn-active {
            background-color: alpha(@accent_bg_color, 0.18);
            border: 2px solid @accent_bg_color;
            color: @card_fg_color;
            box-shadow: 0 0 10px alpha(@accent_bg_color, 0.25);
        }

        scrolledwindow,
        viewport {
            background: transparent;
            background-color: transparent;
        }

        .wallpaper-preview-box {
            border: 1px solid @borders;
            border-radius: 14px;
            background-color: @card_bg_color;
        }

        .wallpaper-grid-item {
            border-radius: 12px;
            padding: 6px;
            transition: all 150ms ease-in-out;
            background: alpha(currentColor, 0.04);
            border: 2px solid transparent;
        }
        .wallpaper-grid-item:hover {
            background: alpha(currentColor, 0.08);
            border-color: alpha(@accent_bg_color, 0.50);
        }
        .wallpaper-grid-item-active {
            border-color: @accent_bg_color;
            background: alpha(@accent_bg_color, 0.15);
            box-shadow: 0 0 10px alpha(@accent_bg_color, 0.25);
        }
        .wallpaper-item-title {
            font-size: 11px;
            font-weight: 500;
            color: @card_fg_color;
        }
    "#;
    provider.load_from_string(css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_settings_window(app: &Application, initial_panel: &str) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ajustes del Sistema")
        .default_width(940)
        .default_height(680)
        .decorated(true)
        .css_classes(["macos-settings-window"])
        .build();

    // Reemplaza la barra superior de GNOME por un contenedor invisible,
    // preservando los márgenes CSD de redimensión interactiva y la sombra perimetral.
    let dummy_titlebar = GtkBox::builder()
        .visible(false)
        .build();
    window.set_titlebar(Some(&dummy_titlebar));

    // ==========================================
    // 1. Barra Lateral Estilo macOS Sonoma / Sequoia
    // ==========================================
    let sidebar = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-sidebar"])
        .build();

    // Controles de ventana nativos (Cerrar, Minimizar, Maximizar) según el tema activo
    let window_controls_start = WindowControls::builder()
        .side(PackType::Start)
        .valign(Align::Center)
        .build();

    let controls_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["sidebar-windowcontrols"])
        .valign(Align::Center)
        .build();
    controls_box.append(&window_controls_start);

    let traffic_handle = WindowHandle::builder()
        .child(&controls_box)
        .build();
    sidebar.append(&traffic_handle);

    let sidebar_body = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["sidebar-body"])
        .build();

    // Buscador de Ajustes
    let search_entry = SearchEntry::builder()
        .placeholder_text("Buscar")
        .css_classes(["sidebar-search"])
        .build();
    sidebar_body.append(&search_entry);

    // Tarjeta de Perfil de Usuario estilo Apple ID
    let profile_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["profile-card"])
        .build();

    let user_name = std::env::var("USER").unwrap_or_else(|_| "cristian".to_string());
    let initial_char = user_name
        .chars()
        .next()
        .unwrap_or('C')
        .to_uppercase()
        .to_string();

    let avatar = Label::builder()
        .label(&initial_char)
        .css_classes(["avatar-circle"])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let profile_text_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .spacing(2)
        .hexpand(true)
        .build();
    let profile_name = Label::builder()
        .label(&format!("{}{}", &initial_char, &user_name.chars().skip(1).collect::<String>()))
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    let profile_sub = Label::builder()
        .label("Malus DE · Ecosistema Activo")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    profile_text_box.append(&profile_name);
    profile_text_box.append(&profile_sub);

    profile_card.append(&avatar);
    profile_card.append(&profile_text_box);
    sidebar_body.append(&profile_card);

    // Items de navegación de la barra lateral (macOS System Settings)
    let nav_items = [
        ("appearance", "🌓", "Aspecto", "badge-dark"),
        ("dock", "🖥️", "Escritorio y Dock", "badge-blue"),
        ("wallpaper", "🖼️", "Fondo de pantalla", "badge-teal"),
        ("displays", "☀️", "Pantallas", "badge-blue"),
        ("spotlight", "🔍", "Spotlight", "badge-blue"),
        ("sound", "🔊", "Sonido", "badge-pink"),
        ("keyboard", "⌨️", "Teclado", "badge-indigo"),
        ("about", "⚙️", "General", "badge-gray"),
    ];

    let stack = Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .transition_duration(160)
        .hexpand(true)
        .vexpand(true)
        .build();

    let mut sidebar_buttons: Vec<(String, Button)> = Vec::new();

    for (id, icon_txt, label_txt, badge_class) in nav_items {
        let btn = Button::builder()
            .css_classes(["sidebar-btn"])
            .has_frame(false)
            .build();

        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .build();

        let badge = Label::builder()
            .label(icon_txt)
            .css_classes(["sidebar-icon-badge", badge_class])
            .valign(Align::Center)
            .halign(Align::Center)
            .build();

        let lbl = Label::builder()
            .label(label_txt)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        row.append(&badge);
        row.append(&lbl);
        btn.set_child(Some(&row));

        sidebar_body.append(&btn);
        sidebar_buttons.push((id.to_string(), btn));
    }

    let sidebar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .propagate_natural_height(true)
        .child(&sidebar_body)
        .hexpand(true)
        .vexpand(true)
        .build();
    sidebar.append(&sidebar_scroll);

    // Filtro interactivo de búsqueda de la barra lateral
    let btns_for_search = sidebar_buttons.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        for (id, btn) in &btns_for_search {
            if query.is_empty() {
                btn.set_visible(true);
            } else {
                let matches = id.contains(&query) || match id.as_str() {
                    "displays" => "pantallas monitor resolucion ppi truesize",
                    "wallpaper" => "fondo pantalla wallpaper imagen",
                    "keyboard" => "teclado distribucion lenguaje idioma xkb",
                    "appearance" => "aspecto tema oscuro claro cursor",
                    "dock" => "escritorio dock iconos aladino genie",
                    "spotlight" => "spotlight busqueda aplicaciones archivos",
                    "sound" => "sonido volumen audio",
                    "about" => "acerca sistema procesador ram version general",
                    _ => "",
                }.contains(&query);
                btn.set_visible(matches);
            }
        }
    });

    // ==========================================
    // 2. Construcción de los Paneles de Contenido
    // ==========================================
    let (displays_pane, refresh_displays_pane) = create_displays_pane();
    stack.add_named(&displays_pane, Some("displays"));

    let (wallpaper_pane, refresh_wallpaper_pane) = create_wallpaper_pane(&window);
    stack.add_named(&wallpaper_pane, Some("wallpaper"));

    let (keyboard_pane, refresh_keyboard_pane) = create_keyboard_pane();
    stack.add_named(&keyboard_pane, Some("keyboard"));

    let (dock_pane, refresh_dock_pane) = create_dock_pane();
    stack.add_named(&dock_pane, Some("dock"));

    let (spotlight_pane, refresh_spotlight_pane) = create_spotlight_pane();
    stack.add_named(&spotlight_pane, Some("spotlight"));

    let sound_pane = create_sound_pane();
    stack.add_named(&sound_pane, Some("sound"));

    let about_pane = create_about_pane();
    stack.add_named(&about_pane, Some("about"));

    // Header Toolbar macOS (‹ › chevrons + título de la vista actual)
    let content_area = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    let header_toolbar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["content-header-toolbar"])
        .spacing(12)
        .valign(Align::Center)
        .build();

    let nav_btn_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .valign(Align::Center)
        .build();

    let btn_back = Button::builder()
        .label("‹")
        .css_classes(["nav-chevron-btn"])
        .tooltip_text("Atrás")
        .has_frame(false)
        .sensitive(false)
        .valign(Align::Center)
        .build();

    let btn_forward = Button::builder()
        .label("›")
        .css_classes(["nav-chevron-btn"])
        .tooltip_text("Adelante")
        .has_frame(false)
        .sensitive(false)
        .valign(Align::Center)
        .build();

    nav_btn_box.append(&btn_back);
    nav_btn_box.append(&btn_forward);
    header_toolbar.append(&nav_btn_box);

    let active_pane_title = Label::builder()
        .label("Spotlight")
        .css_classes(["content-header-title"])
        .xalign(0.0)
        .hexpand(true)
        .valign(Align::Center)
        .build();
    header_toolbar.append(&active_pane_title);

    let window_controls_end = WindowControls::builder()
        .side(PackType::End)
        .valign(Align::Center)
        .build();
    header_toolbar.append(&window_controls_end);

    let header_handle = WindowHandle::builder()
        .child(&header_toolbar)
        .hexpand(true)
        .build();
    content_area.append(&header_handle);
    content_area.append(&stack);

    let stack_ref = stack.clone();
    let btns_ref = Rc::new(sidebar_buttons.clone());
    let ref_wall_c = refresh_wallpaper_pane.clone();
    let title_ref_wall = active_pane_title.clone();
    let goto_wallpaper = Rc::new(move || {
        for (id, btn) in btns_ref.iter() {
            if id == "wallpaper" {
                btn.add_css_class("sidebar-btn-active");
            } else {
                btn.remove_css_class("sidebar-btn-active");
            }
        }
        ref_wall_c();
        title_ref_wall.set_label("Fondo de pantalla");
        stack_ref.set_visible_child_name("wallpaper");
    });

    let (appearance_pane, refresh_appearance_pane) = create_appearance_pane(&window, Some(goto_wallpaper));
    stack.add_named(&appearance_pane, Some("appearance"));

    // Conectar navegación de la barra lateral
    let all_btns = Rc::new(sidebar_buttons.clone());
    for (id, btn) in &sidebar_buttons {
        let target_id = id.clone();
        let stack_c = stack.clone();
        let btns_c = all_btns.clone();
        let title_lbl_c = active_pane_title.clone();
        let ref_disp = refresh_displays_pane.clone();
        let ref_wall = refresh_wallpaper_pane.clone();
        let ref_kb = refresh_keyboard_pane.clone();
        let ref_app = refresh_appearance_pane.clone();
        let ref_dock = refresh_dock_pane.clone();
        let ref_spot = refresh_spotlight_pane.clone();

        let display_title = match id.as_str() {
            "displays" => "Pantallas",
            "wallpaper" => "Fondo de pantalla",
            "keyboard" => "Teclado",
            "appearance" => "Aspecto",
            "dock" => "Escritorio y Dock",
            "spotlight" => "Spotlight",
            "sound" => "Sonido",
            "about" => "General",
            _ => "Ajustes",
        }.to_string();

        btn.connect_clicked(move |_| {
            for (other_id, other_btn) in btns_c.iter() {
                if other_id == &target_id {
                    other_btn.add_css_class("sidebar-btn-active");
                } else {
                    other_btn.remove_css_class("sidebar-btn-active");
                }
            }
            if target_id == "displays" {
                ref_disp();
            } else if target_id == "wallpaper" {
                ref_wall();
            } else if target_id == "keyboard" {
                ref_kb();
            } else if target_id == "appearance" {
                ref_app();
            } else if target_id == "dock" {
                ref_dock();
            } else if target_id == "spotlight" {
                ref_spot();
            }
            title_lbl_c.set_label(&display_title);
            stack_c.set_visible_child_name(&target_id);
        });
    }

    // Seleccionar panel inicial
    let start_id = match initial_panel {
        "wallpaper" | "keyboard" | "appearance" | "sound" | "about" | "dock" | "spotlight" | "displays" => initial_panel,
        _ => "spotlight",
    };
    for (id, btn) in &sidebar_buttons {
        if id == start_id {
            btn.add_css_class("sidebar-btn-active");
        }
    }
    stack.set_visible_child_name(start_id);
    let initial_title = match start_id {
        "displays" => "Pantallas",
        "wallpaper" => "Fondo de pantalla",
        "keyboard" => "Teclado",
        "appearance" => "Aspecto",
        "dock" => "Escritorio y Dock",
        "spotlight" => "Spotlight",
        "sound" => "Sonido",
        "about" => "General",
        _ => "Ajustes",
    };
    active_pane_title.set_label(initial_title);

    let split_view = libadwaita::OverlaySplitView::builder()
        .max_sidebar_width(280.0)
        .min_sidebar_width(220.0)
        .sidebar_width_fraction(0.245)
        .sidebar_width_unit(libadwaita::LengthUnit::Px)
        .enable_show_gesture(false)
        .enable_hide_gesture(false)
        .sidebar(&sidebar)
        .content(&content_area)
        .build();

    window.set_child(Some(&split_view));
    window.present();
}

// ============================================================================
// PANEL 1: PANTALLAS (macOS System Settings > Displays)
// ============================================================================

#[derive(Clone, Debug)]
struct CanvasInteractionState {
    selected_output_idx: usize,
    dragging_idx: Option<usize>,
    drag_offset_x: f64,
    drag_offset_y: f64,
    monitor_rects: Vec<(usize, f64, f64, f64, f64)>,
}

fn create_displays_pane() -> (ScrolledWindow, Rc<dyn Fn()>) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["content-pane"])
        .spacing(16)
        .build();

    // Cabecera de Pantallas
    let header_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    let title = Label::builder()
        .label("Pantallas")
        .css_classes(["pane-header-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Arrastra los monitores en el lienzo para organizar su disposición física, ajusta la escala de cada pantalla o enlázalas 1:1 pixel a pixel por PPI.")
        .css_classes(["pane-header-subtitle"])
        .wrap(true)
        .xalign(0.0)
        .build();
    header_box.append(&title);
    header_box.append(&subtitle);
    pane.append(&header_box);

    // ------------------------------------------------------------------------
    // SECCIÓN A: Lienzo Interactivo de Disposición de Pantallas (macOS Arrange)
    // ------------------------------------------------------------------------
    let canvas_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-canvas-card"])
        .spacing(10)
        .build();

    let canvas_header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let canvas_title = Label::builder()
        .label("📐 Disposición Interactiva de Pantallas (Arrastra con el ratón para mover o intercambiar)")
        .css_classes(["card-section-title"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    let canvas_coords_lbl = Label::builder()
        .label("")
        .css_classes(["card-section-sub"])
        .build();
    canvas_header.append(&canvas_title);
    canvas_header.append(&canvas_coords_lbl);
    canvas_card.append(&canvas_header);

    let drawing_area = DrawingArea::builder()
        .content_height(225)
        .hexpand(true)
        .build();
    canvas_card.append(&drawing_area);

    // Barra de herramientas de disposición debajo del lienzo
    let arrange_toolbar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    let btn_swap = Button::builder()
        .label("⇄ Invertir Izquierda / Derecha")
        .css_classes(["macos-action-btn-primary"])
        .tooltip_text("Intercambia la posición horizontal de ambos monitores")
        .has_frame(false)
        .build();

    let btn_align_top = Button::builder()
        .label("⬆ Alinear Arriba")
        .css_classes(["macos-action-btn"])
        .tooltip_text("Alinea los bordes superiores (y = 0)")
        .has_frame(false)
        .build();

    let btn_align_center = Button::builder()
        .label("⬌ Centrar")
        .css_classes(["macos-action-btn"])
        .tooltip_text("Alinea los centros verticales de ambas pantallas")
        .has_frame(false)
        .build();

    let btn_align_bottom = Button::builder()
        .label("⬇ Alinear Abajo")
        .css_classes(["macos-action-btn"])
        .tooltip_text("Alinea los bordes inferiores")
        .has_frame(false)
        .build();

    let btn_align_desk = Button::builder()
        .label("🖥 Escritorio (y=519)")
        .css_classes(["macos-action-btn"])
        .tooltip_text("Alineación ergonómica de escritorio (24\" desplazado a y=519)")
        .has_frame(false)
        .build();

    arrange_toolbar.append(&btn_swap);
    arrange_toolbar.append(&btn_align_top);
    arrange_toolbar.append(&btn_align_center);
    arrange_toolbar.append(&btn_align_bottom);
    arrange_toolbar.append(&btn_align_desk);
    canvas_card.append(&arrange_toolbar);

    pane.append(&canvas_card);

    // ------------------------------------------------------------------------
    // SECCIÓN B: Botón Maestro de Enlace 1:1 Pixel a Pixel (TrueSize PPI Link)
    // ------------------------------------------------------------------------
    let truesize_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["truesize-banner-active"])
        .spacing(10)
        .build();

    let truesize_top_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let truesize_text_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .spacing(3)
        .build();

    let truesize_title = Label::builder()
        .label("🔗 Enlazar Resolución Física 1:1 Pixel a Pixel (TrueSize PPI)")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let truesize_sub = Label::builder()
        .label("Iguala matemáticamente el tamaño físico en milímetros entre el monitor de 27\" (163.18 PPI) y el de 24\" (183.58 PPI · factor 1.125x). Al arrastrar una ventana entre pantallas, conserva exactamente su tamaño real.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    truesize_text_col.append(&truesize_title);
    truesize_text_col.append(&truesize_sub);

    let truesize_toggle_btn = Button::builder()
        .label("🔗 Enlazadas 1:1")
        .css_classes(["macos-action-btn-primary"])
        .valign(Align::Center)
        .has_frame(false)
        .build();

    truesize_top_row.append(&truesize_text_col);
    truesize_top_row.append(&truesize_toggle_btn);

    // Fila de Ajustes Predefinidos de Enlace 1:1 solicitados por el usuario
    let truesize_presets_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .homogeneous(true)
        .build();

    let ts_preset1_btn = Button::builder()
        .css_classes(["truesize-preset-btn"])
        .has_frame(false)
        .build();
    let ts_p1_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(10)
        .margin_end(10)
        .build();
    let ts_p1_title = Label::builder()
        .label("Ajuste 1: LG 24\" a 171% · Mi 27\" a 150%")
        .css_classes(["card-section-title"])
        .xalign(0.5)
        .build();
    let ts_p1_sub = Label::builder()
        .label("Texto Nítido · Recomendado (2246×1263 / 2560×1440)")
        .css_classes(["card-section-sub"])
        .xalign(0.5)
        .build();
    ts_p1_box.append(&ts_p1_title);
    ts_p1_box.append(&ts_p1_sub);
    ts_preset1_btn.set_child(Some(&ts_p1_box));

    let ts_preset2_btn = Button::builder()
        .css_classes(["truesize-preset-btn"])
        .has_frame(false)
        .build();
    let ts_p2_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(10)
        .margin_end(10)
        .build();
    let ts_p2_title = Label::builder()
        .label("Ajuste 2: LG 24\" a 152.5% · Mi 27\" a 133.33%")
        .css_classes(["card-section-title"])
        .xalign(0.5)
        .build();
    let ts_p2_sub = Label::builder()
        .label("Mayor Espacio de Trabajo (2518×1416 / 2880×1620)")
        .css_classes(["card-section-sub"])
        .xalign(0.5)
        .build();
    ts_p2_box.append(&ts_p2_title);
    ts_p2_box.append(&ts_p2_sub);
    ts_preset2_btn.set_child(Some(&ts_p2_box));

    truesize_presets_row.append(&ts_preset1_btn);
    truesize_presets_row.append(&ts_preset2_btn);

    let truesize_math_lbl = Label::builder()
        .label("")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();

    truesize_card.append(&truesize_top_row);
    truesize_card.append(&truesize_presets_row);
    truesize_card.append(&truesize_math_lbl);
    pane.append(&truesize_card);

    // ------------------------------------------------------------------------
    // SECCIÓN C: Tarjetas de Escalado estilo macOS ("Texto más grande" <-> "Más espacio")
    //            para Monitor 1 (DP-1: LG 24") y Monitor 2 (HDMI-A-1: Mi 27")
    // ------------------------------------------------------------------------
    let monitors_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(14)
        .homogeneous(true)
        .build();

    let (mon0_card, mon0_widgets) = create_macos_monitor_scaling_card(
        0,
        "DP-1 · LG Ultra HD 24\"",
        "3840 × 2160 @ 60 Hz · 24.0\" · 183.58 PPI",
        &[
            ("1920 × 1080", "200%", 2.0),
            ("2246 × 1263", "171% (1:1)", 1.71),
            ("2518 × 1416", "152.5% (1:1)", 1.525),
            ("2560 × 1440", "150%", 1.5),
            ("3840 × 2160", "100%", 1.0),
        ],
    );

    let (mon1_card, mon1_widgets) = create_macos_monitor_scaling_card(
        1,
        "HDMI-A-1 · Mi Monitor 27\"",
        "3840 × 2160 @ 60 Hz · 27.0\" · 163.18 PPI",
        &[
            ("1920 × 1080", "200%", 2.0),
            ("2304 × 1296", "166.7%", 1.666667),
            ("2560 × 1440", "150% (1:1)", 1.5),
            ("2880 × 1620", "133.33% (1:1)", 1.333333),
            ("3840 × 2160", "100%", 1.0),
        ],
    );

    monitors_grid.append(&mon0_card);
    monitors_grid.append(&mon1_card);
    pane.append(&monitors_grid);

    // ------------------------------------------------------------------------
    // SECCIÓN D: Curvatura de Esquinas (Border-Radius de los Escritorios)
    // ------------------------------------------------------------------------
    let radius_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    let radius_top_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let radius_text_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .hexpand(true)
        .build();

    let radius_title = Label::builder()
        .label("🖥 Curvatura de Esquinas del Escritorio (Squircle Display Border-Radius)")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let radius_sub = Label::builder()
        .label("Simula el bisel redondeado superelíptico orgánico (squircle n=4.2) en las 4 esquinas de los escritorios y pantallas.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    radius_text_col.append(&radius_title);
    radius_text_col.append(&radius_sub);

    let radius_val_lbl = Label::builder()
        .label("24 px")
        .css_classes(["badge-blue", "sidebar-icon-badge"])
        .build();

    radius_top_row.append(&radius_text_col);
    radius_top_row.append(&radius_val_lbl);
    radius_card.append(&radius_top_row);

    let radius_controls_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let radius_slider = Scale::with_range(Orientation::Horizontal, 0.0, 48.0, 1.0);
    radius_slider.set_hexpand(true);
    radius_slider.set_draw_value(false);

    let preset_0 = Button::builder().label("0 px (Recto)").css_classes(["macos-action-btn"]).has_frame(false).build();
    let preset_16 = Button::builder().label("16 px").css_classes(["macos-action-btn"]).has_frame(false).build();
    let preset_20 = Button::builder().label("20 px").css_classes(["macos-action-btn"]).has_frame(false).build();
    let preset_24 = Button::builder().label("24 px (Recomendado)").css_classes(["macos-action-btn-primary"]).has_frame(false).build();
    let preset_32 = Button::builder().label("32 px").css_classes(["macos-action-btn"]).has_frame(false).build();

    radius_controls_row.append(&radius_slider);
    radius_controls_row.append(&preset_0);
    radius_controls_row.append(&preset_16);
    radius_controls_row.append(&preset_20);
    radius_controls_row.append(&preset_24);
    radius_controls_row.append(&preset_32);
    radius_card.append(&radius_controls_row);

    pane.append(&radius_card);

    // Tarjeta de Radio de Curvatura de Ventanas
    let win_radius_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    let win_radius_top_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let win_radius_text_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .hexpand(true)
        .build();

    let win_radius_title = Label::builder()
        .label("🪟 Curvatura de Esquinas de Ventanas (Squircle Window Border-Radius)")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let win_radius_sub = Label::builder()
        .label("Aplica el bisel redondeado superelíptico orgánico (squircle n=4.2) en las 4 esquinas de las ventanas de aplicaciones.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    win_radius_text_col.append(&win_radius_title);
    win_radius_text_col.append(&win_radius_sub);

    let win_radius_val_lbl = Label::builder()
        .label("32 px")
        .css_classes(["badge-blue", "sidebar-icon-badge"])
        .build();

    win_radius_top_row.append(&win_radius_text_col);
    win_radius_top_row.append(&win_radius_val_lbl);
    win_radius_card.append(&win_radius_top_row);

    let win_radius_controls_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let win_radius_slider = Scale::with_range(Orientation::Horizontal, 0.0, 48.0, 1.0);
    win_radius_slider.set_hexpand(true);
    win_radius_slider.set_draw_value(false);

    let win_preset_0 = Button::builder().label("0 px (Recto)").css_classes(["macos-action-btn"]).has_frame(false).build();
    let win_preset_16 = Button::builder().label("16 px").css_classes(["macos-action-btn"]).has_frame(false).build();
    let win_preset_24 = Button::builder().label("24 px").css_classes(["macos-action-btn"]).has_frame(false).build();
    let win_preset_32 = Button::builder().label("32 px (Recomendado)").css_classes(["macos-action-btn-primary"]).has_frame(false).build();
    let win_preset_40 = Button::builder().label("40 px").css_classes(["macos-action-btn"]).has_frame(false).build();

    win_radius_controls_row.append(&win_radius_slider);
    win_radius_controls_row.append(&win_preset_0);
    win_radius_controls_row.append(&win_preset_16);
    win_radius_controls_row.append(&win_preset_24);
    win_radius_controls_row.append(&win_preset_32);
    win_radius_controls_row.append(&win_preset_40);
    win_radius_card.append(&win_radius_controls_row);

    pane.append(&win_radius_card);

    scrolled.set_child(Some(&pane));

    // ==========================================
    // Lógica de Dibujo y Sincronización en Vivo
    // ==========================================
    let canvas_state = Rc::new(RefCell::new(CanvasInteractionState {
        selected_output_idx: 0,
        dragging_idx: None,
        drag_offset_x: 0.0,
        drag_offset_y: 0.0,
        monitor_rects: Vec::new(),
    }));

    let cs_draw = canvas_state.clone();
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        let cfg = DisplaysConfig::load();
        let mut st = cs_draw.borrow_mut();
        st.monitor_rects.clear();

        // Fondo del lienzo de disposición con cuadrícula sutil (adaptativo claro/oscuro)
        let is_dark = is_dark_mode_active();
        if is_dark {
            cr.set_source_rgb(0.078, 0.078, 0.088);
        } else {
            cr.set_source_rgb(0.91, 0.91, 0.93);
        }
        let _ = cr.paint();

        if is_dark {
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.04);
        } else {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.05);
        }
        cr.set_line_width(1.0);
        let mut gx = 0.0;
        while gx < width as f64 {
            cr.move_to(gx, 0.0);
            cr.line_to(gx, height as f64);
            gx += 32.0;
        }
        let mut gy = 0.0;
        while gy < height as f64 {
            cr.move_to(0.0, gy);
            cr.line_to(width as f64, gy);
            gy += 32.0;
        }
        let _ = cr.stroke();

        if cfg.outputs.is_empty() {
            return;
        }

        // Calcular bounding box total del espacio lógico para escalar dentro del lienzo
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for o in &cfg.outputs {
            let (lw, lh) = o.logical_size();
            min_x = min_x.min(o.x);
            min_y = min_y.min(o.y);
            max_x = max_x.max(o.x + lw);
            max_y = max_y.max(o.y + lh);
        }

        let total_w = (max_x - min_x).max(1000) as f64;
        let total_h = (max_y - min_y).max(800) as f64;

        let pad = 34.0;
        let avail_w = (width as f64 - pad * 2.0).max(200.0);
        let avail_h = (height as f64 - pad * 2.0).max(120.0);
        let zoom = (avail_w / total_w).min(avail_h / total_h);

        let origin_x = (width as f64 - total_w * zoom) / 2.0;
        let origin_y = (height as f64 - total_h * zoom) / 2.0;

        for (idx, o) in cfg.outputs.iter().enumerate() {
            let (lw, lh) = o.logical_size();
            let mut rx = origin_x + (o.x - min_x) as f64 * zoom;
            let mut ry = origin_y + (o.y - min_y) as f64 * zoom;
            let rw = lw as f64 * zoom;
            let rh = lh as f64 * zoom;

            if st.dragging_idx == Some(idx) {
                rx += st.drag_offset_x;
                ry += st.drag_offset_y;
            }

            st.monitor_rects.push((idx, rx, ry, rw, rh));

            // Dibujar rectángulo redondeado del monitor (estilo macOS Sonoma)
            let radius = 8.0;
            cr.new_sub_path();
            cr.arc(rx + rw - radius, ry + radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
            cr.arc(rx + rw - radius, ry + rh - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
            cr.arc(rx + radius, ry + rh - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cr.arc(rx + radius, ry + radius, radius, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
            cr.close_path();

            if idx == 0 {
                cr.set_source_rgb(0.12, 0.28, 0.52);
            } else {
                cr.set_source_rgb(0.18, 0.22, 0.44);
            }
            let _ = cr.fill_preserve();

            if is_dark {
                if st.selected_output_idx == idx || st.dragging_idx == Some(idx) {
                    cr.set_source_rgb(0.04, 0.52, 1.0);
                    cr.set_line_width(2.5);
                } else {
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.35);
                    cr.set_line_width(1.5);
                }
            } else {
                if st.selected_output_idx == idx || st.dragging_idx == Some(idx) {
                    cr.set_source_rgb(0.0, 0.48, 1.0);
                    cr.set_line_width(2.5);
                } else {
                    cr.set_source_rgba(0.0, 0.0, 0.0, 0.22);
                    cr.set_line_width(1.5);
                }
            }
            let _ = cr.stroke();

            // Si es el monitor primario, dibujar la barra de menús blanca superior estilo macOS
            if o.primary {
                cr.rectangle(rx + 2.0, ry + 2.0, rw - 4.0, 7.0);
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.88);
                let _ = cr.fill();
            }

            // Etiquetas dentro del monitor
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_font_size(12.5);
            let title_str = format!(
                "{} {} ({})",
                idx + 1,
                if o.effective_diagonal() >= 26.5 { "LG 27\"" } else { "Mi 24\"" },
                o.connector
            );
            cr.move_to(rx + 12.0, ry + 28.0);
            let _ = cr.show_text(&title_str);

            cr.set_source_rgba(0.90, 0.90, 0.95, 0.90);
            cr.set_font_size(11.0);
            let res_str = format!("{} × {} ({:.1}%)", lw, lh, o.scale * 100.0);
            cr.move_to(rx + 12.0, ry + 45.0);
            let _ = cr.show_text(&res_str);

            cr.set_source_rgba(0.65, 0.82, 1.0, 0.90);
            cr.set_font_size(10.5);
            let pos_str = format!(
                "pos=({}, {}) · {:.1} PPI{}",
                o.x,
                o.y,
                o.ppi(),
                if o.primary { " · ★ Principal" } else { "" }
            );
            cr.move_to(rx + 12.0, ry + 61.0);
            let _ = cr.show_text(&pos_str);
        }
    });

    let is_syncing = Rc::new(RefCell::new(false));

    let da_ref = drawing_area.clone();
    let coords_ref = canvas_coords_lbl.clone();
    let ts_card_ref = truesize_card.clone();
    let ts_btn_ref = truesize_toggle_btn.clone();
    let ts_p1_ref = ts_preset1_btn.clone();
    let ts_p2_ref = ts_preset2_btn.clone();
    let ts_math_ref = truesize_math_lbl.clone();
    let m0_w = mon0_widgets.clone();
    let m1_w = mon1_widgets.clone();
    let rad_lbl_ref = radius_val_lbl.clone();
    let rad_sld_ref = radius_slider.clone();
    let win_rad_lbl_ref = win_radius_val_lbl.clone();
    let win_rad_sld_ref = win_radius_slider.clone();
    let sync_lock = is_syncing.clone();

    let refresh_all: Rc<dyn Fn()> = Rc::new(move || {
        let cfg = DisplaysConfig::load();
        *sync_lock.borrow_mut() = true;

        // Actualizar texto de coordenadas en el lienzo
        let mut sorted: Vec<_> = cfg.outputs.iter().collect();
        sorted.sort_by_key(|o| o.x);
        let summary: Vec<String> = sorted
            .iter()
            .map(|o| {
                let (lw, lh) = o.logical_size();
                format!("{} [{}×{} @({}, {})]", o.connector, lw, lh, o.x, o.y)
            })
            .collect();
        coords_ref.set_label(&summary.join("  ↔  "));

        // Actualizar banner de Enlace 1:1 TrueSize
        if cfg.linked_physical_1to1 {
            ts_card_ref.remove_css_class("truesize-banner-inactive");
            ts_card_ref.add_css_class("truesize-banner-active");
            ts_btn_ref.set_label("🔗 Enlazadas 1:1 (Activo)");
            ts_btn_ref.add_css_class("macos-action-btn-primary");
            ts_btn_ref.remove_css_class("macos-action-btn");
        } else {
            ts_card_ref.remove_css_class("truesize-banner-active");
            ts_card_ref.add_css_class("truesize-banner-inactive");
            ts_btn_ref.set_label("🔓 Escala Independiente (Enlazar 1:1)");
            ts_btn_ref.remove_css_class("macos-action-btn-primary");
            ts_btn_ref.add_css_class("macos-action-btn");
        }

        // Actualizar estado de los botones de Ajuste 1 y Ajuste 2
        let curr_preset = cfg.current_linked_preset();
        if curr_preset == Some(1) {
            ts_p1_ref.add_css_class("truesize-preset-btn-active");
            ts_p1_ref.remove_css_class("truesize-preset-btn");
            ts_p2_ref.remove_css_class("truesize-preset-btn-active");
            ts_p2_ref.add_css_class("truesize-preset-btn");
        } else if curr_preset == Some(2) {
            ts_p2_ref.add_css_class("truesize-preset-btn-active");
            ts_p2_ref.remove_css_class("truesize-preset-btn");
            ts_p1_ref.remove_css_class("truesize-preset-btn-active");
            ts_p1_ref.add_css_class("truesize-preset-btn");
        } else {
            ts_p1_ref.remove_css_class("truesize-preset-btn-active");
            ts_p1_ref.add_css_class("truesize-preset-btn");
            ts_p2_ref.remove_css_class("truesize-preset-btn-active");
            ts_p2_ref.add_css_class("truesize-preset-btn");
        }

        if cfg.outputs.len() >= 2 {
            let o0 = &cfg.outputs[0];
            let o1 = &cfg.outputs[1];
            let mm0 = (100.0 * o0.scale / o0.ppi()) * 25.4;
            let mm1 = (100.0 * o1.scale / o1.ppi()) * 25.4;
            ts_math_ref.set_label(&format!(
                "📐 Regla Física Actual: 100 pt lógicos = {:.1} mm en {} ({:.1}%)  |  {:.1} mm en {} ({:.1}%)",
                mm0,
                o0.connector,
                o0.scale * 100.0,
                mm1,
                o1.connector,
                o1.scale * 100.0
            ));
        }

        // Actualizar Tarjeta Monitor 0
        if let Some(o0) = cfg.outputs.first() {
            update_monitor_card_ui(&m0_w, o0);
        }
        // Actualizar Tarjeta Monitor 1
        if let Some(o1) = cfg.outputs.get(1) {
            update_monitor_card_ui(&m1_w, o1);
        }

        // Actualizar Curvatura de Esquinas (Border-Radius Pantallas)
        rad_lbl_ref.set_label(&format!("{:.0} px", cfg.screen_corner_radius));
        rad_sld_ref.set_value(cfg.screen_corner_radius);

        // Actualizar Curvatura de Esquinas (Border-Radius Ventanas)
        win_rad_lbl_ref.set_label(&format!("{:.0} px", cfg.window_corner_radius));
        win_rad_sld_ref.set_value(cfg.window_corner_radius);

        da_ref.queue_draw();
        *sync_lock.borrow_mut() = false;
    });

    refresh_all();

    let save_and_reload = |cfg: DisplaysConfig, ref_fn: &Rc<dyn Fn()>| {
        let _ = cfg.save();
        let _ = IpcClient::send_command(IpcMessage::ReloadDisplaysConfig);
        ref_fn();
    };

    // Eventos de controles de Curvatura de Esquinas (Border-Radius Pantallas)
    let ref_r0 = refresh_all.clone();
    preset_0.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.screen_corner_radius = 0.0;
        save_and_reload(cfg, &ref_r0);
    });

    let ref_r16 = refresh_all.clone();
    preset_16.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.screen_corner_radius = 16.0;
        save_and_reload(cfg, &ref_r16);
    });

    let ref_r20 = refresh_all.clone();
    preset_20.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.screen_corner_radius = 20.0;
        save_and_reload(cfg, &ref_r20);
    });

    let ref_r24 = refresh_all.clone();
    preset_24.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.screen_corner_radius = 24.0;
        save_and_reload(cfg, &ref_r24);
    });

    let ref_r32 = refresh_all.clone();
    preset_32.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.screen_corner_radius = 32.0;
        save_and_reload(cfg, &ref_r32);
    });

    let sync_rad = is_syncing.clone();
    let ref_rad_sld = refresh_all.clone();
    radius_slider.connect_value_changed(move |sld| {
        if *sync_rad.borrow() {
            return;
        }
        let val = sld.value().round();
        let mut cfg = DisplaysConfig::load();
        cfg.screen_corner_radius = val;
        save_and_reload(cfg, &ref_rad_sld);
    });

    // Eventos de controles de Curvatura de Esquinas (Border-Radius Ventanas)
    let ref_w0 = refresh_all.clone();
    win_preset_0.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.window_corner_radius = 0.0;
        save_and_reload(cfg, &ref_w0);
    });

    let ref_w16 = refresh_all.clone();
    win_preset_16.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.window_corner_radius = 16.0;
        save_and_reload(cfg, &ref_w16);
    });

    let ref_w24 = refresh_all.clone();
    win_preset_24.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.window_corner_radius = 24.0;
        save_and_reload(cfg, &ref_w24);
    });

    let ref_w32 = refresh_all.clone();
    win_preset_32.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.window_corner_radius = 32.0;
        save_and_reload(cfg, &ref_w32);
    });

    let ref_w40 = refresh_all.clone();
    win_preset_40.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.window_corner_radius = 40.0;
        save_and_reload(cfg, &ref_w40);
    });

    let sync_win_rad = is_syncing.clone();
    let ref_win_rad_sld = refresh_all.clone();
    win_radius_slider.connect_value_changed(move |sld| {
        if *sync_win_rad.borrow() {
            return;
        }
        let val = sld.value().round();
        let mut cfg = DisplaysConfig::load();
        cfg.window_corner_radius = val;
        save_and_reload(cfg, &ref_win_rad_sld);
    });

    // Gestos de Clic y Arrastre en el Lienzo Interactivo
    let click_gesture = GestureClick::new();
    let cs_click = canvas_state.clone();
    let da_click = drawing_area.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let mut st = cs_click.borrow_mut();
        if let Some(&(idx, _, _, _, _)) = st
            .monitor_rects
            .iter()
            .find(|&&(_, rx, ry, rw, rh)| x >= rx && x <= rx + rw && y >= ry && y <= ry + rh)
        {
            st.selected_output_idx = idx;
            da_click.queue_draw();
        }
    });
    drawing_area.add_controller(click_gesture);

    let drag_gesture = GestureDrag::new();
    let cs_drag_begin = canvas_state.clone();
    drag_gesture.connect_drag_begin(move |_, x, y| {
        let mut st = cs_drag_begin.borrow_mut();
        if let Some(&(idx, _, _, _, _)) = st
            .monitor_rects
            .iter()
            .find(|&&(_, rx, ry, rw, rh)| x >= rx && x <= rx + rw && y >= ry && y <= ry + rh)
        {
            st.selected_output_idx = idx;
            st.dragging_idx = Some(idx);
            st.drag_offset_x = 0.0;
            st.drag_offset_y = 0.0;
        }
    });

    let cs_drag_upd = canvas_state.clone();
    let da_drag_upd = drawing_area.clone();
    drag_gesture.connect_drag_update(move |_, offset_x, offset_y| {
        let mut st = cs_drag_upd.borrow_mut();
        if st.dragging_idx.is_some() {
            st.drag_offset_x = offset_x;
            st.drag_offset_y = offset_y;
            da_drag_upd.queue_draw();
        }
    });

    let cs_drag_end = canvas_state.clone();
    let ref_drag_end = refresh_all.clone();
    drag_gesture.connect_drag_end(move |_, offset_x, offset_y| {
        let mut st = cs_drag_end.borrow_mut();
        let Some(drag_idx) = st.dragging_idx.take() else { return };
        st.drag_offset_x = 0.0;
        st.drag_offset_y = 0.0;

        let mut cfg = DisplaysConfig::load();
        if cfg.outputs.len() >= 2 {
            let other_idx = if drag_idx == 0 { 1 } else { 0 };
            // Verificar si el usuario arrastró el monitor horizontalmente al otro lado del otro monitor
            let rect_drag = st.monitor_rects.iter().find(|r| r.0 == drag_idx).copied();
            let rect_other = st.monitor_rects.iter().find(|r| r.0 == other_idx).copied();
            drop(st);

            if let (Some((_, rx, _, rw, _)), Some((_, ox, _, ow, _))) = (rect_drag, rect_other) {
                let new_center_x = rx + offset_x + rw / 2.0;
                let other_center_x = ox + ow / 2.0;
                let was_left = cfg.outputs[drag_idx].x <= cfg.outputs[other_idx].x;
                let now_left = new_center_x < other_center_x;
                if was_left != now_left {
                    cfg.swap_horizontal_order();
                }
            }

            // Aplicar desplazamiento vertical proporcional (~8x de conversión lienzo -> espacio lógico)
            let logical_dy = (offset_y * 8.0).round() as i32;
            if logical_dy.abs() >= 15 {
                cfg.adjust_vertical_offset(drag_idx, logical_dy);
            }
            save_and_reload(cfg, &ref_drag_end);
        }
    });
    drawing_area.add_controller(drag_gesture);

    // Conectar botones de barra de herramientas de disposición
    let ref_swap = refresh_all.clone();
    btn_swap.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.swap_horizontal_order();
        save_and_reload(cfg, &ref_swap);
    });

    for (btn, mode) in [
        (&btn_align_top, "top"),
        (&btn_align_center, "center"),
        (&btn_align_bottom, "bottom"),
        (&btn_align_desk, "desk"),
    ] {
        let ref_align = refresh_all.clone();
        let m = mode.to_string();
        btn.connect_clicked(move |_| {
            let mut cfg = DisplaysConfig::load();
            cfg.set_vertical_alignment(&m);
            save_and_reload(cfg, &ref_align);
        });
    }

    // Conectar botones de Ajustes Predefinidos 1 y 2 de Enlace 1:1
    let ref_p1 = refresh_all.clone();
    ts_preset1_btn.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.apply_linked_preset(1);
        save_and_reload(cfg, &ref_p1);
    });

    let ref_p2 = refresh_all.clone();
    ts_preset2_btn.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.apply_linked_preset(2);
        save_and_reload(cfg, &ref_p2);
    });

    // Conectar botón de Enlace 1:1 Pixel a Pixel
    let ref_ts = refresh_all.clone();
    truesize_toggle_btn.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.linked_physical_1to1 = !cfg.linked_physical_1to1;
        if cfg.linked_physical_1to1 {
            if cfg.current_linked_preset().is_none() {
                cfg.apply_linked_preset(1);
            } else {
                cfg.apply_1to1_link();
            }
        } else {
            cfg.recalculate_layout();
        }
        save_and_reload(cfg, &ref_ts);
    });

    // Conectar controles de Monitor 0 y Monitor 1
    wire_monitor_card_events(0, &mon0_widgets, &is_syncing, &refresh_all);
    wire_monitor_card_events(1, &mon1_widgets, &is_syncing, &refresh_all);

    (scrolled, refresh_all)
}

#[derive(Clone)]
struct MonitorCardWidgets {
    primary_btn: Button,
    active_info_lbl: Label,
    scale_slider: Scale,
    scale_entry: Entry,
    apply_btn: Button,
    preset_buttons: Vec<(f64, Button)>,
}

fn create_macos_monitor_scaling_card(
    _output_idx: usize,
    title_str: &str,
    hw_specs_str: &str,
    presets: &[(&str, &str, f64)],
) -> (GtkBox, MonitorCardWidgets) {
    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    // Cabecera del Monitor + Botón de Pantalla Principal
    let top_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let title_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .spacing(2)
        .build();

    let title_lbl = Label::builder()
        .label(title_str)
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();
    let specs_lbl = Label::builder()
        .label(hw_specs_str)
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();
    title_col.append(&title_lbl);
    title_col.append(&specs_lbl);

    let primary_btn = Button::builder()
        .label("★ Pantalla Principal")
        .css_classes(["macos-action-btn"])
        .valign(Align::Center)
        .has_frame(false)
        .build();

    top_row.append(&title_col);
    top_row.append(&primary_btn);
    card.append(&top_row);

    // Miniaturas estilo macOS: "Texto más grande" <-> "Más espacio"
    let scale_header_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .build();
    let larger_txt_lbl = Label::builder()
        .label("Texto más grande")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .hexpand(true)
        .build();
    let more_space_lbl = Label::builder()
        .label("Más espacio")
        .css_classes(["card-section-sub"])
        .xalign(1.0)
        .build();
    scale_header_row.append(&larger_txt_lbl);
    scale_header_row.append(&more_space_lbl);
    card.append(&scale_header_row);

    let thumbs_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let mut preset_buttons = Vec::new();
    for &(res_label, pct_label, scale_val) in presets {
        let btn = Button::builder()
            .css_classes(["res-thumb-btn"])
            .has_frame(false)
            .build();
        let col = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .halign(Align::Center)
            .build();
        let aa = Label::builder()
            .label("Aa")
            .css_classes(["res-aa-label"])
            .build();
        let dim = Label::builder()
            .label(res_label)
            .css_classes(["res-dim-label"])
            .build();
        let pct = Label::builder()
            .label(pct_label)
            .css_classes(["res-pct-label"])
            .build();
        col.append(&aa);
        col.append(&dim);
        col.append(&pct);
        btn.set_child(Some(&col));
        thumbs_row.append(&btn);
        preset_buttons.push((scale_val, btn));
    }
    card.append(&thumbs_row);

    // Deslizador continuo de escala fraccionaria con input exacto
    let slider_header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    let slider_title = Label::builder()
        .label("Ajuste continuo de escala:")
        .css_classes(["card-section-sub"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    let active_info_lbl = Label::builder()
        .label("150.0% · 2560 × 1440")
        .css_classes(["card-section-title"])
        .build();
    slider_header.append(&slider_title);
    slider_header.append(&active_info_lbl);
    card.append(&slider_header);

    let slider_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let scale_slider = Scale::with_range(Orientation::Horizontal, 100.0, 250.0, 2.5);
    scale_slider.set_draw_value(false);
    scale_slider.set_hexpand(true);

    let scale_entry = Entry::builder()
        .placeholder_text("150.0%")
        .width_chars(7)
        .max_width_chars(10)
        .css_classes(["scale-input-entry"])
        .valign(Align::Center)
        .build();

    let apply_btn = Button::builder()
        .label("Aplicar")
        .css_classes(["macos-action-btn"])
        .valign(Align::Center)
        .has_frame(false)
        .build();

    slider_row.append(&scale_slider);
    slider_row.append(&scale_entry);
    slider_row.append(&apply_btn);
    card.append(&slider_row);

    (
        card,
        MonitorCardWidgets {
            primary_btn,
            active_info_lbl,
            scale_slider,
            scale_entry,
            apply_btn,
            preset_buttons,
        },
    )
}

fn update_monitor_card_ui(
    w: &MonitorCardWidgets,
    o: &malus_de_common::config::DisplayOutputConfig,
) {
    let (lw, lh) = o.logical_size();
    w.active_info_lbl
        .set_label(&format!("{:.1}% · {} × {}", o.scale * 100.0, lw, lh));
    w.scale_slider
        .set_value((o.scale * 100.0).clamp(100.0, 250.0));
    w.scale_entry
        .set_text(&format!("{:.2}%", o.scale * 100.0));

    if o.primary {
        w.primary_btn.set_label("★ Pantalla Principal");
        w.primary_btn.add_css_class("macos-action-btn-primary");
        w.primary_btn.remove_css_class("macos-action-btn");
    } else {
        w.primary_btn.set_label("Hacer Principal");
        w.primary_btn.remove_css_class("macos-action-btn-primary");
        w.primary_btn.add_css_class("macos-action-btn");
    }

    for (preset_scale, btn) in &w.preset_buttons {
        if (o.scale - *preset_scale).abs() < 0.02 {
            btn.add_css_class("res-thumb-btn-active");
        } else {
            btn.remove_css_class("res-thumb-btn-active");
        }
    }
}

fn wire_monitor_card_events(
    output_idx: usize,
    w: &MonitorCardWidgets,
    is_syncing: &Rc<RefCell<bool>>,
    refresh_all: &Rc<dyn Fn()>,
) {
    // Botón Hacer Principal
    let ref_prim = refresh_all.clone();
    w.primary_btn.connect_clicked(move |_| {
        let mut cfg = DisplaysConfig::load();
        cfg.set_primary_output(output_idx);
        let _ = cfg.save();
        let _ = IpcClient::send_command(IpcMessage::ReloadDisplaysConfig);
        ref_prim();
    });

    // Presets estilo macOS
    for (target_scale, btn) in &w.preset_buttons {
        let s = *target_scale;
        let ref_p = refresh_all.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = DisplaysConfig::load();
            cfg.set_output_scale(output_idx, s);
            let _ = cfg.save();
            let _ = IpcClient::send_command(IpcMessage::ReloadDisplaysConfig);
            ref_p();
        });
    }

    // Slider continuo de escala
    let sync_c = is_syncing.clone();
    let ref_sld = refresh_all.clone();
    w.scale_slider.connect_value_changed(move |sld| {
        if *sync_c.borrow() {
            return;
        }
        let pct = (sld.value() / 2.5).round() * 2.5;
        let new_scale = (pct / 100.0).clamp(0.5, 3.0);
        let mut cfg = DisplaysConfig::load();
        cfg.set_output_scale(output_idx, new_scale);
        let _ = cfg.save();
        let _ = IpcClient::send_command(IpcMessage::ReloadDisplaysConfig);
        ref_sld();
    });

    // Input numérico directo y exacto de escala
    let parse_and_apply_scale = {
        let entry_c = w.scale_entry.clone();
        let ref_apply = refresh_all.clone();
        let is_sync_c = is_syncing.clone();
        Rc::new(move || {
            if *is_sync_c.borrow() {
                return;
            }
            let text = entry_c.text().trim().trim_end_matches('%').trim().to_string();
            if let Ok(mut val) = text.parse::<f64>() {
                if val <= 0.0 {
                    return;
                }
                // Permite introducir decimal (ej. 1.6667) o porcentaje (ej. 166.67 o 170)
                if val > 4.0 {
                    val /= 100.0;
                }
                let new_scale = (val.clamp(0.5, 3.0) * 10000.0).round() / 10000.0;
                let mut cfg = DisplaysConfig::load();
                cfg.set_output_scale(output_idx, new_scale);
                let _ = cfg.save();
                let _ = IpcClient::send_command(IpcMessage::ReloadDisplaysConfig);
                ref_apply();
            }
        })
    };

    let p_activate = parse_and_apply_scale.clone();
    w.scale_entry.connect_activate(move |_| {
        p_activate();
    });

    let p_click = parse_and_apply_scale.clone();
    w.apply_btn.connect_clicked(move |_| {
        p_click();
    });
}

// ============================================================================
// PANEL 2: TECLADO E IDIOMA (macOS System Settings > Keyboard)
// ============================================================================

fn create_keyboard_pane() -> (ScrolledWindow, Rc<dyn Fn()>) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["content-pane"])
        .spacing(16)
        .build();

    let header_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    let title = Label::builder()
        .label("Teclado y Fuentes de Entrada")
        .css_classes(["pane-header-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Selecciona la distribución de teclado XKB activa en toda la sesión Wayland (por defecto Español Latinoamericano 'latam').")
        .css_classes(["pane-header-subtitle"])
        .wrap(true)
        .xalign(0.0)
        .build();
    header_box.append(&title);
    header_box.append(&subtitle);
    pane.append(&header_box);

    let layout_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    let active_status_lbl = Label::builder()
        .label("Distribución activa: 🇲🇽 Español (Latinoamérica)")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();
    layout_card.append(&active_status_lbl);

    let layouts_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .homogeneous(true)
        .build();

    let create_layout_btn = |flag_title: &str, code_sub: &str, desc: &str| -> Button {
        let btn = Button::builder()
            .css_classes(["res-thumb-btn"])
            .has_frame(false)
            .build();
        let col = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        let t = Label::builder()
            .label(flag_title)
            .css_classes(["card-section-title"])
            .build();
        let c = Label::builder()
            .label(code_sub)
            .css_classes(["res-dim-label"])
            .build();
        let d = Label::builder()
            .label(desc)
            .css_classes(["res-pct-label"])
            .wrap(true)
            .build();
        col.append(&t);
        col.append(&c);
        col.append(&d);
        btn.set_child(Some(&col));
        btn
    };

    let btn_latam = create_layout_btn(
        "🇲🇽 Español (Latinoamérica)",
        "XKB: latam (Predeterminado)",
        "Tecla ñ, acentos muertos (áéíóú), ¿¡ y @ en AltGr+Q",
    );
    let btn_es = create_layout_btn(
        "🇪🇸 Español (España)",
        "XKB: es",
        "Distribución ISO española con ç, ñ y @ en AltGr+2",
    );
    let btn_us = create_layout_btn(
        "🇺🇸 English (US)",
        "XKB: us",
        "Distribución ANSI estadounidense estándar",
    );

    layouts_row.append(&btn_latam);
    layouts_row.append(&btn_es);
    layouts_row.append(&btn_us);
    layout_card.append(&layouts_row);

    // Campo de prueba en vivo
    let test_lbl = Label::builder()
        .label("Prueba tu teclado en vivo aquí (escribe ñ, á, é, í, ó, ú, ¿, ¡, @):")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();
    let test_entry = Entry::builder()
        .placeholder_text("Escribe aquí para probar: niño, canción, pingüino, ¿qué tal?...")
        .css_classes(["sidebar-search"])
        .build();
    layout_card.append(&test_lbl);
    layout_card.append(&test_entry);

    pane.append(&layout_card);

    // Tarjeta de Trackpad / Ratón
    let input_opts_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(10)
        .build();

    let nat_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let nat_lbl = Label::builder()
        .label("Desplazamiento natural (el contenido sigue el movimiento de los dedos)")
        .css_classes(["card-section-title"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    let nat_switch = Switch::builder()
        .valign(Align::Center)
        .active(InputConfig::load().natural_scrolling)
        .build();
    nat_row.append(&nat_lbl);
    nat_row.append(&nat_switch);
    input_opts_card.append(&nat_row);

    nat_switch.connect_state_set(|_, active| {
        let mut cfg = InputConfig::load();
        cfg.natural_scrolling = active;
        let _ = cfg.save();
        glib::Propagation::Proceed
    });

    pane.append(&input_opts_card);
    scrolled.set_child(Some(&pane));

    let b_latam = btn_latam.clone();
    let b_es = btn_es.clone();
    let b_us = btn_us.clone();
    let st_lbl = active_status_lbl.clone();

    let refresh_kb: Rc<dyn Fn()> = Rc::new(move || {
        let cfg = InputConfig::load();
        st_lbl.set_label(&format!("Distribución activa: {}", cfg.layout_display_name()));
        b_latam.remove_css_class("res-thumb-btn-active");
        b_es.remove_css_class("res-thumb-btn-active");
        b_us.remove_css_class("res-thumb-btn-active");
        match cfg.keyboard_layout.as_str() {
            "latam" => b_latam.add_css_class("res-thumb-btn-active"),
            "es" => b_es.add_css_class("res-thumb-btn-active"),
            "us" => b_us.add_css_class("res-thumb-btn-active"),
            _ => b_latam.add_css_class("res-thumb-btn-active"),
        }
    });

    refresh_kb();

    for (btn, code) in [(&btn_latam, "latam"), (&btn_es, "es"), (&btn_us, "us")] {
        let c = code.to_string();
        let ref_c = refresh_kb.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = InputConfig::load();
            cfg.keyboard_layout = c.clone();
            cfg.keyboard_variant = String::new();
            let _ = cfg.save();
            let _ = IpcClient::send_command(IpcMessage::SetKeyboardLayout {
                layout: c.clone(),
                variant: String::new(),
            });
            ref_c();
        });
    }

    (scrolled, refresh_kb)
}

// ============================================================================
// PANEL 3: APARIENCIA Y DOCK
// ============================================================================

fn get_installed_cursor_themes() -> Vec<(String, String, &'static str)> {
    let mut themes: Vec<(String, String, &'static str)> = Vec::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dirs = [
        format!("{}/.icons", home),
        format!("{}/.local/share/icons", home),
        "/usr/share/icons".to_string(),
    ];

    let mut found_names = std::collections::HashSet::new();

    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("cursors").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name != "default" && found_names.insert(name.to_string()) {
                            let (label, desc) = match name {
                                "macOS" => ("🍎 macOS Original", "Flecha negra clásica con contorno blanco Apple"),
                                "macOS-White" => ("⚪ macOS Blanco", "Variante blanca oficial de Apple"),
                                "Adwaita" => ("🐧 Adwaita", "Tema estándar de GNOME Linux"),
                                "AdwaitaLegacy" => ("🐧 Adwaita Legacy", "Tema clásico de GNOME"),
                                other => (other, "Tema instalado por el usuario"),
                            };
                            themes.push((name.to_string(), label.to_string(), desc));
                        }
                    }
                }
            }
        }
    }

    if themes.is_empty() {
        themes.push(("macOS".to_string(), "🍎 macOS Original".to_string(), "Flecha negra clásica con contorno blanco Apple"));
        themes.push(("macOS-White".to_string(), "⚪ macOS Blanco".to_string(), "Variante blanca oficial de Apple"));
        themes.push(("Adwaita".to_string(), "🐧 Adwaita".to_string(), "Tema estándar de GNOME Linux"));
    }

    // Ordenar con temas de macOS al principio
    themes.sort_by(|a, b| {
        let is_a_mac = a.0.starts_with("macOS");
        let is_b_mac = b.0.starts_with("macOS");
        match (is_a_mac, is_b_mac) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        }
    });

    themes
}

fn get_current_cursor_config() -> (String, u32) {
    let out_theme = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "cursor-theme"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().trim_matches('\'').to_string())
        .unwrap_or_else(|| "macOS".to_string());

    let out_size = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "cursor-size"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(24);

    (out_theme, out_size)
}

fn is_dark_mode_active() -> bool {
    if let Ok(out) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout);
        return s.contains("prefer-dark");
    }
    false
}

fn get_installed_gtk_themes() -> Vec<(String, String, &'static str)> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dirs = [
        format!("{}/.themes", home),
        format!("{}/.local/share/themes", home),
        "/usr/share/themes".to_string(),
    ];

    let mut found_names = std::collections::HashSet::new();
    let mut themes = Vec::new();

    // Soportar siempre el tema nativo Adwaita de GNOME / Libadwaita
    themes.push((
        "Adwaita".to_string(),
        "🐧 Adwaita (Nativo)".to_string(),
        "Tema estándar y limpio del sistema GNOME / Libadwaita",
    ));
    found_names.insert("Adwaita".to_string());

    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let has_gtk3 = path.join("gtk-3.0").exists();
                    let has_gtk4 = path.join("gtk-4.0").exists();
                    if has_gtk3 || has_gtk4 {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if !name.ends_with("-hdpi") && !name.ends_with("-xhdpi") && found_names.insert(name.to_string()) {
                                let (label, desc) = match name {
                                    "WhiteSur-Dark" => ("🍎 WhiteSur Oscuro", "Estilo macOS Sonoma oscuro con controles y botones redondeados"),
                                    "WhiteSur-Light" => ("🍎 WhiteSur Claro", "Estilo macOS Sonoma claro con controles y botones redondeados"),
                                    "WhiteSur-Dark-solid" => ("🍎 WhiteSur Oscuro Sólido", "Variante oscura con fondos sólidos sin efecto de translucidez"),
                                    "WhiteSur-Light-solid" => ("🍎 WhiteSur Claro Sólido", "Variante clara con fondos sólidos sin efecto de translucidez"),
                                    other => (other, "Tema de controles GTK instalado"),
                                };
                                themes.push((name.to_string(), label.to_string(), desc));
                            }
                        }
                    }
                }
            }
        }
    }

    // Ordenar para priorizar variantes WhiteSur y Adwaita
    themes.sort_by(|a, b| {
        let order_score = |id: &str| match id {
            "WhiteSur-Dark" => 1,
            "WhiteSur-Light" => 2,
            "WhiteSur-Dark-solid" => 3,
            "WhiteSur-Light-solid" => 4,
            "Adwaita" => 5,
            _ => 10,
        };
        order_score(&a.0).cmp(&order_score(&b.0)).then_with(|| a.0.cmp(&b.0))
    });

    themes
}

fn get_current_gtk_theme() -> String {
    Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Adwaita".to_string())
}

fn set_gtk_theme(theme_name: &str) {
    tracing::info!("🎨 Aplicando tema GTK y controles: {}", theme_name);

    // 1. gsettings
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "gtk-theme", theme_name])
        .status();

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    // 2. keyfile aislado de Malus DE
    let keyfile_path = PathBuf::from(&home).join(".config/malus-de/glib-2.0/settings/keyfile");
    if keyfile_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&keyfile_path) {
            let mut new_lines = Vec::new();
            let mut found = false;
            for line in content.lines() {
                if line.starts_with("gtk-theme=") {
                    new_lines.push(format!("gtk-theme='{}'", theme_name));
                    found = true;
                } else {
                    new_lines.push(line.to_string());
                }
            }
            if !found {
                new_lines.push(format!("gtk-theme='{}'", theme_name));
            }
            let _ = std::fs::write(&keyfile_path, new_lines.join("\n") + "\n");
        }
    }

    // 3. settings.ini para GTK3 y GTK4
    let ini_paths = [
        format!("{}/.config/malus-de/gtk-3.0/settings.ini", home),
        format!("{}/.config/malus-de/gtk-4.0/settings.ini", home),
        format!("{}/.config/gtk-3.0/settings.ini", home),
        format!("{}/.config/gtk-4.0/settings.ini", home),
    ];
    for path_str in &ini_paths {
        let path = PathBuf::from(path_str);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut lines: Vec<String> = Vec::new();
                let mut found = false;
                for line in content.lines() {
                    if line.starts_with("gtk-theme-name=") {
                        lines.push(format!("gtk-theme-name={}", theme_name));
                        found = true;
                    } else {
                        lines.push(line.to_string());
                    }
                }
                if !found {
                    lines.push(format!("gtk-theme-name={}", theme_name));
                }
                let _ = std::fs::write(&path, lines.join("\n") + "\n");
            }
        } else {
            let dark_num = if is_dark_mode_active() { "1" } else { "0" };
            let (cursor_theme, cursor_sz) = get_current_cursor_config();
            let content = format!(
                "[Settings]\ngtk-decoration-layout=close,minimize,maximize:\ngtk-application-prefer-dark-theme={}\ngtk-cursor-theme-name={}\ngtk-cursor-theme-size={}\ngtk-theme-name={}\n",
                dark_num, cursor_theme, cursor_sz, theme_name
            );
            let _ = std::fs::write(&path, content);
        }
    }

    // 4. GTK4 / Libadwaita theme linking y theme.css
    let gtk_css_paths = [
        format!("{}/.config/gtk-4.0/gtk.css", home),
        format!("{}/.config/malus-de/gtk-4.0/gtk.css", home),
    ];
    for gcss_str in &gtk_css_paths {
        let gcss_path = PathBuf::from(gcss_str);
        if gcss_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gcss_path) {
                if !content.contains("@import url(\"theme.css\");") {
                    let updated = format!("@import url(\"theme.css\");\n{}", content);
                    let _ = std::fs::write(&gcss_path, updated);
                }
            }
        }
    }

    let theme_dirs = [
        format!("{}/.themes/{}", home, theme_name),
        format!("{}/.local/share/themes/{}", home, theme_name),
        format!("/usr/share/themes/{}", theme_name),
    ];

    let mut gtk4_css_file: Option<PathBuf> = None;
    let mut theme_gtk4_dir: Option<PathBuf> = None;

    if theme_name != "Adwaita" {
        for tdir_str in &theme_dirs {
            let tdir = PathBuf::from(tdir_str);
            let candidate_css = tdir.join("gtk-4.0/gtk.css");
            if candidate_css.exists() {
                gtk4_css_file = Some(candidate_css);
                theme_gtk4_dir = Some(tdir.join("gtk-4.0"));
                break;
            }
        }
    }

    let theme_css_paths = [
        format!("{}/.config/gtk-4.0/theme.css", home),
        format!("{}/.config/malus-de/gtk-4.0/theme.css", home),
    ];

    let gtk4_folders = [
        format!("{}/.config/gtk-4.0", home),
        format!("{}/.config/malus-de/gtk-4.0", home),
    ];

    if let (Some(css_file), Some(gtk4_dir)) = (gtk4_css_file, theme_gtk4_dir) {
        let import_line = format!("@import url(\"{}\");\n", css_file.to_string_lossy());
        for tpath_str in &theme_css_paths {
            let _ = std::fs::write(tpath_str, &import_line);
        }

        // Crear enlaces simbólicos para assets y windows-assets si existen
        for folder_str in &gtk4_folders {
            let folder = PathBuf::from(folder_str);
            let assets_src = gtk4_dir.join("assets");
            let assets_dst = folder.join("assets");
            if assets_dst.is_symlink() || assets_dst.exists() {
                let _ = std::fs::remove_file(&assets_dst);
            }
            if assets_src.exists() {
                let _ = std::os::unix::fs::symlink(&assets_src, &assets_dst);
            }

            let win_assets_src = gtk4_dir.join("windows-assets");
            let win_assets_dst = folder.join("windows-assets");
            if win_assets_dst.is_symlink() || win_assets_dst.exists() {
                let _ = std::fs::remove_file(&win_assets_dst);
            }
            if win_assets_src.exists() {
                let _ = std::os::unix::fs::symlink(&win_assets_src, &win_assets_dst);
            }
        }
    } else {
        // Modo Adwaita nativo: limpiar theme.css y enlaces simbólicos de assets
        for tpath_str in &theme_css_paths {
            let _ = std::fs::write(tpath_str, "/* Native Adwaita / Libadwaita theme */\n");
        }
        for folder_str in &gtk4_folders {
            let folder = PathBuf::from(folder_str);
            let assets_dst = folder.join("assets");
            if assets_dst.is_symlink() {
                let _ = std::fs::remove_file(&assets_dst);
            }
            let win_assets_dst = folder.join("windows-assets");
            if win_assets_dst.is_symlink() {
                let _ = std::fs::remove_file(&win_assets_dst);
            }
        }
    }

    // 5. Notificar recarga en caliente de CSS en GTK4 tocando los archivos
    for gcss_str in &gtk_css_paths {
        let _ = Command::new("touch").arg(gcss_str).status();
    }

    // 6. Armonizar modo claro / oscuro si el nombre del tema lo especifica
    let is_dark = if theme_name.contains("Dark") || theme_name.contains("dark") {
        set_system_appearance_mode(true);
        true
    } else if theme_name.contains("Light") || theme_name.contains("light") {
        set_system_appearance_mode(false);
        false
    } else {
        is_dark_mode_active()
    };

    // 7. Notificar a componentes del entorno Malus DE
    let _ = IpcClient::send_command(IpcMessage::ThemeChanged {
        theme_name: theme_name.to_string(),
        is_dark,
    });
}

fn set_system_appearance_mode(to_dark: bool) {
    let target = if to_dark { "prefer-dark" } else { "prefer-light" };
    tracing::info!("🎨 Cambiando modo de apariencia del sistema: {}", target);

    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "color-scheme", target])
        .status();

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let keyfile_path = PathBuf::from(&home).join(".config/malus-de/glib-2.0/settings/keyfile");
    if keyfile_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&keyfile_path) {
            let replaced = if content.contains("color-scheme=") {
                content
                    .replace("color-scheme='prefer-light'", &format!("color-scheme='{}'", target))
                    .replace("color-scheme='prefer-dark'", &format!("color-scheme='{}'", target))
            } else {
                content
            };
            let _ = std::fs::write(&keyfile_path, replaced);
        }
    }

    let dark_val = if to_dark { "1" } else { "0" };
    let ini_paths = [
        format!("{}/.config/malus-de/gtk-3.0/settings.ini", home),
        format!("{}/.config/malus-de/gtk-4.0/settings.ini", home),
        format!("{}/.config/gtk-3.0/settings.ini", home),
        format!("{}/.config/gtk-4.0/settings.ini", home),
    ];

    for path_str in &ini_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut lines: Vec<String> = Vec::new();
                let mut found = false;
                for line in content.lines() {
                    if line.starts_with("gtk-application-prefer-dark-theme=") {
                        lines.push(format!("gtk-application-prefer-dark-theme={}", dark_val));
                        found = true;
                    } else {
                        lines.push(line.to_string());
                    }
                }
                if !found {
                    lines.push(format!("gtk-application-prefer-dark-theme={}", dark_val));
                }
                let _ = std::fs::write(&path, lines.join("\n") + "\n");
            }
        }
    }

    libadwaita::StyleManager::default().set_color_scheme(
        if to_dark {
            libadwaita::ColorScheme::ForceDark
        } else {
            libadwaita::ColorScheme::ForceLight
        }
    );

    let cur_theme = get_current_gtk_theme();
    let _ = IpcClient::send_command(IpcMessage::ThemeChanged {
        theme_name: cur_theme,
        is_dark: to_dark,
    });
}

fn set_cursor_theme_and_size(theme: &str, size: u32) {
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "cursor-theme", theme])
        .status();
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "cursor-size", &size.to_string()])
        .status();

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let default_theme_path = PathBuf::from(&home).join(".icons/default/index.theme");
    let content = format!("[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits={}\n", theme);
    let _ = std::fs::write(default_theme_path, content);

    let dark_num = if is_dark_mode_active() { "1" } else { "0" };
    let cur_gtk_theme = get_current_gtk_theme();

    // Actualizar settings.ini en malus-de y general preservando el tema GTK
    for ini_path in [
        format!("{}/.config/malus-de/gtk-3.0/settings.ini", home),
        format!("{}/.config/malus-de/gtk-4.0/settings.ini", home),
        format!("{}/.config/gtk-3.0/settings.ini", home),
        format!("{}/.config/gtk-4.0/settings.ini", home),
    ] {
        let p = PathBuf::from(ini_path);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let body = format!(
            "[Settings]\ngtk-decoration-layout=close,minimize,maximize:\ngtk-application-prefer-dark-theme={}\ngtk-cursor-theme-name={}\ngtk-cursor-theme-size={}\ngtk-theme-name={}\n",
            dark_num, theme, size, cur_gtk_theme
        );
        let _ = std::fs::write(p, body);
    }

    let keyfile_path = PathBuf::from(&home).join(".config/malus-de/glib-2.0/settings/keyfile");
    if keyfile_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&keyfile_path) {
            let mut new_lines = Vec::new();
            for line in content.lines() {
                if line.starts_with("cursor-theme=") {
                    new_lines.push(format!("cursor-theme='{}'", theme));
                } else if line.starts_with("cursor-size=") {
                    new_lines.push(format!("cursor-size={}", size));
                } else {
                    new_lines.push(line.to_string());
                }
            }
            let _ = std::fs::write(&keyfile_path, new_lines.join("\n") + "\n");
        }
    }

    let _ = Command::new("dbus-update-activation-environment")
        .args(["--systemd", &format!("XCURSOR_THEME={}", theme), &format!("XCURSOR_SIZE={}", size)])
        .status();
}

fn create_appearance_pane(
    _window: &ApplicationWindow,
    on_goto_wallpaper: Option<Rc<dyn Fn()>>,
) -> (ScrolledWindow, Rc<dyn Fn()>) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["content-pane"])
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Aspecto")
        .css_classes(["pane-header-title"])
        .xalign(0.0)
        .build();
    pane.append(&title);

    // ------------------------------------------------------------------------
    // SECCIÓN 1: Modo de Apariencia del Sistema (Claro / Oscuro)
    // ------------------------------------------------------------------------
    let appearance_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(14)
        .build();

    let app_header = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .build();

    let app_title = Label::builder()
        .label("🌓 Modo de Apariencia del Sistema")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let app_desc = Label::builder()
        .label("Selecciona el aspecto visual general (claro u oscuro) para las aplicaciones y ventanas del sistema. La barra superior mantiene su diseño base transparente con texto blanco.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    app_header.append(&app_title);
    app_header.append(&app_desc);
    appearance_card.append(&app_header);

    let modes_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(14)
        .homogeneous(true)
        .build();

    let btn_light = Button::builder()
        .css_classes(["appearance-mode-btn"])
        .has_frame(false)
        .build();
    let light_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .halign(Align::Center)
        .build();
    let light_icon = Label::builder()
        .label("☀️")
        .css_classes(["cursor-preview-icon"])
        .build();
    let light_lbl = Label::builder()
        .label("Claro")
        .css_classes(["cursor-theme-name"])
        .build();
    light_box.append(&light_icon);
    light_box.append(&light_lbl);
    btn_light.set_child(Some(&light_box));

    let btn_dark = Button::builder()
        .css_classes(["appearance-mode-btn"])
        .has_frame(false)
        .build();
    let dark_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .halign(Align::Center)
        .build();
    let dark_icon = Label::builder()
        .label("🌙")
        .css_classes(["cursor-preview-icon"])
        .build();
    let dark_lbl = Label::builder()
        .label("Oscuro")
        .css_classes(["cursor-theme-name"])
        .build();
    dark_box.append(&dark_icon);
    dark_box.append(&dark_lbl);
    btn_dark.set_child(Some(&dark_box));

    modes_grid.append(&btn_light);
    modes_grid.append(&btn_dark);
    appearance_card.append(&modes_grid);

    pane.append(&appearance_card);

    // ------------------------------------------------------------------------
    // SECCIÓN 2: Temas de Aplicaciones y Ventanas (GTK / Libadwaita)
    // ------------------------------------------------------------------------
    let theme_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(14)
        .build();

    let theme_header = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .build();

    let theme_title = Label::builder()
        .label("🎨 Tema de Aplicaciones y Ventanas (GTK)")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let theme_desc = Label::builder()
        .label("Selecciona el tema de controles, botones y estilo de ventanas para todas las aplicaciones GTK3, GTK4 y Libadwaita del entorno.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    theme_header.append(&theme_title);
    theme_header.append(&theme_desc);
    theme_card.append(&theme_header);

    let gtk_themes_list = get_installed_gtk_themes();
    let gtk_themes_grid = Grid::builder()
        .column_spacing(10)
        .row_spacing(10)
        .column_homogeneous(true)
        .build();

    let gtk_theme_buttons: Rc<RefCell<Vec<(String, Button)>>> = Rc::new(RefCell::new(Vec::new()));

    let gtk_status_hint_lbl = Label::builder()
        .label("✓ Tema aplicado en vivo para todas las aplicaciones.")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();

    let total_themes = gtk_themes_list.len();
    for (idx, (theme_id, label_txt, desc_txt)) in gtk_themes_list.iter().enumerate() {
        let btn = Button::builder()
            .css_classes(["theme-select-btn"])
            .has_frame(false)
            .build();

        let inner_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(10)
            .margin_end(10)
            .build();

        let icon_badge = Label::builder()
            .label(if theme_id.starts_with("WhiteSur") { "🍎" } else if theme_id == "Adwaita" { "🐧" } else { "🎨" })
            .css_classes([
                "spotlight-banner-icon",
                if theme_id.contains("Dark") {
                    "badge-dark"
                } else if theme_id.contains("Light") {
                    "badge-blue"
                } else if theme_id == "Adwaita" {
                    "badge-teal"
                } else {
                    "badge-indigo"
                },
            ])
            .valign(Align::Center)
            .build();

        let text_col = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let t_lbl = Label::builder()
            .label(label_txt)
            .css_classes(["card-section-title"])
            .xalign(0.0)
            .build();

        let d_lbl = Label::builder()
            .label(*desc_txt)
            .css_classes(["card-section-sub"])
            .xalign(0.0)
            .wrap(true)
            .build();

        text_col.append(&t_lbl);
        text_col.append(&d_lbl);

        inner_box.append(&icon_badge);
        inner_box.append(&text_col);
        btn.set_child(Some(&inner_box));

        let col = (idx % 2) as i32;
        let row = (idx / 2) as i32;
        if idx == total_themes - 1 && idx % 2 == 0 {
            gtk_themes_grid.attach(&btn, 0, row, 2, 1);
        } else {
            gtk_themes_grid.attach(&btn, col, row, 1, 1);
        }

        gtk_theme_buttons.borrow_mut().push((theme_id.clone(), btn));
    }

    theme_card.append(&gtk_themes_grid);
    theme_card.append(&gtk_status_hint_lbl);
    pane.append(&theme_card);

    // ------------------------------------------------------------------------
    // SECCIÓN 3: Tema y Estilo del Cursor del Ratón (Puntero macOS)
    // ------------------------------------------------------------------------
    let cursor_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(14)
        .build();

    let cursor_title_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .spacing(3)
        .build();

    let cursor_title = Label::builder()
        .label("🖱 Tema y Estilo del Puntero / Cursor")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let cursor_desc = Label::builder()
        .label("Selecciona el diseño y tamaño del cursor para todas las ventanas, aplicaciones y el escritorio. Compatible con temas instalados en ~/.icons.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    cursor_title_col.append(&cursor_title);
    cursor_title_col.append(&cursor_desc);
    cursor_card.append(&cursor_title_col);

    let themes_list = get_installed_cursor_themes();
    let themes_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .homogeneous(true)
        .build();

    let theme_buttons: Rc<RefCell<Vec<(String, Button)>>> = Rc::new(RefCell::new(Vec::new()));
    let size_buttons: Rc<RefCell<Vec<(u32, Button)>>> = Rc::new(RefCell::new(Vec::new()));

    let (active_theme, active_size) = get_current_cursor_config();
    let state_theme = Rc::new(RefCell::new(active_theme));
    let state_size = Rc::new(RefCell::new(active_size));

    let cursor_status_hint_lbl = Label::builder()
        .label("✓ Tema aplicado en vivo para nuevas ventanas y aplicaciones.")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();

    for (theme_id, label_txt, desc_txt) in &themes_list {
        let btn = Button::builder()
            .css_classes(["cursor-theme-btn"])
            .has_frame(false)
            .build();

        let inner_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(3)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();

        let t_lbl = Label::builder()
            .label(label_txt)
            .css_classes(["card-section-title"])
            .xalign(0.5)
            .build();

        let d_lbl = Label::builder()
            .label(*desc_txt)
            .css_classes(["card-section-sub"])
            .xalign(0.5)
            .wrap(true)
            .build();

        inner_box.append(&t_lbl);
        inner_box.append(&d_lbl);
        btn.set_child(Some(&inner_box));

        themes_grid.append(&btn);
        theme_buttons.borrow_mut().push((theme_id.clone(), btn));
    }
    cursor_card.append(&themes_grid);

    // Fila de selección de tamaño del cursor
    let size_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let size_prompt_lbl = Label::builder()
        .label("Tamaño del cursor:")
        .css_classes(["card-section-title"])
        .valign(Align::Center)
        .build();
    size_box.append(&size_prompt_lbl);

    let size_pills_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    for &(sz, sz_label) in &[
        (24, "24 px (Estándar)"),
        (32, "32 px (Mediano)"),
        (48, "48 px (Grande / 4K)"),
    ] {
        let p_btn = Button::builder()
            .css_classes(["macos-action-btn"])
            .label(sz_label)
            .has_frame(false)
            .build();
        size_pills_box.append(&p_btn);
        size_buttons.borrow_mut().push((sz, p_btn));
    }
    size_box.append(&size_pills_box);
    cursor_card.append(&size_box);
    cursor_card.append(&cursor_status_hint_lbl);

    pane.append(&cursor_card);

    // ------------------------------------------------------------------------
    // SECCIÓN 4: Opciones del Dock
    // ------------------------------------------------------------------------
    let dock_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    let dock_cfg = DockConfig::load_or_create_default();

    let dock_header_lbl = Label::builder()
        .label("🧭 Dock de Aplicaciones")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();
    dock_card.append(&dock_header_lbl);

    let size_lbl = Label::builder()
        .label(&format!("Tamaño de iconos del Dock: {} px", dock_cfg.icon_size))
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();
    let size_slider = Scale::with_range(Orientation::Horizontal, 36.0, 72.0, 2.0);
    size_slider.set_value(dock_cfg.icon_size as f64);
    size_slider.set_draw_value(false);

    let size_lbl_c = size_lbl.clone();
    size_slider.connect_value_changed(move |s| {
        let val = s.value().round() as u32;
        size_lbl_c.set_label(&format!("Tamaño de iconos del Dock: {} px", val));
        let mut cfg = DockConfig::load_or_create_default();
        cfg.icon_size = val;
        let _ = cfg.save();
        let _ = IpcClient::send_command(IpcMessage::ReloadDockConfig);
    });

    dock_card.append(&size_lbl);
    dock_card.append(&size_slider);
    pane.append(&dock_card);

    // ------------------------------------------------------------------------
    // SECCIÓN 5: Fondo de Pantalla (Enlace a la sección dedicada)
    // ------------------------------------------------------------------------
    let wp_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(10)
        .build();

    let wp_header_lbl = Label::builder()
        .label("🖼 Fondo de Pantalla del Escritorio")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let wp_desc_lbl = Label::builder()
        .label("Personaliza el fondo de escritorio con colecciones exclusivas de macOS, fotos de tu galería o degradados dinámicos.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    wp_card.append(&wp_header_lbl);
    wp_card.append(&wp_desc_lbl);

    if let Some(goto_wp) = on_goto_wallpaper {
        let btn_open_wp = Button::builder()
            .label("Personalizar fondos de pantalla...")
            .css_classes(["macos-action-btn"])
            .halign(Align::Start)
            .has_frame(false)
            .build();
        btn_open_wp.connect_clicked(move |_| {
            goto_wp();
        });
        wp_card.append(&btn_open_wp);
    }

    pane.append(&wp_card);

    scrolled.set_child(Some(&pane));

    // Función para refrescar la selección visual activa de temas y tamaños
    let gtk_tb_ref = gtk_theme_buttons.clone();
    let tb_ref = theme_buttons.clone();
    let sb_ref = size_buttons.clone();
    let st_theme = state_theme.clone();
    let st_size = state_size.clone();
    let btn_l_ref = btn_light.clone();
    let btn_d_ref = btn_dark.clone();

    let refresh_appearance: Rc<dyn Fn()> = Rc::new(move || {
        let current_gtk = get_current_gtk_theme();
        let (current_t, current_s) = get_current_cursor_config();
        *st_theme.borrow_mut() = current_t.clone();
        *st_size.borrow_mut() = current_s;

        // Modo Claro / Oscuro
        if is_dark_mode_active() {
            btn_d_ref.add_css_class("appearance-mode-btn-active");
            btn_l_ref.remove_css_class("appearance-mode-btn-active");
        } else {
            btn_l_ref.add_css_class("appearance-mode-btn-active");
            btn_d_ref.remove_css_class("appearance-mode-btn-active");
        }

        // Temas GTK
        for (id, btn) in gtk_tb_ref.borrow().iter() {
            if id == &current_gtk {
                btn.add_css_class("theme-select-btn-active");
                btn.remove_css_class("theme-select-btn");
            } else {
                btn.remove_css_class("theme-select-btn-active");
                btn.add_css_class("theme-select-btn");
            }
        }

        // Temas de cursor
        for (id, btn) in tb_ref.borrow().iter() {
            if id == &current_t {
                btn.add_css_class("cursor-theme-btn-active");
                btn.remove_css_class("cursor-theme-btn");
            } else {
                btn.remove_css_class("cursor-theme-btn-active");
                btn.add_css_class("cursor-theme-btn");
            }
        }

        // Tamaños de cursor
        for (sz, btn) in sb_ref.borrow().iter() {
            if *sz == current_s {
                btn.add_css_class("macos-action-btn-primary");
                btn.remove_css_class("macos-action-btn");
            } else {
                btn.remove_css_class("macos-action-btn-primary");
                btn.add_css_class("macos-action-btn");
            }
        }
    });

    refresh_appearance();

    // Conectar eventos del modo de apariencia
    let ref_app_1 = refresh_appearance.clone();
    btn_light.connect_clicked(move |_| {
        let cur = get_current_gtk_theme();
        if cur == "WhiteSur-Dark" {
            set_gtk_theme("WhiteSur-Light");
        } else if cur == "WhiteSur-Dark-solid" {
            set_gtk_theme("WhiteSur-Light-solid");
        } else {
            set_system_appearance_mode(false);
        }
        ref_app_1();
    });

    let ref_app_2 = refresh_appearance.clone();
    btn_dark.connect_clicked(move |_| {
        let cur = get_current_gtk_theme();
        if cur == "WhiteSur-Light" {
            set_gtk_theme("WhiteSur-Dark");
        } else if cur == "WhiteSur-Light-solid" {
            set_gtk_theme("WhiteSur-Dark-solid");
        } else {
            set_system_appearance_mode(true);
        }
        ref_app_2();
    });

    let ref_app_notify = refresh_appearance.clone();
    libadwaita::StyleManager::default().connect_dark_notify(move |_| {
        ref_app_notify();
    });

    // Conectar eventos de clic de los botones de tema GTK
    for (t_id, btn) in gtk_theme_buttons.borrow().iter() {
        let target_theme = t_id.clone();
        let ref_fn = refresh_appearance.clone();
        let hint_lbl_c = gtk_status_hint_lbl.clone();

        btn.connect_clicked(move |_| {
            set_gtk_theme(&target_theme);
            hint_lbl_c.set_label(&format!("✓ Tema '{}' aplicado en vivo para todas las aplicaciones.", target_theme));
            ref_fn();
        });
    }

    // Conectar eventos de clic de los botones de tema de cursor
    for (t_id, btn) in theme_buttons.borrow().iter() {
        let target_theme = t_id.clone();
        let st_size_c = state_size.clone();
        let ref_fn = refresh_appearance.clone();

        btn.connect_clicked(move |_| {
            let sz = *st_size_c.borrow();
            set_cursor_theme_and_size(&target_theme, sz);
            ref_fn();
        });
    }

    // Conectar eventos de clic de los botones de tamaño de cursor
    for (sz, btn) in size_buttons.borrow().iter() {
        let target_size = *sz;
        let st_theme_c = state_theme.clone();
        let ref_fn = refresh_appearance.clone();

        btn.connect_clicked(move |_| {
            let current_t = st_theme_c.borrow().clone();
            set_cursor_theme_and_size(&current_t, target_size);
            ref_fn();
        });
    }

    (scrolled, refresh_appearance)
}

// ============================================================================
// PANEL 3.5: FONDO DE PANTALLA (WALLPAPER)
// ============================================================================

#[derive(Clone)]
#[allow(dead_code)]
struct WallpaperPreset {
    id: String,
    title: String,
    subtitle: String,
    path: Option<PathBuf>,
}

fn get_current_wallpaper_file() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let malus_dir = PathBuf::from(&home).join(".config/malus-de");
    let candidates = [
        malus_dir.join("wallpaper.png"),
        malus_dir.join("wallpaper.jpg"),
        malus_dir.join("wallpaper.jpeg"),
        malus_dir.join("wallpaper.webp"),
        malus_dir.join("wallpaper.svg"),
    ];
    candidates
        .into_iter()
        .find(|p| p.exists() && std::fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false))
}

fn apply_system_wallpaper(source: Option<&std::path::Path>) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let malus_dir = PathBuf::from(&home).join(".config/malus-de");
    let _ = std::fs::create_dir_all(&malus_dir);

    for ext in &["png", "jpg", "jpeg", "webp", "svg"] {
        let p = malus_dir.join(format!("wallpaper.{}", ext));
        let _ = std::fs::remove_file(&p);
    }

    if let Some(src) = source {
        let ext = src
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg")
            .to_lowercase();
        let target = malus_dir.join(format!("wallpaper.{}", ext));
        let _ = std::fs::copy(src, &target);

        let uri = format!("file://{}", target.to_str().unwrap_or(""));
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri-dark", &uri])
            .status();
    } else {
        let _ = Command::new("gsettings")
            .args(["reset", "org.gnome.desktop.background", "picture-uri"])
            .status();
        let _ = Command::new("gsettings")
            .args(["reset", "org.gnome.desktop.background", "picture-uri-dark"])
            .status();
    }

    // Reiniciar malus-de-wallpaper
    let _ = Command::new("killall").args(["-9", "malus-de-wallpaper"]).status();
    let wayland_disp = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-1".to_string());

    let bin_candidates = [
        PathBuf::from(&home).join("Apps/malus-de/target/release/malus-de-wallpaper"),
        PathBuf::from("/usr/local/bin/malus-de-wallpaper"),
    ];
    let exec_cmd = bin_candidates
        .into_iter()
        .find(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "malus-de-wallpaper".to_string());

    let mut cmd = Command::new(&exec_cmd);
    cmd.env("WAYLAND_DISPLAY", &wayland_disp);
    cmd.env("GDK_BACKEND", "wayland");
    cmd.env("XDG_CONFIG_HOME", format!("{}/.config/malus-de", home));
    let _ = cmd.spawn();
}

fn get_available_wallpaper_presets() -> Vec<WallpaperPreset> {
    let mut list = Vec::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    list.push(WallpaperPreset {
        id: "default-gradient".to_string(),
        title: "Degradado Aurora".to_string(),
        subtitle: "macOS Dynamic Shader".to_string(),
        path: None,
    });

    let user_wp_dir = PathBuf::from(&home).join("Imágenes/Wallpapers");
    if let Ok(entries) = std::fs::read_dir(&user_wp_dir) {
        let mut user_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
                    let lext = ext.to_lowercase();
                    lext == "jpg" || lext == "jpeg" || lext == "png" || lext == "webp"
                } else {
                    false
                }
            })
            .collect();
        user_files.sort();
        user_files.truncate(8);
        for p in user_files {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Fondo").to_string();
            let title = if stem.contains("beach-aerial") {
                "Costa de California (macOS)".to_string()
            } else if stem.contains("Blueprint") {
                "macOS Blueprint".to_string()
            } else if stem.contains("Blackprint") {
                "macOS Blackprint".to_string()
            } else if stem.contains("do_more") {
                "Do More (Inspirational)".to_string()
            } else if stem.contains("growing-tree") {
                "Growing Tree 4K".to_string()
            } else if stem.len() > 24 {
                format!("{}...", &stem[..21])
            } else {
                stem
            };

            list.push(WallpaperPreset {
                id: format!("user-{}", p.to_string_lossy()),
                title,
                subtitle: "Galería de Wallpapers".to_string(),
                path: Some(p),
            });
        }
    }

    for dir in &["/usr/share/backgrounds/fedora-workstation", "/usr/share/backgrounds/gnome"] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut sys_files: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
                        let lext = ext.to_lowercase();
                        lext == "webp" || lext == "jpg" || lext == "png"
                    } else {
                        false
                    }
                })
                .collect();
            sys_files.sort();
            sys_files.truncate(4);
            for p in sys_files {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Fondo").to_string();
                if stem.ends_with("-d") || stem.ends_with("_dark") {
                    continue;
                }
                let clean_name = stem.trim_end_matches("-l").trim_end_matches("_light");
                list.push(WallpaperPreset {
                    id: format!("sys-{}", p.to_string_lossy()),
                    title: format!("{} (Sistema)", clean_name),
                    subtitle: "Fondo del sistema".to_string(),
                    path: Some(p),
                });
            }
        }
    }

    list
}

fn create_wallpaper_pane(window: &ApplicationWindow) -> (ScrolledWindow, Rc<dyn Fn()>) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["content-pane"])
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Fondo de pantalla")
        .css_classes(["pane-header-title"])
        .xalign(0.0)
        .build();

    let subtitle = Label::builder()
        .label("Personaliza el fondo de escritorio con colecciones auténticas de macOS, fondos dinámicos o tus fotos.")
        .css_classes(["pane-header-subtitle"])
        .xalign(0.0)
        .wrap(true)
        .build();

    pane.append(&title);
    pane.append(&subtitle);

    // ==========================================
    // TARJETA 1: Vista Previa y Fondo Actual
    // ==========================================
    let current_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    let current_title = Label::builder()
        .label("🖼 Fondo de Pantalla Actual")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();
    current_card.append(&current_title);

    let preview_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["wallpaper-preview-box"])
        .height_request(160)
        .hexpand(true)
        .build();
    current_card.append(&preview_container);

    let current_status_lbl = Label::builder()
        .label("Cargando fondo...")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();
    current_card.append(&current_status_lbl);

    let actions_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let btn_choose = Button::builder()
        .label("Seleccionar archivo...")
        .css_classes(["macos-action-btn-primary"])
        .has_frame(false)
        .build();

    let btn_reset = Button::builder()
        .label("Restablecer degradado macOS")
        .css_classes(["macos-action-btn"])
        .has_frame(false)
        .build();

    let btn_open_dir = Button::builder()
        .label("Abrir carpeta de Wallpapers")
        .css_classes(["macos-action-btn"])
        .has_frame(false)
        .build();

    actions_row.append(&btn_choose);
    actions_row.append(&btn_reset);
    actions_row.append(&btn_open_dir);
    current_card.append(&actions_row);

    pane.append(&current_card);

    // ==========================================
    // TARJETA 2: Galería de Fondos Disponibles
    // ==========================================
    let gallery_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(12)
        .build();

    let gallery_title = Label::builder()
        .label("🎨 Galería de Fondos y Colecciones")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let gallery_sub = Label::builder()
        .label("Haz clic en cualquier diseño para aplicarlo instantáneamente como fondo de pantalla.")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();

    gallery_card.append(&gallery_title);
    gallery_card.append(&gallery_sub);

    let flow = FlowBox::builder()
        .valign(Align::Start)
        .max_children_per_line(4)
        .min_children_per_line(2)
        .selection_mode(gtk4::SelectionMode::None)
        .row_spacing(12)
        .column_spacing(12)
        .build();

    let presets = get_available_wallpaper_presets();
    let preset_buttons: Rc<RefCell<Vec<(WallpaperPreset, Button)>>> = Rc::new(RefCell::new(Vec::new()));

    for preset in &presets {
        let btn = Button::builder()
            .css_classes(["wallpaper-grid-item"])
            .has_frame(false)
            .build();

        let item_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .width_request(160)
            .build();

        if let Some(ref p) = preset.path {
            let pic = if let Ok(pb) = gdk_pixbuf::Pixbuf::from_file_at_scale(p, 160, 95, true) {
                let texture = gtk4::gdk::Texture::for_pixbuf(&pb);
                Picture::for_paintable(&texture)
            } else {
                Picture::for_filename(p)
            };
            pic.set_content_fit(gtk4::ContentFit::Cover);
            pic.set_size_request(160, 95);
            pic.set_css_classes(&["wallpaper-thumb-frame"]);
            item_box.append(&pic);
        } else {
            let grad_box = GtkBox::builder()
                .css_classes(["wallpaper-gradient-box", "wallpaper-thumb-frame"])
                .width_request(160)
                .height_request(95)
                .build();
            item_box.append(&grad_box);
        }

        let name_lbl = Label::builder()
            .label(&preset.title)
            .css_classes(["wallpaper-item-title"])
            .xalign(0.5)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        item_box.append(&name_lbl);

        btn.set_child(Some(&item_box));
        flow.append(&btn);
        preset_buttons.borrow_mut().push((preset.clone(), btn));
    }

    gallery_card.append(&flow);
    pane.append(&gallery_card);

    scrolled.set_child(Some(&pane));

    // Refrescar UI del panel de fondos
    let prev_box_c = preview_container.clone();
    let status_lbl_c = current_status_lbl.clone();
    let p_btns_c = preset_buttons.clone();

    let refresh_wallpaper_pane: Rc<dyn Fn()> = Rc::new(move || {
        let active_file = get_current_wallpaper_file();

        while let Some(child) = prev_box_c.first_child() {
            prev_box_c.remove(&child);
        }

        if let Some(ref path) = active_file {
            let pic = if let Ok(pb) = gdk_pixbuf::Pixbuf::from_file_at_scale(path, 480, 200, true) {
                let texture = gtk4::gdk::Texture::for_pixbuf(&pb);
                Picture::for_paintable(&texture)
            } else {
                Picture::for_filename(path)
            };
            pic.set_content_fit(gtk4::ContentFit::Cover);
            pic.set_hexpand(true);
            pic.set_vexpand(true);
            prev_box_c.append(&pic);

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("wallpaper");
            status_lbl_c.set_label(&format!("Fondo activo: {} ({})", name, path.display()));
        } else {
            let grad = GtkBox::builder()
                .css_classes(["wallpaper-gradient-box"])
                .hexpand(true)
                .vexpand(true)
                .build();
            prev_box_c.append(&grad);
            status_lbl_c.set_label("Fondo activo: Degradado Aurora Dinámico (macOS Oficial)");
        }

        for (preset, btn) in p_btns_c.borrow().iter() {
            let is_match = match (&preset.path, &active_file) {
                (None, None) => true,
                (Some(p), Some(act)) => {
                    p == act || p.file_name() == act.file_name()
                }
                _ => false,
            };

            if is_match {
                btn.add_css_class("wallpaper-grid-item-active");
            } else {
                btn.remove_css_class("wallpaper-grid-item-active");
            }
        }
    });

    refresh_wallpaper_pane();

    for (preset, btn) in preset_buttons.borrow().iter() {
        let p_path = preset.path.clone();
        let ref_fn = refresh_wallpaper_pane.clone();
        btn.connect_clicked(move |_| {
            apply_system_wallpaper(p_path.as_deref());
            ref_fn();
        });
    }

    let ref_reset = refresh_wallpaper_pane.clone();
    btn_reset.connect_clicked(move |_| {
        apply_system_wallpaper(None);
        ref_reset();
    });

    btn_open_dir.connect_clicked(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let p = format!("{}/Imágenes/Wallpapers", home);
        let _ = Command::new("xdg-open").arg(p).spawn();
    });

    let win_c = window.clone();
    let ref_choose = refresh_wallpaper_pane.clone();
    btn_choose.connect_clicked(move |_| {
        let chooser = FileChooserNative::new(
            Some("Seleccionar fondo de pantalla"),
            Some(&win_c),
            FileChooserAction::Open,
            Some("Establecer Fondo"),
            Some("Cancelar"),
        );

        let filter = FileFilter::new();
        filter.set_name(Some("Archivos de imagen (*.jpg, *.png, *.webp, *.svg)"));
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/webp");
        filter.add_mime_type("image/svg+xml");
        chooser.add_filter(&filter);

        let ref_sub = ref_choose.clone();
        chooser.connect_response(move |dialog, resp| {
            if resp == gtk4::ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        apply_system_wallpaper(Some(&path));
                        ref_sub();
                    }
                }
            }
        });

        chooser.show();
    });

    (scrolled, refresh_wallpaper_pane)
}

// ============================================================================
// PANEL 4: SONIDO
// ============================================================================

fn create_sound_pane() -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["content-pane"])
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Sonido")
        .css_classes(["pane-header-title"])
        .xalign(0.0)
        .build();
    pane.append(&title);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(10)
        .build();

    let vol_lbl = Label::builder()
        .label("Volumen de salida maestro (PipeWire / WirePlumber)")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();
    let vol_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    vol_slider.set_value(75.0);
    vol_slider.connect_value_changed(|s| {
        let pct = s.value().round() as i32;
        let _ = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", pct)])
            .spawn();
    });

    card.append(&vol_lbl);
    card.append(&vol_slider);
    pane.append(&card);

    scrolled.set_child(Some(&pane));
    scrolled
}

// ============================================================================
// PANEL 5: ACERCA DE ESTE EQUIPO
// ============================================================================

fn create_about_pane() -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["content-pane"])
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Acerca de este equipo")
        .css_classes(["pane-header-title"])
        .xalign(0.0)
        .build();
    pane.append(&title);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["macos-group-card"])
        .spacing(8)
        .build();

    for line in [
        "🍎 Malus Desktop Environment v1.1.0",
        "🖥 Compositor: Smithay 0.7 (Wayland Multi-CRTC DRM/KMS)",
        "🎮 Gráficos: AMD Radeon Vega 7 (OpenGL ES 3.2 / GBM)",
        "📐 Monitores: LG Ultra HD 24\" (DP-1, 183.6 PPI) + Mi Monitor 27\" (HDMI-A-1, 163.2 PPI)",
        "⌨️ Teclado predeterminado: Español Latinoamericano (XKB 'latam')",
    ] {
        card.append(
            &Label::builder()
                .label(line)
                .css_classes(["card-section-title"])
                .xalign(0.0)
                .build(),
        );
    }

    pane.append(&card);
    scrolled.set_child(Some(&pane));
    scrolled
}

fn create_spotlight_pane() -> (ScrolledWindow, Rc<dyn Fn()>) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["content-pane"])
        .build();

    // 1. Tarjeta Banner Superior (igual a la imagen de referencia Descargas/ajustes.webp)
    let banner_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(14)
        .css_classes(["macos-group-card"])
        .build();

    let icon_badge = Label::builder()
        .label("🔍")
        .css_classes(["spotlight-banner-icon"])
        .valign(Align::Start)
        .build();

    let banner_text = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .hexpand(true)
        .build();

    let title_lbl = Label::builder()
        .label("Spotlight")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let desc_lbl = Label::builder()
        .label("Spotlight permite realizar búsquedas rápidas en el ordenador y muestra sugerencias de internet, Música, App Store, la cartelera de cine, ubicaciones cercanas y mucho más.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    banner_text.append(&title_lbl);
    banner_text.append(&desc_lbl);
    banner_card.append(&icon_badge);
    banner_card.append(&banner_text);
    pane.append(&banner_card);

    // 2. Título de Sección: "Resultados de la búsqueda"
    let sec_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .margin_top(4)
        .build();

    let sec_title = Label::builder()
        .label("Resultados de la búsqueda")
        .css_classes(["section-header-title"])
        .xalign(0.0)
        .build();

    let sec_sub = Label::builder()
        .label("En los resultados de la búsqueda de Spotlight, solo se incluirán las categorías seleccionadas.")
        .css_classes(["card-section-sub"])
        .xalign(0.0)
        .build();

    sec_box.append(&sec_title);
    sec_box.append(&sec_sub);
    pane.append(&sec_box);

    // 3. Tarjeta de Lista con Casillas de Verificación (CheckButton estilo macOS)
    let list_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .css_classes(["macos-group-card", "spotlight-list-card"])
        .build();

    let categories = [
        "Ajustes del Sistema",
        "Aplicaciones",
        "Calculadora",
        "Carpetas",
        "Consejos",
        "Contactos",
        "Conversión",
        "Correo y mensajes",
        "Definición",
        "Documentos",
        "Documentos PDF",
        "Eventos y recordatorios",
        "Hojas de cálculo",
        "Imágenes",
        "Música",
        "Otros",
        "Presentaciones",
        "Sitios web",
        "Sugerencias de Siri",
        "Tipos de letra",
        "Vídeos",
    ];

    for (i, cat) in categories.iter().enumerate() {
        if i > 0 {
            let sep = GtkBox::builder()
                .css_classes(["list-row-separator"])
                .build();
            list_card.append(&sep);
        }

        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .css_classes(["list-option-row"])
            .build();

        let chk = CheckButton::builder()
            .label(*cat)
            .active(true)
            .hexpand(true)
            .css_classes(["macos-checkbox"])
            .build();

        row.append(&chk);
        list_card.append(&row);
    }

    pane.append(&list_card);
    scrolled.set_child(Some(&pane));

    let refresh: Rc<dyn Fn()> = Rc::new(|| {});
    (scrolled, refresh)
}

fn create_dock_pane() -> (ScrolledWindow, Rc<dyn Fn()>) {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["content-pane"])
        .build();

    let banner_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(14)
        .css_classes(["macos-group-card"])
        .build();

    let icon_badge = Label::builder()
        .label("🖥️")
        .css_classes(["spotlight-banner-icon", "badge-blue"])
        .valign(Align::Start)
        .build();

    let banner_text = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .hexpand(true)
        .build();

    let title_lbl = Label::builder()
        .label("Escritorio y Dock")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .build();

    let desc_lbl = Label::builder()
        .label("Personaliza la apariencia del Dock flotante, el tamaño de iconos, el efecto Genie de minimización y el auto-ocultado.")
        .css_classes(["card-section-sub"])
        .wrap(true)
        .xalign(0.0)
        .build();

    banner_text.append(&title_lbl);
    banner_text.append(&desc_lbl);
    banner_card.append(&icon_badge);
    banner_card.append(&banner_text);
    pane.append(&banner_card);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["macos-group-card"])
        .build();

    let cfg = DockConfig::load_or_create_default();

    // Tamaño del Dock Slider
    let size_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();
    let size_lbl = Label::builder()
        .label("Tamaño:")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .width_request(130)
        .build();
    let size_slider = Scale::with_range(Orientation::Horizontal, 36.0, 96.0, 2.0);
    size_slider.set_hexpand(true);
    size_slider.set_value(cfg.icon_size as f64);
    size_slider.connect_value_changed(|s| {
        let mut cfg = DockConfig::load_or_create_default();
        cfg.icon_size = s.value().round() as u32;
        let _ = cfg.save();
        let _ = IpcClient::send_command(IpcMessage::ReloadDockConfig);
    });
    size_row.append(&size_lbl);
    size_row.append(&size_slider);
    card.append(&size_row);

    // Ampliación / Magnification
    let mag_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();
    let mag_lbl = Label::builder()
        .label("Ampliación:")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .hexpand(true)
        .build();
    let mag_sw = Switch::builder()
        .active(cfg.magnification_enabled)
        .valign(Align::Center)
        .build();
    mag_sw.connect_state_set(|_, active| {
        let mut cfg = DockConfig::load_or_create_default();
        cfg.magnification_enabled = active;
        let _ = cfg.save();
        let _ = IpcClient::send_command(IpcMessage::ReloadDockConfig);
        glib::Propagation::Proceed
    });
    mag_row.append(&mag_lbl);
    mag_row.append(&mag_sw);
    card.append(&mag_row);

    // Efecto de minimización
    let eff_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();
    let eff_lbl = Label::builder()
        .label("Efecto al minimizar:")
        .css_classes(["card-section-title"])
        .xalign(0.0)
        .hexpand(true)
        .build();
    let eff_val = Label::builder()
        .label("🧞 Efecto Aladino (Genie Lamp)")
        .css_classes(["card-section-sub"])
        .build();
    eff_row.append(&eff_lbl);
    eff_row.append(&eff_val);
    card.append(&eff_row);

    pane.append(&card);
    scrolled.set_child(Some(&pane));

    let refresh: Rc<dyn Fn()> = Rc::new(|| {});
    (scrolled, refresh)
}
