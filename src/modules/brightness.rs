use crate::system::brightness;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Brightness {
    pub widget: gtk4::Box,
    label: gtk4::Label,
    slider: gtk4::Scale,
    _available: bool,
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

        // Night mode section
        let night_backend = brightness::detect_night_backend();
        if night_backend != brightness::NightModeBackend::None {
            let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            sep.add_css_class("audio-separator");
            popover_content.append(&sep);

            let night_section = build_night_mode_section();
            popover_content.append(&night_section);
        }

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
            _available: available,
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

fn build_night_mode_section() -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // Header button that toggles the reveal
    let header_btn = gtk4::Button::new();
    header_btn.add_css_class("night-mode-header");

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let rune_label = gtk4::Label::new(Some("\u{16BE}")); // ᚾ Nauthiz
    rune_label.add_css_class("night-mode-header-rune");
    header_box.append(&rune_label);

    let title_label = gtk4::Label::new(Some("Nott's Veil (Night Mode)"));
    title_label.add_css_class("night-mode-header-title");
    title_label.set_hexpand(true);
    title_label.set_halign(gtk4::Align::Start);
    header_box.append(&title_label);

    let arrow = gtk4::Label::new(Some("\u{25B6}")); // ▶
    arrow.add_css_class("night-mode-arrow");
    header_box.append(&arrow);

    header_btn.set_child(Some(&header_box));
    container.append(&header_btn);

    // Revealer
    let revealer = gtk4::Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(200);
    revealer.set_reveal_child(false);

    let reveal_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    reveal_content.set_margin_top(8);
    reveal_content.set_margin_start(4);
    reveal_content.set_margin_end(4);

    // Toggle row
    let toggle_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let toggle_rune = gtk4::Label::new(Some("\u{16BE}")); // ᚾ Nauthiz
    toggle_rune.add_css_class("night-mode-rune");
    toggle_row.append(&toggle_rune);

    let toggle_label = gtk4::Label::new(Some("Night Mode"));
    toggle_label.add_css_class("night-mode-label");
    toggle_label.set_hexpand(true);
    toggle_label.set_halign(gtk4::Align::Start);
    toggle_row.append(&toggle_label);

    let night_switch = gtk4::Switch::new();
    night_switch.add_css_class("night-mode-switch");
    night_switch.set_active(brightness::is_night_mode_active());
    toggle_row.append(&night_switch);

    reveal_content.append(&toggle_row);

    // Temperature row
    let temp_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let temp_label = gtk4::Label::new(Some("Warmth"));
    temp_label.add_css_class("night-mode-label");
    temp_label.set_halign(gtk4::Align::Start);
    temp_row.append(&temp_label);

    let temp_value = gtk4::Label::new(Some("4000K"));
    temp_value.add_css_class("night-temp-value");
    temp_value.set_hexpand(true);
    temp_value.set_halign(gtk4::Align::End);
    temp_row.append(&temp_value);

    reveal_content.append(&temp_row);

    // Temperature slider
    let temp_slider =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 2500.0, 6500.0, 100.0);
    temp_slider.add_css_class("night-temp-slider");
    temp_slider.set_value(4000.0);
    temp_slider.set_hexpand(true);
    temp_slider.set_draw_value(false);

    let night_updating = Rc::new(Cell::new(false));

    // Temperature slider handler
    let temp_value_ref = temp_value.clone();
    let night_updating_clone = night_updating.clone();
    temp_slider.connect_value_changed(move |scale| {
        if night_updating_clone.get() {
            return;
        }
        let temp = scale.value() as i32;
        temp_value_ref.set_text(&format!("{temp}K"));
        brightness::set_night_temperature(temp);
    });

    reveal_content.append(&temp_slider);

    revealer.set_child(Some(&reveal_content));
    container.append(&revealer);

    // Switch handler
    let temp_slider_ref = temp_slider.clone();
    let night_updating_clone2 = night_updating.clone();
    night_switch.connect_state_set(move |_switch, active| {
        if night_updating_clone2.get() {
            return glib::Propagation::Proceed;
        }
        if active {
            let temp = temp_slider_ref.value() as i32;
            let _ = brightness::start_night_mode(temp);
        } else {
            brightness::stop_night_mode();
        }
        glib::Propagation::Proceed
    });

    // Toggle on header click
    let revealer_ref = revealer.clone();
    let arrow_ref = arrow.clone();
    header_btn.connect_clicked(move |btn| {
        let revealed = revealer_ref.reveals_child();
        revealer_ref.set_reveal_child(!revealed);
        if revealed {
            arrow_ref.set_text("\u{25B6}"); // ▶ collapsed
            btn.remove_css_class("expanded");
        } else {
            arrow_ref.set_text("\u{25BC}"); // ▼ expanded
            btn.add_css_class("expanded");
        }
    });

    container
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
