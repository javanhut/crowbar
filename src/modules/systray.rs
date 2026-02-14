use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const WATCHER_SERVICE: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const WATCHER_INTERFACE: &str = "org.kde.StatusNotifierWatcher";
const ITEM_INTERFACE: &str = "org.kde.StatusNotifierItem";

#[derive(Debug, Clone)]
struct TrayItem {
    service: String,
    path: String,
    icon_name: String,
    title: String,
}

pub struct Systray {
    pub widget: gtk4::Box,
    icons: Rc<RefCell<HashMap<String, gtk4::Image>>>,
    items: Arc<Mutex<Vec<TrayItem>>>,
}

impl Systray {
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        widget.add_css_class("systray");

        // ᛉ Algiz - Protection
        let rune = gtk4::Label::new(Some("\u{16C9}"));
        rune.add_css_class("module-rune");
        rune.set_tooltip_text(Some("\u{16C9} Algiz - Protection"));
        widget.append(&rune);

        let items = Arc::new(Mutex::new(Vec::new()));

        let mut systray = Self {
            widget,
            icons: Rc::new(RefCell::new(HashMap::new())),
            items,
        };

        systray.start_dbus_listener();
        systray
    }

    fn start_dbus_listener(&mut self) {
        let items = self.items.clone();
        let (sender, receiver) = async_channel::unbounded::<()>();

        // Spawn background thread for D-Bus
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Warning: Could not create tokio runtime for systray: {e}");
                    return;
                }
            };

            rt.block_on(async move {
                let Ok(conn) = zbus::Connection::session().await else {
                    eprintln!("Warning: Could not connect to session bus for systray");
                    return;
                };

                // Try to get registered items
                let proxy = zbus::Proxy::new(
                    &conn,
                    WATCHER_SERVICE,
                    WATCHER_PATH,
                    WATCHER_INTERFACE,
                )
                .await;

                let Ok(proxy) = proxy else {
                    return;
                };

                // Get currently registered items
                if let Ok(registered) = proxy
                    .get_property::<Vec<String>>("RegisteredStatusNotifierItems")
                    .await
                {
                    let mut new_items = Vec::new();
                    for service_str in registered {
                        let item = fetch_item_properties(&conn, &service_str).await;
                        new_items.push(item);
                    }
                    *items.lock().unwrap() = new_items;
                    let _ = sender.send_blocking(());
                }

                // Listen for signals
                let stream = proxy
                    .receive_signal("StatusNotifierItemRegistered")
                    .await;

                if let Ok(mut stream) = stream {
                    use futures_util::StreamExt;
                    while let Some(signal) = stream.next().await {
                        if let Ok(args) = signal.body().deserialize::<(String,)>() {
                            let item = fetch_item_properties(&conn, &args.0).await;
                            items.lock().unwrap().push(item);
                            let _ = sender.send_blocking(());
                        }
                    }
                }
            });
        });

        let widget = self.widget.clone();
        let icons = self.icons.clone();
        let items = self.items.clone();

        glib::spawn_future_local(async move {
            while receiver.recv().await.is_ok() {
                let _ = refresh_tray(&widget, &icons, &items);
            }
        });
    }

    pub fn stop(&self) {
        // D-Bus thread will stop when connection is dropped
    }
}

async fn fetch_item_properties(conn: &zbus::Connection, service_str: &str) -> TrayItem {
    let (service_name, path) = if let Some(idx) = service_str.find('/') {
        (
            service_str[..idx].to_string(),
            format!("/{}", &service_str[idx + 1..]),
        )
    } else {
        (service_str.to_string(), "/StatusNotifierItem".to_string())
    };

    let mut item = TrayItem {
        service: service_name.clone(),
        path: path.clone(),
        icon_name: String::new(),
        title: String::new(),
    };

    if let Ok(proxy) = zbus::Proxy::new(
        conn,
        service_name.as_str(),
        path.as_str(),
        ITEM_INTERFACE,
    )
    .await
    {
        if let Ok(icon) = proxy.get_property::<String>("IconName").await {
            item.icon_name = icon;
        }
        if let Ok(title) = proxy.get_property::<String>("Title").await {
            item.title = title;
        }
        if item.title.is_empty() {
            if let Ok(id) = proxy.get_property::<String>("Id").await {
                item.title = id;
            }
        }
    }

    item
}

fn refresh_tray(
    widget: &gtk4::Box,
    icons: &Rc<RefCell<HashMap<String, gtk4::Image>>>,
    items: &Arc<Mutex<Vec<TrayItem>>>,
) -> bool {
    let current_items = items.lock().unwrap().clone();
    let mut present = std::collections::HashSet::new();
    let mut icons = icons.borrow_mut();

    for item in &current_items {
        let key = format!("{}{}", item.service, item.path);
        present.insert(key.clone());

        if !icons.contains_key(&key) {
            let icon = if item.icon_name.is_empty() {
                gtk4::Image::from_icon_name("application-x-executable-symbolic")
            } else {
                gtk4::Image::from_icon_name(&item.icon_name)
            };
            icon.add_css_class("systray-icon");
            icon.set_pixel_size(16);

            let tooltip = if item.title.is_empty() {
                &item.service
            } else {
                &item.title
            };
            icon.set_tooltip_text(Some(tooltip));

            widget.append(&icon);
            icons.insert(key, icon);
        }
    }

    // Remove gone items
    let to_remove: Vec<String> = icons
        .keys()
        .filter(|k| !present.contains(*k))
        .cloned()
        .collect();
    for key in to_remove {
        if let Some(icon) = icons.remove(&key) {
            widget.remove(&icon);
        }
    }

    widget.set_visible(!icons.is_empty());
    true
}
