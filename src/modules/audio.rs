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

        popover.set_child(Some(&popover_content));
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

        let (sender, receiver) = async_channel::unbounded::<()>();

        glib::spawn_future_local(async move {
            while receiver.recv().await.is_ok() {
                refresh_audio(&label, &widget, &slider, &mute_btn, &updating);
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
    }

    pub fn stop(&self) {
        if let Some(listener) = &self.event_listener {
            listener.stop();
        }
    }
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
