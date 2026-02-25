use crate::system::battery::{self, BatteryStatus};
use gtk4::glib;
use gtk4::prelude::*;

pub struct Battery {
    pub widget: gtk4::Box,
    label: gtk4::Label,
    _available: bool,
    source_id: Option<glib::SourceId>,
}

impl Battery {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        widget.add_css_class("battery");

        let label = gtk4::Label::new(Some("--"));

        // ᚠ Fehu - Energy
        let rune = gtk4::Label::new(Some("\u{16A0}"));
        rune.add_css_class("module-rune");
        rune.set_tooltip_text(Some("\u{16A0} Fehu - Energy"));

        widget.append(&rune);
        widget.append(&label);

        let batteries = battery::find_batteries();
        let available = !batteries.is_empty();

        if !available {
            widget.set_visible(false);
        }

        let mut module = Self {
            widget,
            label,
            _available: available,
            source_id: None,
        };

        if available {
            module.refresh();
            module.start_updates(interval_secs);
        }

        module
    }

    fn start_updates(&mut self, interval_secs: u32) {
        let label = self.label.clone();
        let widget = self.widget.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_battery(&label, &widget);
            glib::ControlFlow::Continue
        }));
    }

    fn refresh(&self) {
        refresh_battery(&self.label, &self.widget);
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn refresh_battery(label: &gtk4::Label, widget: &gtk4::Box) {
    let Some(info) = battery::get_first_battery() else {
        label.set_text("--");
        return;
    };

    label.set_text(&format!("{}%", info.capacity));

    widget.remove_css_class("charging");
    widget.remove_css_class("low");
    widget.remove_css_class("critical");

    match info.status {
        BatteryStatus::Charging => widget.add_css_class("charging"),
        _ if info.capacity <= 10 => widget.add_css_class("critical"),
        _ if info.capacity <= 20 => widget.add_css_class("low"),
        _ => {}
    }

    let status_str = match &info.status {
        BatteryStatus::Charging => "Charging",
        BatteryStatus::Discharging => "Discharging",
        BatteryStatus::Full => "Fully charged",
        BatteryStatus::NotCharging => "Not charging",
        BatteryStatus::Unknown => "Unknown",
    };

    let mut tooltip = format!("{}% - {}", info.capacity, status_str);
    let time_str = battery::format_time_remaining(info.time_remaining);
    if !time_str.is_empty() {
        match info.status {
            BatteryStatus::Charging => tooltip += &format!("\n{time_str} until full"),
            BatteryStatus::Discharging => tooltip += &format!("\n{time_str} remaining"),
            _ => {}
        }
    }
    widget.set_tooltip_text(Some(&tooltip));
}
