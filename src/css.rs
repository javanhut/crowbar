use crate::config::{ThemeColors, ThemeConfig};
use gtk4::gdk;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;

pub fn load_css() -> Option<PathBuf> {
    let provider = gtk4::CssProvider::new();

    let css_path = find_css();
    let Some(path) = css_path else {
        eprintln!("Warning: Could not find style.css");
        return None;
    };

    provider.load_from_path(path.to_str().unwrap_or(""));

    let display = gdk::Display::default().expect("Could not get default display");
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    Some(path)
}

pub fn start_css_watcher(css_path: PathBuf) {
    let (tx, rx) = mpsc::channel();

    let mut watcher: RecommendedWatcher =
        match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                ) {
                    let _ = tx.send(());
                }
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Warning: Could not create CSS watcher: {e}");
                return;
            }
        };

    let watch_path = css_path.parent().unwrap_or(&css_path).to_path_buf();
    if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
        eprintln!("Warning: Could not watch CSS path: {e}");
        return;
    }

    let css_path_clone = css_path.clone();
    let (sender, receiver) = async_channel::unbounded::<()>();

    std::thread::spawn(move || {
        let _watcher = watcher; // keep alive
        while rx.recv().is_ok() {
            let _ = sender.send_blocking(());
            // Debounce: drain any queued events
            while rx.try_recv().is_ok() {}
        }
    });

    gtk4::glib::spawn_future_local(async move {
        while receiver.recv().await.is_ok() {
            let provider = gtk4::CssProvider::new();
            provider.load_from_path(css_path_clone.to_str().unwrap_or(""));

            let display = gdk::Display::default().expect("Could not get default display");
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });
}

pub fn apply_theme(theme: &ThemeConfig) {
    let colors = if theme.preset == "custom" {
        theme.colors.clone()
    } else {
        ThemeColors::for_preset(&theme.preset)
    };

    let css = format!(
        r#"
@define-color void_deep {};
@define-color void_dark {};
@define-color void_mid {};
@define-color void_light {};
@define-color frost_dark {};
@define-color frost_mid {};
@define-color frost_light {};
@define-color bifrost_blue {};
@define-color bifrost_cyan {};
@define-color bifrost_teal {};
@define-color bifrost_purple {};
@define-color fire_orange {};
@define-color fire_red {};
@define-color fire_ember {};
@define-color leaf_green {};
@define-color mead_gold {};
@define-color bark_brown {};
@define-color starlight {};
@define-color moonlight {};
@define-color sunlight {};
"#,
        colors.void_deep,
        colors.void_dark,
        colors.void_mid,
        colors.void_light,
        colors.frost_dark,
        colors.frost_mid,
        colors.frost_light,
        colors.bifrost_blue,
        colors.bifrost_cyan,
        colors.bifrost_teal,
        colors.bifrost_purple,
        colors.fire_orange,
        colors.fire_red,
        colors.fire_ember,
        colors.leaf_green,
        colors.mead_gold,
        colors.bark_brown,
        colors.starlight,
        colors.moonlight,
        colors.sunlight,
    );

    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&css);

    let display = gdk::Display::default().expect("Could not get default display");
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );
}

fn find_css() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    let locations = [
        PathBuf::from("style.css"),
        exe_dir.join("style.css"),
        PathBuf::from(format!("{home}/.config/crowbar/style.css")),
        PathBuf::from(format!("{home}/.local/share/crowbar/style.css")),
        PathBuf::from("/usr/local/share/crowbar/style.css"),
        PathBuf::from("/usr/share/crowbar/style.css"),
    ];

    for loc in &locations {
        if loc.exists() {
            return Some(loc.clone());
        }
    }
    None
}
