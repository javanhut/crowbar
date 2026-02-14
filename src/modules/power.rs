use crate::system::power;
use gtk4::glib;
use gtk4::prelude::*;

pub struct Power {
    pub widget: gtk4::Box,
    label: gtk4::Label,
    source_id: Option<glib::SourceId>,
}

impl Power {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        widget.add_css_class("power");

        let label = gtk4::Label::new(Some("--"));

        // ᚢ Uruz - Power
        let rune = gtk4::Label::new(Some("\u{16A2}"));
        rune.add_css_class("module-rune");
        rune.set_tooltip_text(Some("\u{16A2} Uruz - Power"));

        widget.append(&rune);
        widget.append(&label);

        let mut module = Self {
            widget,
            label,
            source_id: None,
        };

        module.refresh();
        module.start_updates(interval_secs);
        module
    }

    fn start_updates(&mut self, interval_secs: u32) {
        let label = self.label.clone();
        let widget = self.widget.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_power(&label, &widget);
            glib::ControlFlow::Continue
        }));
    }

    fn refresh(&self) {
        refresh_power(&self.label, &self.widget);
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn refresh_power(label: &gtk4::Label, widget: &gtk4::Box) {
    let info = power::get_info();

    let label_text = if info.has_temp {
        power::format_temperature(info.temperature)
    } else {
        info.governor.display_name().to_string()
    };
    label.set_text(&label_text);

    widget.remove_css_class("hot");
    widget.remove_css_class("critical");
    if info.has_temp {
        if info.temperature >= 85.0 {
            widget.add_css_class("critical");
        } else if info.temperature >= 70.0 {
            widget.add_css_class("hot");
        }
    }

    let mut tooltip = format!("Governor: {}", info.governor.display_name());
    if info.has_temp {
        tooltip += &format!("\nCPU: {}", power::format_temperature(info.temperature));
    }
    if info.frequency_mhz > 0 {
        if info.frequency_mhz >= 1000 {
            tooltip += &format!("\nFrequency: {:.1} GHz", info.frequency_mhz as f64 / 1000.0);
        } else {
            tooltip += &format!("\nFrequency: {} MHz", info.frequency_mhz);
        }
    }
    widget.set_tooltip_text(Some(&tooltip));
}
