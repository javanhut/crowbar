use crate::config::Config;
use crate::hyprland::{EventListener, HyprEvent, HyprlandClient};
use crate::modules;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::LayerShell;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bar {
    pub window: gtk4::Window,
    workspaces: Option<Rc<modules::workspaces::Workspaces>>,
    window_title: Option<Rc<modules::window_title::WindowTitle>>,
    app_tracker: Option<Rc<modules::app_tracker::AppTracker>>,
    media: Option<modules::media::Media>,
    _app_finder: Option<modules::app_finder::AppFinder>,
    systray: Option<modules::systray::Systray>,
    connectivity: Option<modules::connectivity::Connectivity>,
    audio: Option<modules::audio::Audio>,
    brightness: Option<modules::brightness::Brightness>,
    power: Option<modules::power::Power>,
    battery: Option<modules::battery::Battery>,
    clock: Option<modules::clock::Clock>,
    _power_menu: Option<modules::power_menu::PowerMenu>,
    _settings: Option<modules::settings::Settings>,
    event_listener: Option<EventListener>,
}

impl Bar {
    pub fn new(
        app: &gtk4::Application,
        client: Option<Rc<HyprlandClient>>,
        config: &Config,
        shared_config: Rc<RefCell<Config>>,
        monitor: Option<&gtk4::gdk::Monitor>,
        all_windows: Rc<RefCell<Vec<gtk4::Window>>>,
    ) -> Self {
        let window = gtk4::Window::new();
        window.set_title(Some("CrowBar"));
        app.add_window(&window);

        // Layer shell setup
        if gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            window.set_layer(gtk4_layer_shell::Layer::Top);
            apply_position_anchors(&window, &config.bar.position, config.bar.height);
            window.auto_exclusive_zone_enable();
            window.set_namespace(Some("crowbar"));
            window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

            if let Some(mon) = monitor {
                window.set_monitor(Some(mon));
            }
        }

        let is_vertical = config.bar.position == "left" || config.bar.position == "right";
        let orientation = if is_vertical {
            gtk4::Orientation::Vertical
        } else {
            gtk4::Orientation::Horizontal
        };
        let container = gtk4::Box::new(orientation, 4);
        container.add_css_class("bar-container");
        if is_vertical {
            container.add_css_class("bar-vertical");
        } else {
            container.add_css_class("bar-horizontal");
        }
        container.add_css_class(&format!("bar-{}", config.bar.position));
        container.set_margin_start(4);
        container.set_margin_end(4);
        container.set_margin_top(2);
        container.set_margin_bottom(2);
        window.set_child(Some(&container));

        let mut workspaces = None;
        let mut window_title = None;
        let mut app_tracker = None;
        let mut media = None;
        let mut app_finder = None;
        let mut systray = None;
        let mut connectivity = None;
        let mut audio = None;
        let mut brightness = None;
        let mut power = None;
        let mut battery = None;
        let mut clock = None;
        let mut power_menu = None;
        let mut settings = None;

        // Build left modules
        for module_name in &config.modules.left {
            match module_name.as_str() {
                "workspaces" => {
                    if let Some(ref c) = client {
                        let ws = Rc::new(modules::workspaces::Workspaces::new(c.clone()));
                        container.append(&ws.widget);
                        workspaces = Some(ws);
                    }
                }
                "separator" => {
                    let sep = gtk4::Separator::new(gtk4::Orientation::Vertical);
                    container.append(&sep);
                }
                "window_title" => {
                    if let Some(ref c) = client {
                        let wt = Rc::new(modules::window_title::WindowTitle::new(c.clone()));
                        container.append(&wt.widget);
                        window_title = Some(wt);
                    }
                }
                "app_tracker" => {
                    if let Some(ref c) = client {
                        let at = Rc::new(modules::app_tracker::AppTracker::new(
                            c.clone(),
                            config.intervals.app_tracker,
                        ));
                        container.append(&at.widget);
                        app_tracker = Some(at);
                    }
                }
                "media" => {
                    let m = modules::media::Media::new(config.intervals.media);
                    container.append(&m.widget);
                    media = Some(m);
                }
                "app_finder" => {
                    let af = modules::app_finder::AppFinder::new();
                    container.append(&af.widget);
                    app_finder = Some(af);
                }
                _ => {}
            }
        }

        // Spacer
        let spacer = gtk4::Box::new(orientation, 0);
        if is_vertical {
            spacer.set_vexpand(true);
        } else {
            spacer.set_hexpand(true);
        }
        container.append(&spacer);

        // Build right modules
        for module_name in &config.modules.right {
            match module_name.as_str() {
                "systray" => {
                    let st = modules::systray::Systray::new();
                    container.append(&st.widget);
                    systray = Some(st);
                }
                "connectivity" => {
                    let conn =
                        modules::connectivity::Connectivity::new(config.intervals.connectivity);
                    container.append(&conn.widget);
                    connectivity = Some(conn);
                }
                "audio" => {
                    let a = modules::audio::Audio::new();
                    container.append(&a.widget);
                    audio = Some(a);
                }
                "brightness" => {
                    let b = modules::brightness::Brightness::new(config.intervals.brightness);
                    container.append(&b.widget);
                    brightness = Some(b);
                }
                "power" => {
                    let p = modules::power::Power::new(config.intervals.power);
                    container.append(&p.widget);
                    power = Some(p);
                }
                "battery" => {
                    let bat = modules::battery::Battery::new(config.intervals.battery);
                    container.append(&bat.widget);
                    battery = Some(bat);
                }
                "separator" => {
                    let sep = gtk4::Separator::new(gtk4::Orientation::Vertical);
                    container.append(&sep);
                }
                "clock" => {
                    let c = modules::clock::Clock::new(config.intervals.clock);
                    container.append(&c.widget);
                    clock = Some(c);
                }
                "settings" => {
                    let s = modules::settings::Settings::new(shared_config.clone(), all_windows.clone());
                    container.append(&s.widget);
                    settings = Some(s);
                }
                "power_menu" => {
                    let pm = modules::power_menu::PowerMenu::new();
                    container.append(&pm.widget);
                    power_menu = Some(pm);
                }
                "app_finder" => {
                    let af = modules::app_finder::AppFinder::new();
                    container.append(&af.widget);
                    app_finder = Some(af);
                }
                _ => {}
            }
        }

        Self {
            window,
            workspaces,
            window_title,
            app_tracker,
            media,
            _app_finder: app_finder,
            systray,
            connectivity,
            audio,
            brightness,
            power,
            battery,
            clock,
            _power_menu: power_menu,
            _settings: settings,
            event_listener: None,
        }
    }

    pub fn setup_events(&mut self) {
        // Audio events (independent of Hyprland)
        if let Some(ref mut audio) = self.audio {
            audio.setup_events();
        }

        if self.workspaces.is_none() && self.window_title.is_none() {
            return;
        }

        let listener = match EventListener::new() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Warning: Could not set up event listener: {e}");
                return;
            }
        };

        let (sender, receiver) = async_channel::unbounded::<HyprEvent>();

        if let Err(e) = listener.start(sender) {
            eprintln!("Warning: Could not start event listener: {e}");
            return;
        }

        // Clone Rc handles for safe sharing with the async event handler
        let workspaces = self.workspaces.clone();
        let window_title = self.window_title.clone();
        let app_tracker = self.app_tracker.clone();

        glib::spawn_future_local(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    HyprEvent::Workspace(_)
                    | HyprEvent::CreateWorkspace(_)
                    | HyprEvent::DestroyWorkspace(_) => {
                        if let Some(ref ws) = workspaces {
                            ws.refresh();
                        }
                    }
                    HyprEvent::ActiveWindow(_) | HyprEvent::WindowTitle(_) => {
                        if let Some(ref wt) = window_title {
                            wt.refresh();
                        }
                    }
                    HyprEvent::CloseWindow(_) => {
                        if let Some(ref wt) = window_title {
                            wt.refresh();
                        }
                        if let Some(ref at) = app_tracker {
                            at.refresh();
                        }
                    }
                    HyprEvent::OpenWindow(_) => {
                        if let Some(ref at) = app_tracker {
                            at.refresh();
                        }
                    }
                }
            }
        });

        self.event_listener = Some(listener);
    }

    pub fn show(&self) {
        self.window.set_visible(true);
    }

    pub fn stop(&mut self) {
        if let Some(ref at) = self.app_tracker {
            at.stop();
        }
        if let Some(ref mut m) = self.media {
            m.stop();
        }
        if let Some(ref st) = self.systray {
            st.stop();
        }
        if let Some(ref mut conn) = self.connectivity {
            conn.stop();
        }
        if let Some(ref audio) = self.audio {
            audio.stop();
        }
        if let Some(ref mut b) = self.brightness {
            b.stop();
        }
        if let Some(ref mut p) = self.power {
            p.stop();
        }
        if let Some(ref mut bat) = self.battery {
            bat.stop();
        }
        if let Some(ref mut c) = self.clock {
            c.stop();
        }
        if let Some(ref el) = self.event_listener {
            el.stop();
        }
    }
}

pub fn apply_position_anchors(window: &gtk4::Window, position: &str, thickness: i32) {
    let is_vertical = position == "left" || position == "right";

    // Set anchors
    match position {
        "bottom" => {
            window.set_anchor(gtk4_layer_shell::Edge::Top, false);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        }
        "left" => {
            window.set_anchor(gtk4_layer_shell::Edge::Top, true);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, false);
        }
        "right" => {
            window.set_anchor(gtk4_layer_shell::Edge::Top, true);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_anchor(gtk4_layer_shell::Edge::Left, false);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        }
        _ => {
            // "top" or any default
            window.set_anchor(gtk4_layer_shell::Edge::Top, true);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);
            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        }
    }

    // Set size request: thickness is height for top/bottom, width for left/right
    // Using set_size_request instead of set_default_size so it works on already-visible windows
    if is_vertical {
        window.set_size_request(thickness, -1);
        window.set_default_size(thickness, -1);
    } else {
        window.set_size_request(-1, thickness);
        window.set_default_size(-1, thickness);
    }

    // Update container orientation and CSS classes
    if let Some(child) = window.child() {
        if let Ok(container) = child.downcast::<gtk4::Box>() {
            let new_orientation = if is_vertical {
                gtk4::Orientation::Vertical
            } else {
                gtk4::Orientation::Horizontal
            };
            container.set_orientation(new_orientation);

            // Toggle vertical/horizontal CSS classes for styling
            if is_vertical {
                container.add_css_class("bar-vertical");
                container.remove_css_class("bar-horizontal");
            } else {
                container.add_css_class("bar-horizontal");
                container.remove_css_class("bar-vertical");
            }

            // Toggle position-specific CSS classes for border direction
            container.remove_css_class("bar-left");
            container.remove_css_class("bar-right");
            container.remove_css_class("bar-top");
            container.remove_css_class("bar-bottom");
            container.add_css_class(&format!("bar-{position}"));

            // Update spacer expand direction
            let mut child_iter = container.first_child();
            while let Some(widget) = child_iter {
                if widget.hexpands() || widget.vexpands() {
                    widget.set_hexpand(!is_vertical);
                    widget.set_vexpand(is_vertical);
                }
                child_iter = widget.next_sibling();
            }
        }
    }
}

pub fn create_bars(
    app: &gtk4::Application,
    client: Option<Rc<HyprlandClient>>,
    config: &Config,
    shared_config: Rc<RefCell<Config>>,
) -> Vec<Bar> {
    let mut bars = Vec::new();
    let all_windows: Rc<RefCell<Vec<gtk4::Window>>> = Rc::new(RefCell::new(Vec::new()));

    if config.bar.monitor.is_empty() {
        // Create bars for all monitors
        let display = gtk4::gdk::Display::default().expect("Could not get default display");
        let monitors = display.monitors();
        let n = monitors.n_items();

        if n == 0 {
            // Fallback: create single bar without specific monitor
            let bar = Bar::new(app, client.clone(), config, shared_config.clone(), None, all_windows.clone());
            bars.push(bar);
        } else {
            for i in 0..n {
                let monitor = monitors
                    .item(i)
                    .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok());

                let bar = Bar::new(app, client.clone(), config, shared_config.clone(), monitor.as_ref(), all_windows.clone());
                bars.push(bar);
            }
        }
    } else {
        // Create bar for specific monitor
        let display = gtk4::gdk::Display::default().expect("Could not get default display");
        let monitors = display.monitors();
        let n = monitors.n_items();

        let mut target_monitor = None;
        for i in 0..n {
            if let Some(mon) = monitors
                .item(i)
                .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
            {
                if mon.connector().map(|c| c.to_string()) == Some(config.bar.monitor.clone()) {
                    target_monitor = Some(mon);
                    break;
                }
            }
        }

        let bar = Bar::new(app, client.clone(), config, shared_config.clone(), target_monitor.as_ref(), all_windows.clone());
        bars.push(bar);
    }

    // Populate shared window list now that all bars are created
    {
        let mut windows = all_windows.borrow_mut();
        for bar in &bars {
            windows.push(bar.window.clone());
        }
    }

    // Setup events after bars are constructed.
    if !bars.is_empty() {
        bars[0].setup_events();
    }

    // Show all bars
    for bar in &bars {
        bar.show();
    }

    bars
}
