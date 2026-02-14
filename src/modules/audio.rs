use crate::system::audio;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Audio {
    pub widget: gtk4::Box,
    label: gtk4::Label,
    slider: gtk4::Scale,
    mute_btn: gtk4::Button,
    sink_list: gtk4::Box,
    source_list: gtk4::Box,
    available: bool,
    updating: Rc<Cell<bool>>,
    event_listener: Option<audio::AudioEventListener>,
}

impl Audio {
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("audio");

        let label = gtk4::Label::new(Some("--"));
        let updating = Rc::new(Cell::new(false));

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("audio-button");
        menu_button.set_has_frame(false);

        // ᚨ Ansuz - Voice of Odin
        let rune = gtk4::Label::new(Some("\u{16A8}"));
        rune.add_css_class("module-rune");

        let btn_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_content.append(&rune);
        btn_content.append(&label);
        menu_button.set_child(Some(&btn_content));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("audio-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16A8}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Gjallarhorn (Volume)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        // Slider
        let slider = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        slider.add_css_class("audio-slider");
        slider.set_draw_value(true);
        slider.set_value_pos(gtk4::PositionType::Right);
        slider.set_hexpand(true);
        slider.set_size_request(200, -1);

        let updating_clone = updating.clone();
        slider.connect_value_changed(move |scale| {
            if updating_clone.get() {
                return;
            }
            let vol = scale.value() as i32;
            audio::set_volume(vol);
        });
        popover_content.append(&slider);

        // Mute button
        let mute_btn = gtk4::Button::new();
        mute_btn.add_css_class("audio-mute-btn");
        let mute_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let mute_rune = gtk4::Label::new(Some("\u{16C1}")); // ᛁ Isa
        mute_rune.add_css_class("mute-rune");
        let mute_label = gtk4::Label::new(Some("Mute"));
        mute_label.add_css_class("mute-label");
        mute_box.append(&mute_rune);
        mute_box.append(&mute_label);
        mute_box.set_halign(gtk4::Align::Center);
        mute_btn.set_child(Some(&mute_box));

        let label_clone = label.clone();
        let widget_clone = widget.clone();
        let slider_clone = slider.clone();
        let mute_btn_clone = mute_btn.clone();
        let updating_clone2 = updating.clone();
        mute_btn.connect_clicked(move |_| {
            audio::toggle_mute();
            refresh_audio(
                &label_clone,
                &widget_clone,
                &slider_clone,
                &mute_btn_clone,
                &updating_clone2,
            );
        });
        popover_content.append(&mute_btn);

        // Separator before device sections
        let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep1.add_css_class("audio-separator");
        popover_content.append(&sep1);

        // Output devices (sinks) - expandable section
        let sink_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        sink_list.add_css_class("audio-device-list");
        let sink_revealer = create_device_section(
            "\u{16A0}", // ᚠ Fehu
            "Output Devices",
            &sink_list,
        );
        popover_content.append(&sink_revealer);

        // Input devices (sources) - expandable section
        let source_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        source_list.add_css_class("audio-device-list");
        let source_revealer = create_device_section(
            "\u{16D7}", // ᛗ Mannaz
            "Input Devices",
            &source_list,
        );
        popover_content.append(&source_revealer);

        popover.set_child(Some(&popover_content));

        // Rebuild device lists when popover opens
        let sink_list_ref = sink_list.clone();
        let source_list_ref = source_list.clone();
        popover.connect_show(move |_| {
            rebuild_device_lists(&sink_list_ref, &source_list_ref);
        });

        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        let info = audio::get_info();
        let available = info.available;

        if !available {
            widget.set_visible(false);
        }

        let module = Self {
            widget,
            label,
            slider,
            mute_btn,
            sink_list,
            source_list,
            available,
            updating,
            event_listener: None,
        };

        if available {
            module.refresh();
        }

        module
    }

    pub fn setup_events(&mut self) {
        if !self.available {
            return;
        }

        let label = self.label.clone();
        let widget = self.widget.clone();
        let slider = self.slider.clone();
        let mute_btn = self.mute_btn.clone();
        let updating = self.updating.clone();
        let sink_list = self.sink_list.clone();
        let source_list = self.source_list.clone();

        let (sender, receiver) = async_channel::unbounded::<()>();

        glib::spawn_future_local(async move {
            while receiver.recv().await.is_ok() {
                refresh_audio(&label, &widget, &slider, &mute_btn, &updating);
                rebuild_device_lists(&sink_list, &source_list);
            }
        });

        let listener = audio::AudioEventListener::new();
        listener.start(sender);
        self.event_listener = Some(listener);
    }

    fn refresh(&self) {
        refresh_audio(
            &self.label,
            &self.widget,
            &self.slider,
            &self.mute_btn,
            &self.updating,
        );
        rebuild_device_lists(&self.sink_list, &self.source_list);
    }

    pub fn stop(&self) {
        if let Some(listener) = &self.event_listener {
            listener.stop();
        }
    }
}

/// Creates an expandable device section with an arrow toggle button and a revealer.
fn create_device_section(
    rune_char: &str,
    title: &str,
    device_list: &gtk4::Box,
) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // Header button that toggles the reveal
    let header_btn = gtk4::Button::new();
    header_btn.add_css_class("audio-device-header");

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let rune_label = gtk4::Label::new(Some(rune_char));
    rune_label.add_css_class("audio-device-rune");
    header_box.append(&rune_label);

    let title_label = gtk4::Label::new(Some(title));
    title_label.add_css_class("audio-device-title");
    title_label.set_hexpand(true);
    title_label.set_halign(gtk4::Align::Start);
    header_box.append(&title_label);

    let arrow = gtk4::Label::new(Some("\u{25B6}")); // ▶ right-pointing arrow
    arrow.add_css_class("audio-device-arrow");
    header_box.append(&arrow);

    header_btn.set_child(Some(&header_box));
    container.append(&header_btn);

    // Revealer for smooth expand/collapse
    let revealer = gtk4::Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(200);
    revealer.set_reveal_child(false);

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_max_content_height(150);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(device_list));

    revealer.set_child(Some(&scroll));
    container.append(&revealer);

    // Toggle on click
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

fn refresh_audio(
    label: &gtk4::Label,
    widget: &gtk4::Box,
    slider: &gtk4::Scale,
    mute_btn: &gtk4::Button,
    updating: &Rc<Cell<bool>>,
) {
    let info = audio::get_info();
    if !info.available {
        label.set_text("--");
        return;
    }

    widget.remove_css_class("muted");
    if info.muted {
        label.set_text("Muted");
        widget.add_css_class("muted");
    } else {
        label.set_text(&format!("{}%", info.volume));
    }

    updating.set(true);
    slider.set_value(info.volume as f64);
    updating.set(false);

    if info.muted {
        mute_btn.add_css_class("muted");
    } else {
        mute_btn.remove_css_class("muted");
    }

    widget.set_tooltip_text(Some(&format!(
        "Volume: {}%\nClick to adjust",
        info.volume
    )));
}

fn rebuild_device_lists(sink_list: &gtk4::Box, source_list: &gtk4::Box) {
    // Clear existing children
    while let Some(child) = sink_list.first_child() {
        sink_list.remove(&child);
    }
    while let Some(child) = source_list.first_child() {
        source_list.remove(&child);
    }

    // Populate sinks
    let sinks = audio::list_sinks();
    for device in &sinks {
        let btn = gtk4::Button::new();
        btn.add_css_class("audio-device-btn");

        let label = gtk4::Label::new(Some(&device.description));
        label.set_halign(gtk4::Align::Start);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        btn.set_child(Some(&label));

        if device.is_default {
            btn.add_css_class("active");
        }

        let name = device.name.clone();
        let sink_list_ref = sink_list.clone();
        let source_list_ref = source_list.clone();
        btn.connect_clicked(move |_| {
            audio::set_default_sink(&name);
            rebuild_device_lists(&sink_list_ref, &source_list_ref);
        });

        sink_list.append(&btn);
    }

    // Populate sources
    let sources = audio::list_sources();
    for device in &sources {
        let btn = gtk4::Button::new();
        btn.add_css_class("audio-device-btn");

        let label = gtk4::Label::new(Some(&device.description));
        label.set_halign(gtk4::Align::Start);
        label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        btn.set_child(Some(&label));

        if device.is_default {
            btn.add_css_class("active");
        }

        let name = device.name.clone();
        let sink_list_ref = sink_list.clone();
        let source_list_ref = source_list.clone();
        btn.connect_clicked(move |_| {
            audio::set_default_source(&name);
            rebuild_device_lists(&sink_list_ref, &source_list_ref);
        });

        source_list.append(&btn);
    }
}
