use crate::system::media::{self, PlaybackStatus};
use gtk4::glib;
use gtk4::pango;
use gtk4::prelude::*;

pub struct Media {
    pub widget: gtk4::Box,
    status_icon: gtk4::Image,
    title_label: gtk4::Label,
    track_title: gtk4::Label,
    track_artist: gtk4::Label,
    play_pause_icon: gtk4::Image,
    progress_bar: gtk4::ProgressBar,
    position_label: gtk4::Label,
    available: bool,
    source_id: Option<glib::SourceId>,
}

impl Media {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("media");

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("media-button");
        menu_button.set_has_frame(false);

        // ᛚ Laguz - Flow, water, music
        let rune = gtk4::Label::new(Some("\u{16DA}"));
        rune.add_css_class("module-rune");

        let btn_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_content.append(&rune);

        let status_icon = gtk4::Image::from_icon_name("media-playback-stop-symbolic");
        status_icon.add_css_class("media-status-icon");
        btn_content.append(&status_icon);

        let title_label = gtk4::Label::new(Some("No media"));
        title_label.add_css_class("media-title");
        title_label.set_max_width_chars(20);
        title_label.set_ellipsize(pango::EllipsizeMode::End);
        btn_content.append(&title_label);

        menu_button.set_child(Some(&btn_content));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("media-popover");
        popover.set_autohide(true);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.add_css_class("media-popover-content");

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16DA}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Skaldic Songs (Media)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        content.append(&header);

        // Track info
        let track_info = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        track_info.add_css_class("media-track-info");

        let track_title = gtk4::Label::new(Some("No track playing"));
        track_title.add_css_class("media-track-title");
        track_title.set_halign(gtk4::Align::Start);
        track_title.set_max_width_chars(30);
        track_title.set_ellipsize(pango::EllipsizeMode::End);
        track_info.append(&track_title);

        let track_artist = gtk4::Label::new(None);
        track_artist.add_css_class("media-track-artist");
        track_artist.set_halign(gtk4::Align::Start);
        track_artist.set_max_width_chars(30);
        track_artist.set_ellipsize(pango::EllipsizeMode::End);
        track_info.append(&track_artist);

        content.append(&track_info);

        // Progress
        let progress_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.add_css_class("media-progress");
        progress_bar.set_show_text(false);
        progress_box.append(&progress_bar);

        let position_label = gtk4::Label::new(Some("0:00 / 0:00"));
        position_label.add_css_class("media-position");
        progress_box.append(&position_label);
        content.append(&progress_box);

        // Controls
        let controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        controls.set_halign(gtk4::Align::Center);
        controls.add_css_class("media-controls");

        // Previous - ᚱ Raidho
        let prev_btn = create_control_button("\u{16B1}", "Previous", || media::previous());
        prev_btn.add_css_class("media-prev");
        controls.append(&prev_btn);

        // Play/Pause
        let play_pause_btn = gtk4::Button::new();
        play_pause_btn.add_css_class("media-play-pause");
        let pp_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let play_pause_icon = gtk4::Image::from_icon_name("media-playback-start-symbolic");
        play_pause_icon.add_css_class("media-control-icon");
        pp_box.append(&play_pause_icon);
        play_pause_btn.set_child(Some(&pp_box));
        play_pause_btn.connect_clicked(|_| media::play_pause());
        controls.append(&play_pause_btn);

        // Next - ᚠ Fehu
        let next_btn = create_control_button("\u{16A0}", "Next", || media::next());
        next_btn.add_css_class("media-next");
        controls.append(&next_btn);

        content.append(&controls);

        popover.set_child(Some(&content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);
        widget.set_visible(false);

        let mut module = Self {
            widget,
            status_icon,
            title_label,
            track_title,
            track_artist,
            play_pause_icon,
            progress_bar,
            position_label,
            available: false,
            source_id: None,
        };

        module.refresh();
        module.start_updates(interval_secs);
        module
    }

    fn start_updates(&mut self, interval_secs: u32) {
        let widget = self.widget.clone();
        let status_icon = self.status_icon.clone();
        let title_label = self.title_label.clone();
        let track_title = self.track_title.clone();
        let track_artist = self.track_artist.clone();
        let play_pause_icon = self.play_pause_icon.clone();
        let progress_bar = self.progress_bar.clone();
        let position_label = self.position_label.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_media(
                &widget,
                &status_icon,
                &title_label,
                &track_title,
                &track_artist,
                &play_pause_icon,
                &progress_bar,
                &position_label,
            );
            glib::ControlFlow::Continue
        }));
    }

    fn refresh(&mut self) {
        let visible = refresh_media(
            &self.widget,
            &self.status_icon,
            &self.title_label,
            &self.track_title,
            &self.track_artist,
            &self.play_pause_icon,
            &self.progress_bar,
            &self.position_label,
        );
        self.available = visible;
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn create_control_button(
    rune: &str,
    tooltip: &str,
    on_click: impl Fn() + 'static,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("media-control-btn");
    btn.set_tooltip_text(Some(tooltip));

    let rune_label = gtk4::Label::new(Some(rune));
    rune_label.add_css_class("media-control-rune");
    btn.set_child(Some(&rune_label));
    btn.connect_clicked(move |_| on_click());
    btn
}

#[allow(clippy::too_many_arguments)]
fn refresh_media(
    widget: &gtk4::Box,
    status_icon: &gtk4::Image,
    title_label: &gtk4::Label,
    track_title: &gtk4::Label,
    track_artist: &gtk4::Label,
    play_pause_icon: &gtk4::Image,
    progress_bar: &gtk4::ProgressBar,
    position_label: &gtk4::Label,
) -> bool {
    let info = media::get_media_info();

    let should_show = info.available
        && (info.status == PlaybackStatus::Playing || info.status == PlaybackStatus::Paused)
        && !info.title.is_empty();

    widget.set_visible(should_show);
    if !should_show {
        return false;
    }

    // Status icon
    let icon_name = match info.status {
        PlaybackStatus::Playing => "media-playback-start-symbolic",
        PlaybackStatus::Paused => "media-playback-pause-symbolic",
        _ => "media-playback-stop-symbolic",
    };
    status_icon.set_icon_name(Some(icon_name));

    // Title in bar
    title_label.set_text(&media::truncate_string(&info.title, 20));

    // CSS classes
    widget.remove_css_class("playing");
    widget.remove_css_class("paused");
    match info.status {
        PlaybackStatus::Playing => widget.add_css_class("playing"),
        PlaybackStatus::Paused => widget.add_css_class("paused"),
        _ => {}
    }

    // Popover track info
    track_title.set_text(if info.title.is_empty() {
        "Unknown Track"
    } else {
        &info.title
    });

    if info.artist.is_empty() {
        track_artist.set_visible(false);
    } else {
        track_artist.set_text(&info.artist);
        track_artist.set_visible(true);
    }

    // Play/pause icon
    play_pause_icon.set_icon_name(Some(if info.status == PlaybackStatus::Playing {
        "media-playback-pause-symbolic"
    } else {
        "media-playback-start-symbolic"
    }));

    // Progress
    if info.length > 0 {
        let progress = info.position as f64 / info.length as f64;
        progress_bar.set_fraction(progress);
        position_label.set_text(&format!(
            "{} / {}",
            media::format_duration(info.position),
            media::format_duration(info.length)
        ));
    } else {
        progress_bar.set_fraction(0.0);
        position_label.set_text("0:00 / 0:00");
    }

    // Tooltip
    let mut tooltip = format!("{}\n{}", info.title, info.artist);
    if !info.player.is_empty() {
        tooltip += &format!("\nPlayer: {}", info.player);
    }
    widget.set_tooltip_text(Some(&tooltip));

    true
}
