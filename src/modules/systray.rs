use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const ITEM_INTERFACE: &str = "org.kde.StatusNotifierItem";

#[derive(Debug, Clone)]
struct TrayItem {
    service: String,
    path: String,
    icon_name: String,
    title: String,
}

enum TrayEvent {
    Added(TrayItem),
    Removed(String),
}

// ── StatusNotifierWatcher D-Bus service ──

struct WatcherService {
    items: Arc<tokio::sync::Mutex<Vec<String>>>,
    sender: async_channel::Sender<TrayEvent>,
    connection: zbus::Connection,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl WatcherService {
    async fn register_status_notifier_item(
        &self,
        service: &str,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) {
        let full_service = if service.starts_with('/') {
            let sender_name = header
                .sender()
                .map(|s| s.to_string())
                .unwrap_or_default();
            format!("{}{}", sender_name, service)
        } else {
            service.to_string()
        };

        let mut items = self.items.lock().await;
        if items.contains(&full_service) {
            return;
        }
        items.push(full_service.clone());
        drop(items);

        let item = fetch_item_properties(&self.connection, &full_service).await;
        let _ = self.sender.send(TrayEvent::Added(item)).await;
    }

    fn register_status_notifier_host(&self, _service: &str) {}

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().await.clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(
        signal_ctxt: &zbus::object_server::SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        signal_ctxt: &zbus::object_server::SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(
        signal_ctxt: &zbus::object_server::SignalContext<'_>,
    ) -> zbus::Result<()>;
}

// ── Systray module ──

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
        rune.set_tooltip_text(Some("\u{16C9} Algiz - System Tray"));
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
        let (sender, receiver) = async_channel::unbounded::<TrayEvent>();

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

                let items = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));

                let watcher = WatcherService {
                    items: items.clone(),
                    sender: sender.clone(),
                    connection: conn.clone(),
                };

                if let Err(e) = conn
                    .object_server()
                    .at("/StatusNotifierWatcher", watcher)
                    .await
                {
                    eprintln!("Warning: Could not serve StatusNotifierWatcher: {e}");
                    // Fall back to client mode
                    run_client_mode(&conn, &sender).await;
                    return;
                }

                match conn
                    .request_name("org.kde.StatusNotifierWatcher")
                    .await
                {
                    Ok(_) => {
                        // We are the watcher — monitor for app exits
                        watch_name_changes(&conn, &items, &sender).await;
                    }
                    Err(_) => {
                        // Another watcher is running (e.g. waybar) — use it as client
                        run_client_mode(&conn, &sender).await;
                    }
                }
            });
        });

        let widget = self.widget.clone();
        let icons = self.icons.clone();
        let items = self.items.clone();

        glib::spawn_future_local(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    TrayEvent::Added(item) => {
                        let key = format!("{}{}", item.service, item.path);
                        let mut current = items.lock().unwrap();
                        if !current.iter().any(|i| format!("{}{}", i.service, i.path) == key) {
                            current.push(item);
                        }
                        drop(current);
                        refresh_tray(&widget, &icons, &items);
                    }
                    TrayEvent::Removed(key) => {
                        items
                            .lock()
                            .unwrap()
                            .retain(|i| format!("{}{}", i.service, i.path) != key);
                        refresh_tray(&widget, &icons, &items);
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        // D-Bus thread stops when the process exits
    }
}

// ── NameOwnerChanged watcher ──

async fn watch_name_changes(
    conn: &zbus::Connection,
    items: &Arc<tokio::sync::Mutex<Vec<String>>>,
    sender: &async_channel::Sender<TrayEvent>,
) {
    use futures_util::StreamExt;

    let Ok(dbus_proxy) = zbus::fdo::DBusProxy::new(conn).await else {
        eprintln!("Warning: Could not create DBus proxy for name watching");
        // Keep the task alive so the watcher stays registered
        std::future::pending::<()>().await;
        return;
    };

    let Ok(mut stream) = dbus_proxy.receive_name_owner_changed().await else {
        eprintln!("Warning: Could not subscribe to NameOwnerChanged");
        std::future::pending::<()>().await;
        return;
    };

    while let Some(signal) = stream.next().await {
        let Ok(args) = signal.args() else {
            continue;
        };

        // Only care when a name vanishes (new_owner is empty)
        let new_owner: &str = args.new_owner().as_deref().unwrap_or("");
        if !new_owner.is_empty() {
            continue;
        }

        let vanished = args.name().as_str();
        let mut lock = items.lock().await;

        // Collect items belonging to the vanished service
        let removed: Vec<String> = lock
            .iter()
            .filter(|item_str| {
                let svc = if let Some(idx) = item_str.find('/') {
                    &item_str[..idx]
                } else {
                    item_str.as_str()
                };
                svc == vanished
            })
            .cloned()
            .collect();

        if removed.is_empty() {
            continue;
        }

        lock.retain(|item_str| {
            let svc = if let Some(idx) = item_str.find('/') {
                &item_str[..idx]
            } else {
                item_str.as_str()
            };
            svc != vanished
        });
        drop(lock);

        for svc in &removed {
            // Reconstruct the key the same way the GTK side does
            let (service_name, path) = if let Some(idx) = svc.find('/') {
                (
                    svc[..idx].to_string(),
                    format!("/{}", &svc[idx + 1..]),
                )
            } else {
                (svc.to_string(), "/StatusNotifierItem".to_string())
            };
            let key = format!("{}{}", service_name, path);
            let _ = sender.send(TrayEvent::Removed(key)).await;
        }
    }
}

// ── Client mode fallback ──

async fn run_client_mode(
    conn: &zbus::Connection,
    sender: &async_channel::Sender<TrayEvent>,
) {
    let Ok(proxy) = zbus::Proxy::new(
        conn,
        "org.kde.StatusNotifierWatcher",
        "/StatusNotifierWatcher",
        "org.kde.StatusNotifierWatcher",
    )
    .await
    else {
        return;
    };

    // Fetch currently registered items
    if let Ok(registered) = proxy
        .get_property::<Vec<String>>("RegisteredStatusNotifierItems")
        .await
    {
        for service_str in registered {
            let item = fetch_item_properties(conn, &service_str).await;
            let _ = sender.send(TrayEvent::Added(item)).await;
        }
    }

    // Listen for new registrations
    let Ok(mut stream) = proxy
        .receive_signal("StatusNotifierItemRegistered")
        .await
    else {
        return;
    };

    use futures_util::StreamExt;
    while let Some(signal) = stream.next().await {
        if let Ok(args) = signal.body().deserialize::<(String,)>() {
            let item = fetch_item_properties(conn, &args.0).await;
            let _ = sender.send(TrayEvent::Added(item)).await;
        }
    }
}

// ── Item property fetcher ──

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

// ── GTK refresh ──

fn refresh_tray(
    widget: &gtk4::Box,
    icons: &Rc<RefCell<HashMap<String, gtk4::Image>>>,
    items: &Arc<Mutex<Vec<TrayItem>>>,
) {
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
}
