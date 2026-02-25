use gtk4::prelude::*;
use std::process::Command;

pub struct PowerMenu {
    pub widget: gtk4::MenuButton,
}

impl PowerMenu {
    pub fn new() -> Self {
        let button = gtk4::MenuButton::new();
        button.add_css_class("power-button");
        button.set_tooltip_text(Some("Power Menu - \u{16A6} Thurisaz"));

        // ᚦ Thurisaz rune
        let rune_label = gtk4::Label::new(Some("\u{16A6}"));
        rune_label.add_css_class("power-rune");
        button.set_child(Some(&rune_label));

        // Create popover menu
        let popover = gtk4::Popover::new();
        popover.add_css_class("power-menu");
        popover.set_autohide(true);

        let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        menu_box.add_css_class("power-menu-content");
        menu_box.set_margin_top(8);
        menu_box.set_margin_bottom(8);
        menu_box.set_margin_start(8);
        menu_box.set_margin_end(8);

        // Lock - ᛁ Isa
        let popover_ref = popover.clone();
        let lock_btn = create_menu_item("\u{16C1}", "Lock", move || {
            popover_ref.popdown();
            let _ = Command::new("loginctl").arg("lock-session").spawn();
        });
        menu_box.append(&lock_btn);

        // Logout - ᚱ Raidho
        let popover_ref = popover.clone();
        let logout_btn = create_menu_item("\u{16B1}", "Logout", move || {
            popover_ref.popdown();
            let _ = Command::new("hyprctl").args(["dispatch", "exit"]).spawn();
        });
        menu_box.append(&logout_btn);

        // Separator
        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep.add_css_class("power-menu-separator");
        menu_box.append(&sep);

        // Suspend - ᚾ Nauthiz
        let popover_ref = popover.clone();
        let suspend_btn = create_menu_item("\u{16BE}", "Suspend", move || {
            popover_ref.popdown();
            let _ = Command::new("systemctl").arg("suspend").spawn();
        });
        menu_box.append(&suspend_btn);

        // Reboot - ᛟ Othala
        let popover_ref = popover.clone();
        let reboot_btn = create_menu_item("\u{16DF}", "Reboot", move || {
            popover_ref.popdown();
            let _ = Command::new("systemctl").arg("reboot").spawn();
        });
        menu_box.append(&reboot_btn);

        // Shutdown - ᚺ Hagalaz
        let popover_ref = popover.clone();
        let shutdown_btn = create_menu_item("\u{16BA}", "Shutdown", move || {
            popover_ref.popdown();
            let _ = Command::new("systemctl").arg("poweroff").spawn();
        });
        shutdown_btn.add_css_class("power-menu-shutdown");
        menu_box.append(&shutdown_btn);

        popover.set_child(Some(&menu_box));
        button.set_popover(Some(&popover));

        Self { widget: button }
    }
}

fn create_menu_item(rune: &str, label: &str, on_click: impl Fn() + 'static) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("power-menu-item");

    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

    let rune_label = gtk4::Label::new(Some(rune));
    rune_label.add_css_class("power-menu-rune");

    let text_label = gtk4::Label::new(Some(label));
    text_label.add_css_class("power-menu-label");
    text_label.set_halign(gtk4::Align::Start);
    text_label.set_hexpand(true);

    box_.append(&rune_label);
    box_.append(&text_label);
    btn.set_child(Some(&box_));

    btn.connect_clicked(move |_| on_click());

    btn
}
