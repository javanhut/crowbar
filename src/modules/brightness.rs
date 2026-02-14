use crate::system::brightness;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Brightness {
    pub widget: gtk4::Box,
    label: gtk4::Label,
    slider: gtk4::Scale,
    available: bool,
    device: String,
    updating: Rc<Cell<bool>>,
    source_id: Option<glib::SourceId>,
}

impl Brightness {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("brightness");

        let label = gtk4::Label::new(Some("--"));
        let updating = Rc::new(Cell::new(false));

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("brightness-button");
        menu_button.set_has_frame(false);

        // ᛊ Sowilo - Sun
        let rune = gtk4::Label::new(Some("\u{16CA}"));
        rune.add_css_class("module-rune");

        let btn_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_content.append(&rune);
        btn_content.append(&label);
        menu_button.set_child(Some(&btn_content));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("brightness-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16CA}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Sunlight (Brightness)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        let slider = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 100.0, 1.0);
        slider.add_css_class("brightness-slider");
        slider.set_draw_value(true);
        slider.set_value_pos(gtk4::PositionType::Right);
        slider.set_hexpand(true);
        slider.set_size_request(200, -1);

        let devices = brightness::find_backlights();
        let available = !devices.is_empty();
        let device = devices.first().cloned().unwrap_or_default();

        let updating_clone = updating.clone();
        let device_clone = device.clone();
        slider.connect_value_changed(move |scale| {
            if updating_clone.get() {
                return;
            }
            let val = scale.value() as i32;
            brightness::set_brightness(&device_clone, val);
        });
        popover_content.append(&slider);

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        if !available {
            widget.set_visible(false);
        }

        let mut module = Self {
            widget,
            label,
            slider,
            available,
            device,
            updating,
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
        let slider = self.slider.clone();
        let widget = self.widget.clone();
        let device = self.device.clone();
        let updating = self.updating.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_brightness(&label, &slider, &widget, &device, &updating);
            glib::ControlFlow::Continue
        }));
    }

    fn refresh(&self) {
        refresh_brightness(
            &self.label,
            &self.slider,
            &self.widget,
            &self.device,
            &self.updating,
        );
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn refresh_brightness(
    label: &gtk4::Label,
    slider: &gtk4::Scale,
    widget: &gtk4::Box,
    device: &str,
    updating: &Rc<Cell<bool>>,
) {
    let Some(info) = brightness::get_info(device) else {
        label.set_text("--");
        return;
    };

    label.set_text(&format!("{}%", info.percent));

    updating.set(true);
    slider.set_value(info.percent as f64);
    updating.set(false);

    widget.set_tooltip_text(Some(&format!(
        "Brightness: {}%\nDevice: {}\nClick to adjust",
        info.percent, info.device
    )));
}
