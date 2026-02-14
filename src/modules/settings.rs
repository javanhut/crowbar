use crate::config::Config;
use crate::css;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Settings {
    pub widget: gtk4::Box,
}

impl Settings {
    pub fn new(config: Rc<RefCell<Config>>, windows: Rc<RefCell<Vec<gtk4::Window>>>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("settings");

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("settings-button");
        menu_button.set_has_frame(false);

        // Kenaz rune
        let rune = gtk4::Label::new(Some("\u{16B2}"));
        rune.add_css_class("module-rune");
        menu_button.set_child(Some(&rune));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("settings-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);
        popover_content.set_size_request(350, -1);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16B2}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("M\u{00ed}mir's Well (Settings)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        // === Theme Section ===
        let theme_label = gtk4::Label::new(Some("Realms (Theme)"));
        theme_label.add_css_class("settings-section-label");
        theme_label.set_halign(gtk4::Align::Start);
        popover_content.append(&theme_label);

        let theme_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        theme_box.set_homogeneous(true);

        let presets = [
            ("Nordic", "nordic"),
            ("Light", "light"),
            ("Warm", "warm"),
            ("Frost", "frost"),
        ];

        let theme_buttons: Rc<Vec<gtk4::Button>> = Rc::new(
            presets
                .iter()
                .map(|(label, _)| {
                    let btn = gtk4::Button::with_label(label);
                    btn.add_css_class("settings-theme-btn");
                    btn
                })
                .collect(),
        );

        // Set active based on current config
        {
            let config_borrow = config.borrow();
            let current = &config_borrow.theme.preset;
            for (i, (_, preset)) in presets.iter().enumerate() {
                if current == preset {
                    theme_buttons[i].add_css_class("active");
                }
            }
        }

        // Connect theme buttons
        for (i, btn) in theme_buttons.iter().enumerate() {
            let preset = presets[i].1.to_string();
            let config_clone = config.clone();
            let buttons_clone = theme_buttons.clone();
            btn.connect_clicked(move |_| {
                // Update active class on all buttons
                for b in buttons_clone.iter() {
                    b.remove_css_class("active");
                }
                buttons_clone[i].add_css_class("active");

                let mut cfg = config_clone.borrow_mut();
                cfg.theme.preset = preset.clone();
                cfg.theme.colors = crate::config::ThemeColors::for_preset(&preset);
                css::apply_theme(&cfg.theme);
                if let Err(e) = cfg.save() {
                    eprintln!("Failed to save config: {e}");
                }
            });
            theme_box.append(btn);
        }

        popover_content.append(&theme_box);

        // Separator
        let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep1.add_css_class("settings-separator");
        popover_content.append(&sep1);

        // === Module Toggles ===
        let modules_label = gtk4::Label::new(Some("Runes (Modules)"));
        modules_label.add_css_class("settings-section-label");
        modules_label.set_halign(gtk4::Align::Start);
        popover_content.append(&modules_label);

        let modules_scroll = gtk4::ScrolledWindow::new();
        modules_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        modules_scroll.set_max_content_height(200);
        modules_scroll.set_propagate_natural_height(true);

        let modules_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

        let module_names = [
            ("workspaces", "Workspaces"),
            ("window_title", "Window Title"),
            ("app_tracker", "App Tracker"),
            ("media", "Media"),
            ("app_finder", "App Finder"),
            ("systray", "System Tray"),
            ("connectivity", "Connectivity"),
            ("audio", "Audio"),
            ("brightness", "Brightness"),
            ("power", "Power"),
            ("battery", "Battery"),
            ("clock", "Clock"),
            ("power_menu", "Power Menu"),
        ];

        for (id, display_name) in &module_names {
            let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            row.add_css_class("settings-module-row");

            let label = gtk4::Label::new(Some(display_name));
            label.set_hexpand(true);
            label.set_halign(gtk4::Align::Start);
            label.add_css_class("settings-module-name");
            row.append(&label);

            let switch = gtk4::Switch::new();
            switch.set_valign(gtk4::Align::Center);
            switch.add_css_class("settings-module-switch");

            // Check if module is in current config
            {
                let cfg = config.borrow();
                let is_enabled = cfg.modules.left.contains(&id.to_string())
                    || cfg.modules.right.contains(&id.to_string());
                switch.set_active(is_enabled);
            }

            let config_clone = config.clone();
            let module_id = id.to_string();
            switch.connect_state_set(move |_, state| {
                let mut cfg = config_clone.borrow_mut();
                if state {
                    // Add to appropriate side if not present
                    let is_left = matches!(
                        module_id.as_str(),
                        "workspaces" | "window_title" | "app_tracker" | "media" | "app_finder"
                    );
                    if is_left {
                        if !cfg.modules.left.contains(&module_id) {
                            cfg.modules.left.push(module_id.clone());
                        }
                    } else if !cfg.modules.right.contains(&module_id) {
                        cfg.modules.right.push(module_id.clone());
                    }
                } else {
                    cfg.modules.left.retain(|m| m != &module_id);
                    cfg.modules.right.retain(|m| m != &module_id);
                }
                if let Err(e) = cfg.save() {
                    eprintln!("Failed to save config: {e}");
                }
                glib::Propagation::Proceed
            });

            row.append(&switch);
            modules_box.append(&row);
        }

        modules_scroll.set_child(Some(&modules_box));
        popover_content.append(&modules_scroll);

        // Separator
        let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep2.add_css_class("settings-separator");
        popover_content.append(&sep2);

        // === Bar Settings ===
        let bar_label = gtk4::Label::new(Some("Bifrost (Bar)"));
        bar_label.add_css_class("settings-section-label");
        bar_label.set_halign(gtk4::Align::Start);
        popover_content.append(&bar_label);

        // Height
        let height_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let height_label = gtk4::Label::new(Some("Height"));
        height_label.set_hexpand(true);
        height_label.set_halign(gtk4::Align::Start);
        height_label.add_css_class("settings-module-name");
        height_row.append(&height_label);

        let height_adj = gtk4::Adjustment::new(
            config.borrow().bar.height as f64,
            20.0,
            64.0,
            2.0,
            4.0,
            0.0,
        );
        let height_spin = gtk4::SpinButton::new(Some(&height_adj), 2.0, 0);
        height_spin.add_css_class("settings-spin");

        let config_height = config.clone();
        let windows_height = windows.clone();
        height_spin.connect_value_changed(move |spin| {
            let mut cfg = config_height.borrow_mut();
            cfg.bar.height = spin.value() as i32;
            let is_vertical = cfg.bar.position == "left" || cfg.bar.position == "right";
            if let Err(e) = cfg.save() {
                eprintln!("Failed to save config: {e}");
            }
            let thickness = spin.value() as i32;
            for win in windows_height.borrow().iter() {
                if is_vertical {
                    win.set_default_size(thickness, -1);
                } else {
                    win.set_default_size(-1, thickness);
                }
            }
        });
        height_row.append(&height_spin);
        popover_content.append(&height_row);

        // Position
        let pos_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let pos_label = gtk4::Label::new(Some("Position"));
        pos_label.set_hexpand(true);
        pos_label.set_halign(gtk4::Align::Start);
        pos_label.add_css_class("settings-module-name");
        pos_row.append(&pos_label);

        let pos_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

        let positions = [("Top", "top"), ("Bottom", "bottom"), ("Left", "left"), ("Right", "right")];
        let pos_buttons: Rc<Vec<gtk4::Button>> = Rc::new(
            positions
                .iter()
                .map(|(label, _)| {
                    let btn = gtk4::Button::with_label(label);
                    btn.add_css_class("settings-theme-btn");
                    btn
                })
                .collect(),
        );

        {
            let current = &config.borrow().bar.position;
            for (i, (_, pos)) in positions.iter().enumerate() {
                if current == pos {
                    pos_buttons[i].add_css_class("active");
                }
            }
        }

        for (i, btn) in pos_buttons.iter().enumerate() {
            let pos_value = positions[i].1.to_string();
            let config_pos = config.clone();
            let buttons_clone = pos_buttons.clone();
            let windows_pos = windows.clone();
            let popover_pos = popover.clone();
            btn.connect_clicked(move |_| {
                for b in buttons_clone.iter() {
                    b.remove_css_class("active");
                }
                buttons_clone[i].add_css_class("active");

                let mut cfg = config_pos.borrow_mut();
                cfg.bar.position = pos_value.clone();
                if let Err(e) = cfg.save() {
                    eprintln!("Failed to save config: {e}");
                }

                // Close popover before moving to avoid UI glitch
                popover_pos.popdown();

                // Apply position change live to all bar windows
                if gtk4_layer_shell::is_supported() {
                    let thickness = cfg.bar.height;
                    for win in windows_pos.borrow().iter() {
                        crate::bar::apply_position_anchors(win, &pos_value, thickness);
                    }
                }
            });
            pos_box.append(btn);
        }

        pos_row.append(&pos_box);
        popover_content.append(&pos_row);

        // Restart note
        let note = gtk4::Label::new(Some("Module changes take effect on restart"));
        note.add_css_class("settings-note");
        note.set_halign(gtk4::Align::Start);
        popover_content.append(&note);

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        Self { widget }
    }
}
