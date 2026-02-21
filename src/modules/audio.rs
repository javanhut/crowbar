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
    source_slider: gtk4::Scale,
    source_mute_btn: gtk4::Button,
    source_label: gtk4::Label,
    source_updating: Rc<Cell<bool>>,
    app_streams_list: gtk4::Box,
    card_profiles_list: gtk4::Box,
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
        let source_updating = Rc::new(Cell::new(false));

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("audio-button");
        menu_button.set_has_frame(false);

        // Ansuz - Voice of Odin
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

        // === Section 1: Output Volume (existing) ===
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16A8}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Gjallarhorn (Volume)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

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
        let mute_rune = gtk4::Label::new(Some("\u{16C1}")); // Isa
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

        // === Section 2: Input Volume (NEW) ===
        let sep_source = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep_source.add_css_class("audio-separator");
        popover_content.append(&sep_source);

        let source_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let source_header_rune = gtk4::Label::new(Some("\u{16D7}")); // Mannaz
        source_header_rune.add_css_class("slider-header-rune");
        let source_header_label = gtk4::Label::new(Some("Heimdall's Ear (Microphone)"));
        source_header_label.add_css_class("slider-header-label");
        source_header.append(&source_header_rune);
        source_header.append(&source_header_label);

        let source_label = gtk4::Label::new(Some(""));
        source_label.add_css_class("source-volume-label");
        source_header.append(&source_label);

        popover_content.append(&source_header);

        let source_slider =
            gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 150.0, 1.0);
        source_slider.add_css_class("audio-slider");
        source_slider.add_css_class("source-slider");
        source_slider.set_draw_value(true);
        source_slider.set_value_pos(gtk4::PositionType::Right);
        source_slider.set_hexpand(true);
        source_slider.set_size_request(200, -1);

        let source_updating_clone = source_updating.clone();
        source_slider.connect_value_changed(move |scale| {
            if source_updating_clone.get() {
                return;
            }
            let vol = scale.value() as i32;
            audio::set_source_volume(vol);
        });
        popover_content.append(&source_slider);

        // Source mute button
        let source_mute_btn = gtk4::Button::new();
        source_mute_btn.add_css_class("audio-mute-btn");
        source_mute_btn.add_css_class("source-mute-btn");
        let src_mute_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let src_mute_rune = gtk4::Label::new(Some("\u{16C1}")); // Isa
        src_mute_rune.add_css_class("mute-rune");
        let src_mute_label = gtk4::Label::new(Some("Mute Mic"));
        src_mute_label.add_css_class("mute-label");
        src_mute_box.append(&src_mute_rune);
        src_mute_box.append(&src_mute_label);
        src_mute_box.set_halign(gtk4::Align::Center);
        source_mute_btn.set_child(Some(&src_mute_box));

        let source_slider_clone = source_slider.clone();
        let source_mute_btn_clone = source_mute_btn.clone();
        let source_updating_clone2 = source_updating.clone();
        let source_label_clone = source_label.clone();
        source_mute_btn.connect_clicked(move |_| {
            audio::toggle_source_mute();
            refresh_source_audio(
                &source_slider_clone,
                &source_mute_btn_clone,
                &source_updating_clone2,
                &source_label_clone,
            );
        });
        popover_content.append(&source_mute_btn);

        // === Section 3: App Streams (NEW) ===
        let sep_streams = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep_streams.add_css_class("audio-separator");
        popover_content.append(&sep_streams);

        let app_streams_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        app_streams_list.add_css_class("audio-app-list");
        let streams_section = create_device_section(
            "\u{16B1}", // Raido
            "Streams of Yggdrasil (App Mixer)",
            &app_streams_list,
            200,
        );
        popover_content.append(&streams_section);

        // === Section 4: Output Devices (existing) ===
        let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep1.add_css_class("audio-separator");
        popover_content.append(&sep1);

        let sink_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        sink_list.add_css_class("audio-device-list");
        let sink_revealer = create_device_section(
            "\u{16A0}", // Fehu
            "Output Devices",
            &sink_list,
            150,
        );
        popover_content.append(&sink_revealer);

        // === Section 5: Input Devices (existing) ===
        let source_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        source_list.add_css_class("audio-device-list");
        let source_revealer = create_device_section(
            "\u{16D7}", // Mannaz
            "Input Devices",
            &source_list,
            150,
        );
        popover_content.append(&source_revealer);

        // === Section 6: Audio Profiles (NEW) ===
        let sep_profiles = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep_profiles.add_css_class("audio-separator");
        popover_content.append(&sep_profiles);

        let card_profiles_list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        card_profiles_list.add_css_class("audio-card-list");
        let profiles_section = create_device_section(
            "\u{16B7}", // Gebo
            "Forge Profiles (Audio Cards)",
            &card_profiles_list,
            200,
        );
        popover_content.append(&profiles_section);

        popover.set_child(Some(&popover_content));

        // Rebuild all dynamic lists when popover opens
        let sink_list_ref = sink_list.clone();
        let source_list_ref = source_list.clone();
        let app_streams_ref = app_streams_list.clone();
        let card_profiles_ref = card_profiles_list.clone();
        let source_slider_ref = source_slider.clone();
        let source_mute_ref = source_mute_btn.clone();
        let source_updating_ref = source_updating.clone();
        let source_label_ref = source_label.clone();
        popover.connect_show(move |_| {
            rebuild_device_lists(&sink_list_ref, &source_list_ref);
            rebuild_app_streams(&app_streams_ref);
            rebuild_card_profiles(&card_profiles_ref);
            refresh_source_audio(
                &source_slider_ref,
                &source_mute_ref,
                &source_updating_ref,
                &source_label_ref,
            );
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
            source_slider,
            source_mute_btn,
            source_label,
            source_updating,
            app_streams_list,
            card_profiles_list,
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
        let source_slider = self.source_slider.clone();
        let source_mute_btn = self.source_mute_btn.clone();
        let source_updating = self.source_updating.clone();
        let source_label = self.source_label.clone();
        let app_streams_list = self.app_streams_list.clone();
        let card_profiles_list = self.card_profiles_list.clone();

        let (sender, receiver) = async_channel::unbounded::<()>();

        glib::spawn_future_local(async move {
            while receiver.recv().await.is_ok() {
                refresh_audio(&label, &widget, &slider, &mute_btn, &updating);
                refresh_source_audio(
                    &source_slider,
                    &source_mute_btn,
                    &source_updating,
                    &source_label,
                );
                rebuild_device_lists(&sink_list, &source_list);
                rebuild_app_streams(&app_streams_list);
                rebuild_card_profiles(&card_profiles_list);
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
        refresh_source_audio(
            &self.source_slider,
            &self.source_mute_btn,
            &self.source_updating,
            &self.source_label,
        );
        rebuild_device_lists(&self.sink_list, &self.source_list);
        rebuild_app_streams(&self.app_streams_list);
        rebuild_card_profiles(&self.card_profiles_list);
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
    max_height: i32,
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

    let arrow = gtk4::Label::new(Some("\u{25B6}")); // right-pointing arrow
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
    scroll.set_max_content_height(max_height);
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
            arrow_ref.set_text("\u{25B6}"); // collapsed
            btn.remove_css_class("expanded");
        } else {
            arrow_ref.set_text("\u{25BC}"); // expanded
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

fn refresh_source_audio(
    slider: &gtk4::Scale,
    mute_btn: &gtk4::Button,
    updating: &Rc<Cell<bool>>,
    label: &gtk4::Label,
) {
    let info = audio::get_source_info();
    if !info.available {
        label.set_text("");
        return;
    }

    updating.set(true);
    slider.set_value(info.volume as f64);
    updating.set(false);

    if info.muted {
        mute_btn.add_css_class("muted");
        label.set_text("Muted");
    } else {
        mute_btn.remove_css_class("muted");
        label.set_text(&format!("{}%", info.volume));
    }
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

fn rebuild_app_streams(app_streams_list: &gtk4::Box) {
    // Clear existing children
    while let Some(child) = app_streams_list.first_child() {
        app_streams_list.remove(&child);
    }

    let sink_inputs = audio::list_sink_inputs();

    if sink_inputs.is_empty() {
        let empty_label = gtk4::Label::new(Some("No active streams"));
        empty_label.add_css_class("audio-app-empty");
        app_streams_list.append(&empty_label);
        return;
    }

    let sinks = audio::list_sinks();
    let has_multiple_sinks = sinks.len() > 1;

    for input in &sink_inputs {
        let row = build_app_stream_row(input, &sinks, has_multiple_sinks, app_streams_list);
        app_streams_list.append(&row);
    }
}

fn build_app_stream_row(
    input: &audio::SinkInput,
    sinks: &[audio::AudioDevice],
    has_multiple_sinks: bool,
    app_streams_list: &gtk4::Box,
) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    row.add_css_class("audio-app-row");

    // Top line: app name + mute toggle
    let top_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let name_label = gtk4::Label::new(Some(&input.name));
    name_label.add_css_class("audio-app-name");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_hexpand(true);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name_label.set_max_width_chars(20);
    top_box.append(&name_label);

    let mute_toggle = gtk4::Button::new();
    mute_toggle.add_css_class("audio-app-mute");
    if input.muted {
        mute_toggle.set_label("\u{16C1}"); // Isa rune for muted
        mute_toggle.add_css_class("muted");
    } else {
        mute_toggle.set_label("\u{16A8}"); // Ansuz rune for unmuted
    }

    let idx = input.index;
    let muted = input.muted;
    let streams_ref = app_streams_list.clone();
    mute_toggle.connect_clicked(move |_| {
        let toggle_val = if muted { "0" } else { "1" };
        audio::set_sink_input_mute(idx, toggle_val);
        rebuild_app_streams(&streams_ref);
    });
    top_box.append(&mute_toggle);

    row.append(&top_box);

    // Volume slider
    let vol_slider =
        gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 150.0, 1.0);
    vol_slider.add_css_class("audio-slider");
    vol_slider.add_css_class("audio-app-slider");
    vol_slider.set_draw_value(true);
    vol_slider.set_value_pos(gtk4::PositionType::Right);
    vol_slider.set_hexpand(true);
    vol_slider.set_value(input.volume as f64);

    let idx = input.index;
    vol_slider.connect_value_changed(move |scale| {
        let vol = scale.value() as i32;
        audio::set_sink_input_volume(idx, vol);
    });
    row.append(&vol_slider);

    // Routing dropdown (only if multiple sinks)
    if has_multiple_sinks {
        let descriptions: Vec<String> = sinks.iter().map(|s| s.description.clone()).collect();
        let str_list = gtk4::StringList::new(&descriptions.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let dropdown = gtk4::DropDown::new(Some(str_list), gtk4::Expression::NONE);
        dropdown.add_css_class("audio-app-route");

        // Find current sink index
        let current_pos = sinks
            .iter()
            .position(|s| s.name == input.sink_name)
            .unwrap_or(0);
        dropdown.set_selected(current_pos as u32);

        let sink_names: Vec<String> = sinks.iter().map(|s| s.name.clone()).collect();
        let idx = input.index;
        let streams_ref = app_streams_list.clone();
        dropdown.connect_selected_notify(move |dd| {
            let selected = dd.selected() as usize;
            if selected < sink_names.len() {
                audio::move_sink_input(idx, &sink_names[selected]);
                rebuild_app_streams(&streams_ref);
            }
        });

        row.append(&dropdown);
    }

    row
}

fn rebuild_card_profiles(card_profiles_list: &gtk4::Box) {
    // Clear existing children
    while let Some(child) = card_profiles_list.first_child() {
        card_profiles_list.remove(&child);
    }

    let cards = audio::list_cards();

    if cards.is_empty() {
        let empty_label = gtk4::Label::new(Some("No audio cards found"));
        empty_label.add_css_class("audio-app-empty");
        card_profiles_list.append(&empty_label);
        return;
    }

    for card in &cards {
        let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        row.add_css_class("audio-card-row");

        let name_label = gtk4::Label::new(Some(&card.description));
        name_label.add_css_class("audio-card-name");
        name_label.set_halign(gtk4::Align::Start);
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        row.append(&name_label);

        // Filter to available profiles
        let available_profiles: Vec<&audio::AudioProfile> =
            card.profiles.iter().filter(|p| p.available).collect();

        if available_profiles.is_empty() {
            card_profiles_list.append(&row);
            continue;
        }

        let descriptions: Vec<String> =
            available_profiles.iter().map(|p| p.description.clone()).collect();
        let str_list = gtk4::StringList::new(&descriptions.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let dropdown = gtk4::DropDown::new(Some(str_list), gtk4::Expression::NONE);
        dropdown.add_css_class("audio-card-profile-dropdown");

        // Find currently active profile
        let current_pos = available_profiles
            .iter()
            .position(|p| p.name == card.active_profile)
            .unwrap_or(0);
        dropdown.set_selected(current_pos as u32);

        let card_name = card.name.clone();
        let profile_names: Vec<String> =
            available_profiles.iter().map(|p| p.name.clone()).collect();
        let card_profiles_ref = card_profiles_list.clone();
        dropdown.connect_selected_notify(move |dd| {
            let selected = dd.selected() as usize;
            if selected < profile_names.len() {
                audio::set_card_profile(&card_name, &profile_names[selected]);
                rebuild_card_profiles(&card_profiles_ref);
            }
        });

        row.append(&dropdown);
        card_profiles_list.append(&row);
    }
}
