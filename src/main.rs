mod bar;
mod config;
mod css;
mod hyprland;
mod modules;
mod system;

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let app = gtk4::Application::new(Some("com.github.javanhut.crowbar"), Default::default());

    let bars: Rc<RefCell<Vec<bar::Bar>>> = Rc::new(RefCell::new(Vec::new()));

    let bars_activate = bars.clone();
    app.connect_activate(move |app| {
        // Load config
        let config = config::Config::load();

        // Load CSS
        let css_path = css::load_css();

        // Start CSS hot-reload watcher
        if let Some(path) = css_path {
            css::start_css_watcher(path);
        }

        // Apply theme colors
        css::apply_theme(&config.theme);

        // Check layer shell support
        if !gtk4_layer_shell::is_supported() {
            eprintln!("Warning: Layer shell not supported. Running as regular window.");
            eprintln!(
                "Make sure you're running on Wayland with a compositor that supports wlr-layer-shell."
            );
        }

        // Try to connect to Hyprland IPC
        let client = match hyprland::HyprlandClient::new() {
            Ok(c) => {
                Some(Rc::new(c))
            }
            Err(e) => {
                eprintln!("Warning: Could not connect to Hyprland IPC: {e}");
                eprintln!("Workspace and window features will be disabled.");
                None
            }
        };

        // Shared config for settings module
        let shared_config = Rc::new(RefCell::new(config.clone()));

        // Create bars (multi-monitor support)
        let new_bars = bar::create_bars(app, client, &config, shared_config);
        *bars_activate.borrow_mut() = new_bars;
    });

    let bars_shutdown = bars.clone();
    app.connect_shutdown(move |_| {
        for bar in bars_shutdown.borrow_mut().iter_mut() {
            bar.stop();
        }
    });

    app.run_with_args::<String>(&[]);
}
