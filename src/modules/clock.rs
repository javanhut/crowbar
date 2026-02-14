use gtk4::glib;
use gtk4::prelude::*;

pub struct Clock {
    pub widget: gtk4::Box,
    time_label: gtk4::Label,
    date_label: gtk4::Label,
    source_id: Option<glib::SourceId>,
}

impl Clock {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        widget.add_css_class("clock");

        let time_label = gtk4::Label::new(None);
        time_label.add_css_class("clock-time");

        let date_label = gtk4::Label::new(None);
        date_label.add_css_class("clock-date");

        // ᛃ Jera - cycles of time
        let rune_left = gtk4::Label::new(Some("\u{16C3}"));
        rune_left.add_css_class("clock-rune");

        // ᛞ Dagaz - day
        let rune_right = gtk4::Label::new(Some("\u{16DE}"));
        rune_right.add_css_class("clock-rune");

        widget.append(&rune_left);
        widget.append(&date_label);
        widget.append(&time_label);
        widget.append(&rune_right);

        let mut clock = Self {
            widget,
            time_label,
            date_label,
            source_id: None,
        };

        clock.refresh();
        clock.start_updates(interval_secs);
        clock
    }

    fn start_updates(&mut self, interval_secs: u32) {
        let time_label = self.time_label.clone();
        let date_label = self.date_label.clone();
        let widget = self.widget.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_clock(&time_label, &date_label, &widget);
            glib::ControlFlow::Continue
        }));
    }

    fn refresh(&self) {
        refresh_clock(&self.time_label, &self.date_label, &self.widget);
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn refresh_clock(time_label: &gtk4::Label, date_label: &gtk4::Label, widget: &gtk4::Box) {
    let now = glib::DateTime::now_local().unwrap();

    let time_str = now.format("%H:%M").unwrap_or_default();
    time_label.set_text(&time_str);

    let date_str = now.format("%a, %b %e").unwrap_or_default();
    date_label.set_text(&date_str);

    let tooltip = now
        .format("%A, %B %e, %Y\nWeek %V")
        .unwrap_or_default();
    widget.set_tooltip_text(Some(&tooltip));
}
