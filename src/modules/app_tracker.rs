use crate::hyprland::HyprlandClient;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

struct AppInfo {
    class: String,
    title: String,
    windows: Vec<WindowInfo>,
    focused: bool,
    all_minimized: bool,
}

struct WindowInfo {
    address: String,
    minimized: bool,
}

pub struct AppTracker {
    pub widget: gtk4::Box,
    client: Rc<HyprlandClient>,
    apps: Rc<RefCell<HashMap<String, AppInfo>>>,
    buttons: Rc<RefCell<HashMap<String, gtk4::Button>>>,
    menu_open: Rc<Cell<bool>>,
    source_id: RefCell<Option<glib::SourceId>>,
}

impl AppTracker {
    pub fn new(client: Rc<HyprlandClient>, interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        widget.add_css_class("app-tracker");

        // ᛗ Mannaz rune
        let rune = gtk4::Label::new(Some("\u{16D7}"));
        rune.add_css_class("module-rune");
        rune.set_tooltip_text(Some("\u{16D7} Mannaz - Running Apps"));
        widget.append(&rune);

        let apps = Rc::new(RefCell::new(HashMap::new()));
        let buttons = Rc::new(RefCell::new(HashMap::new()));
        let menu_open = Rc::new(Cell::new(false));

        let tracker = Self {
            widget,
            client,
            apps,
            buttons,
            menu_open,
            source_id: RefCell::new(None),
        };

        tracker.refresh();
        tracker.start_updates(interval_secs);
        tracker
    }

    fn start_updates(&self, interval_secs: u32) {
        let client = self.client.clone();
        let widget = self.widget.clone();
        let apps = self.apps.clone();
        let buttons = self.buttons.clone();
        let menu_open = self.menu_open.clone();

        *self.source_id.borrow_mut() = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            do_refresh(&client, &widget, &apps, &buttons, &menu_open);
            glib::ControlFlow::Continue
        }));
    }

    pub fn refresh(&self) {
        do_refresh(&self.client, &self.widget, &self.apps, &self.buttons, &self.menu_open);
    }

    pub fn stop(&self) {
        if let Some(id) = self.source_id.borrow_mut().take() {
            id.remove();
        }
    }
}

fn do_refresh(
    client: &Rc<HyprlandClient>,
    widget: &gtk4::Box,
    apps_cell: &Rc<RefCell<HashMap<String, AppInfo>>>,
    buttons_cell: &Rc<RefCell<HashMap<String, gtk4::Button>>>,
    menu_open: &Rc<Cell<bool>>,
) {
    // Skip rebuild while a context menu is open to prevent destroying its parent
    if menu_open.get() {
        return;
    }

    let Ok(clients) = client.clients() else {
        return;
    };
    let active_class = client
        .active_window()
        .map(|w| w.class.to_lowercase())
        .unwrap_or_default();

    let mut new_apps: HashMap<String, AppInfo> = HashMap::new();

    for c in &clients {
        let class = c.class.to_lowercase();
        if class.is_empty() {
            continue;
        }

        let is_minimized = c.workspace.name.starts_with("special:minimized");
        let win_info = WindowInfo {
            address: c.address.clone(),
            minimized: is_minimized,
        };

        if let Some(app) = new_apps.get_mut(&class) {
            app.windows.push(win_info);
            if class == active_class {
                app.focused = true;
            }
        } else {
            new_apps.insert(
                class.clone(),
                AppInfo {
                    class: class.clone(),
                    title: c.class.clone(),
                    windows: vec![win_info],
                    focused: class == active_class,
                    all_minimized: false,
                },
            );
        }
    }

    for app in new_apps.values_mut() {
        app.all_minimized = app.windows.iter().all(|w| w.minimized);
    }

    // Rebuild UI
    {
        let mut buttons = buttons_cell.borrow_mut();
        for btn in buttons.values() {
            widget.remove(btn);
        }
        buttons.clear();
    }

    let mut classes: Vec<String> = new_apps.keys().cloned().collect();
    classes.sort();

    for class in &classes {
        let app = &new_apps[class];
        let btn = create_app_button(client, app, apps_cell, widget, menu_open);
        buttons_cell.borrow_mut().insert(class.clone(), btn.clone());
        widget.append(&btn);
    }

    *apps_cell.borrow_mut() = new_apps;
}

fn create_app_button(
    client: &Rc<HyprlandClient>,
    app: &AppInfo,
    apps_cell: &Rc<RefCell<HashMap<String, AppInfo>>>,
    tracker_widget: &gtk4::Box,
    menu_open: &Rc<Cell<bool>>,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("app-button");

    let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let icon = get_app_icon(&app.class);
    icon.add_css_class("app-icon");
    content.append(&icon);

    if app.windows.len() > 1 {
        let count_label = gtk4::Label::new(Some(&app.windows.len().to_string()));
        count_label.add_css_class("app-count");
        content.append(&count_label);
    }

    btn.set_child(Some(&content));

    if app.focused {
        btn.add_css_class("focused");
    }
    if app.all_minimized {
        btn.add_css_class("minimized");
    }

    let minimized_count = app.windows.iter().filter(|w| w.minimized).count();
    let tooltip = if minimized_count > 0 {
        format!(
            "{}\n{} window(s) ({} minimized)\nClick: Focus/Restore | Right-click: Options",
            app.title,
            app.windows.len(),
            minimized_count
        )
    } else {
        format!(
            "{}\n{} window(s)\nClick: Focus | Right-click: Options",
            app.title,
            app.windows.len()
        )
    };
    btn.set_tooltip_text(Some(&tooltip));

    // Left click - focus/cycle/restore
    let client_clone = client.clone();
    let class = app.class.clone();
    let apps_ref = apps_cell.clone();
    btn.connect_clicked(move |_| {
        on_app_clicked(&client_clone, &class, &apps_ref);
    });

    // Right click - context menu
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(gdk::BUTTON_SECONDARY);
    let client_clone = client.clone();
    let class = app.class.clone();
    let apps_ref = apps_cell.clone();
    let btn_ref = btn.clone();
    let tracker_ref = tracker_widget.clone();
    let menu_flag = menu_open.clone();
    gesture.connect_pressed(move |_, _n, _x, _y| {
        show_context_menu(&client_clone, &class, &apps_ref, &btn_ref, &tracker_ref, &menu_flag);
    });
    btn.add_controller(gesture);

    btn
}

fn get_app_icon(class: &str) -> gtk4::Image {
    let icon_mappings: HashMap<&str, &str> = HashMap::from([
        ("firefox", "firefox"),
        ("chromium", "chromium"),
        ("google-chrome", "google-chrome"),
        ("code", "visual-studio-code"),
        ("code-oss", "code-oss"),
        ("discord", "discord"),
        ("spotify", "spotify"),
        ("steam", "steam"),
        ("telegram-desktop", "telegram"),
        ("thunar", "thunar"),
        ("nautilus", "nautilus"),
        ("kitty", "kitty"),
        ("alacritty", "Alacritty"),
        ("foot", "foot"),
        ("wezterm", "wezterm"),
        ("obsidian", "obsidian"),
        ("vlc", "vlc"),
        ("mpv", "mpv"),
        ("gimp", "gimp"),
        ("inkscape", "inkscape"),
        ("blender", "blender"),
        ("libreoffice", "libreoffice-startcenter"),
    ]);

    let icon_name = icon_mappings
        .get(class.to_lowercase().as_str())
        .copied()
        .unwrap_or(class);

    let display = gdk::Display::default().unwrap();
    let icon_theme = gtk4::IconTheme::for_display(&display);

    if icon_theme.has_icon(icon_name) {
        gtk4::Image::from_icon_name(icon_name)
    } else if icon_theme.has_icon(&class.to_lowercase()) {
        gtk4::Image::from_icon_name(&class.to_lowercase())
    } else {
        gtk4::Image::from_icon_name("application-x-executable")
    }
}

fn on_app_clicked(
    client: &HyprlandClient,
    class: &str,
    apps_cell: &Rc<RefCell<HashMap<String, AppInfo>>>,
) {
    let apps = apps_cell.borrow();
    let Some(app) = apps.get(class) else { return };
    if app.windows.is_empty() {
        return;
    }

    if app.all_minimized {
        let _ = client.restore_window(&app.windows[0].address);
        return;
    }

    let visible: Vec<&WindowInfo> = app.windows.iter().filter(|w| !w.minimized).collect();
    if visible.is_empty() {
        return;
    }

    if visible.len() == 1 {
        let _ = client.focus_window(&visible[0].address);
    } else {
        let active = client.active_window().ok();
        let current_addr = active.map(|w| w.address).unwrap_or_default();
        let current_idx = visible.iter().position(|w| w.address == current_addr);
        let next_idx = current_idx.map(|i| (i + 1) % visible.len()).unwrap_or(0);
        let _ = client.focus_window(&visible[next_idx].address);
    }
}

fn show_context_menu(
    client: &Rc<HyprlandClient>,
    class: &str,
    apps_cell: &Rc<RefCell<HashMap<String, AppInfo>>>,
    btn: &gtk4::Button,
    _tracker_widget: &gtk4::Box,
    menu_open: &Rc<Cell<bool>>,
) {
    let apps = apps_cell.borrow();
    let Some(app) = apps.get(class) else { return };

    // Block refreshes while the menu is open
    menu_open.set(true);

    let popover = gtk4::Popover::new();
    popover.add_css_class("app-menu");
    popover.set_parent(btn);
    popover.set_position(gtk4::PositionType::Bottom);
    popover.set_autohide(true);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    let header = gtk4::Label::new(Some(&app.title));
    header.add_css_class("app-menu-header");
    content.append(&header);

    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&sep);

    let visible_count = app.windows.iter().filter(|w| !w.minimized).count();
    let minimized_count = app.windows.iter().filter(|w| w.minimized).count();

    // Minimize
    if visible_count > 0 {
        let label = if visible_count == 1 {
            "Minimize".to_string()
        } else {
            format!("Minimize All ({visible_count})")
        };
        let addresses: Vec<String> = app
            .windows
            .iter()
            .filter(|w| !w.minimized)
            .map(|w| w.address.clone())
            .collect();
        let client_c = client.clone();
        let popover_c = popover.clone();
        let menu_btn = create_menu_item("\u{16BE}", &label, move || {
            for addr in &addresses {
                let _ = client_c.minimize_window(addr);
            }
            popover_c.popdown();
        });
        content.append(&menu_btn);
    }

    // Restore
    if minimized_count > 0 {
        let label = if minimized_count == 1 {
            "Restore".to_string()
        } else {
            format!("Restore All ({minimized_count})")
        };
        let addresses: Vec<String> = app
            .windows
            .iter()
            .filter(|w| w.minimized)
            .map(|w| w.address.clone())
            .collect();
        let client_c = client.clone();
        let popover_c = popover.clone();
        let menu_btn = create_menu_item("\u{16D2}", &label, move || {
            for addr in &addresses {
                let _ = client_c.restore_window(addr);
            }
            popover_c.popdown();
        });
        content.append(&menu_btn);
    }

    let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&sep2);

    // Close window
    if let Some(first) = app.windows.first() {
        let addr = first.address.clone();
        let client_c = client.clone();
        let popover_c = popover.clone();
        let close_btn = create_menu_item("\u{16C1}", "Close Window", move || {
            let _ = client_c.close_window(&addr);
            popover_c.popdown();
        });
        content.append(&close_btn);
    }

    // Close all
    if app.windows.len() > 1 {
        let addresses: Vec<String> = app.windows.iter().map(|w| w.address.clone()).collect();
        let label = format!("Close All ({})", addresses.len());
        let client_c = client.clone();
        let popover_c = popover.clone();
        let close_all_btn = create_menu_item("\u{16BA}", &label, move || {
            for addr in &addresses {
                let _ = client_c.close_window(addr);
            }
            popover_c.popdown();
        });
        close_all_btn.add_css_class("app-menu-danger");
        content.append(&close_all_btn);
    }

    // New instance
    let sep3 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    content.append(&sep3);

    let class_name = class.to_string();
    let client_c = client.clone();
    let popover_c = popover.clone();
    let new_btn = create_menu_item("\u{16A0}", "New Instance", move || {
        let _ = client_c.dispatch(&format!("exec {class_name}"));
        popover_c.popdown();
    });
    content.append(&new_btn);

    popover.set_child(Some(&content));

    // Unblock refreshes and clean up when the popover closes
    let menu_flag = menu_open.clone();
    popover.connect_closed(move |p| {
        menu_flag.set(false);
        p.unparent();
    });

    popover.popup();
}

fn create_menu_item(rune: &str, label: &str, on_click: impl Fn() + 'static) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("app-menu-item");

    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let rune_label = gtk4::Label::new(Some(rune));
    rune_label.add_css_class("app-menu-rune");

    let text_label = gtk4::Label::new(Some(label));
    text_label.add_css_class("app-menu-label");

    box_.append(&rune_label);
    box_.append(&text_label);
    btn.set_child(Some(&box_));

    btn.connect_clicked(move |_| on_click());
    btn
}
