use crate::config::ClockConfig;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const WORLD_TIMEZONES: &[(&str, &str)] = &[
    ("UTC", "UTC"),
    ("US/Eastern", "New York"),
    ("US/Central", "Chicago"),
    ("US/Mountain", "Denver"),
    ("US/Pacific", "Los Angeles"),
    ("Europe/London", "London"),
    ("Europe/Paris", "Paris"),
    ("Europe/Berlin", "Berlin"),
    ("Asia/Tokyo", "Tokyo"),
    ("Asia/Shanghai", "Shanghai"),
    ("Asia/Kolkata", "Kolkata"),
    ("Australia/Sydney", "Sydney"),
    ("Pacific/Auckland", "Auckland"),
    ("America/Sao_Paulo", "São Paulo"),
    ("Africa/Cairo", "Cairo"),
];

pub struct Clock {
    pub widget: gtk4::Box,
    source_id: Option<glib::SourceId>,
}

impl Clock {
    pub fn new(interval_secs: u32, config: &ClockConfig) -> Self {
        let use_12h = Rc::new(RefCell::new(config.use_12h));

        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("clock");

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("clock-button");
        menu_button.set_has_frame(false);

        // Bar button content: rune + date + time + rune
        let btn_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

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

        btn_content.append(&rune_left);
        btn_content.append(&date_label);
        btn_content.append(&time_label);
        btn_content.append(&rune_right);
        menu_button.set_child(Some(&btn_content));

        // === Popover ===
        let popover = gtk4::Popover::new();
        popover.add_css_class("clock-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);
        popover_content.set_size_request(320, -1);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16C3}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("The Norns' Weaving (Clock)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        // Separator
        let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep1.add_css_class("clock-separator");
        popover_content.append(&sep1);

        // Large time display
        let large_time_label = gtk4::Label::new(None);
        large_time_label.add_css_class("clock-time-large");
        large_time_label.set_halign(gtk4::Align::Center);
        popover_content.append(&large_time_label);

        // Full date display
        let full_date_label = gtk4::Label::new(None);
        full_date_label.add_css_class("clock-date-full");
        full_date_label.set_halign(gtk4::Align::Center);
        popover_content.append(&full_date_label);

        // Separator
        let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep2.add_css_class("clock-separator");
        popover_content.append(&sep2);

        // Timezone section
        let tz_section = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        tz_section.add_css_class("clock-timezone-section");
        tz_section.set_margin_top(4);
        tz_section.set_margin_bottom(4);

        let tz_name_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let tz_realm_label = gtk4::Label::new(Some("Realm:"));
        tz_realm_label.add_css_class("clock-timezone-label");
        let tz_name_label = gtk4::Label::new(None);
        tz_name_label.add_css_class("clock-timezone-name");
        tz_name_label.set_hexpand(true);
        tz_name_label.set_halign(gtk4::Align::Start);
        tz_name_row.append(&tz_realm_label);
        tz_name_row.append(&tz_name_label);
        tz_section.append(&tz_name_row);

        let tz_offset_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let tz_offset_prefix = gtk4::Label::new(Some("Offset:"));
        tz_offset_prefix.add_css_class("clock-timezone-label");
        let tz_offset_label = gtk4::Label::new(None);
        tz_offset_label.add_css_class("clock-timezone-offset");
        tz_offset_label.set_hexpand(true);
        tz_offset_label.set_halign(gtk4::Align::Start);
        tz_offset_row.append(&tz_offset_prefix);
        tz_offset_row.append(&tz_offset_label);
        tz_section.append(&tz_offset_row);

        // NTP sync status
        if config.show_ntp_status {
            let ntp_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            let ntp_prefix = gtk4::Label::new(Some("NTP Sync:"));
            ntp_prefix.add_css_class("clock-timezone-label");
            let ntp_status_label = gtk4::Label::new(None);
            ntp_status_label.add_css_class("clock-ntp-status");
            ntp_status_label.set_hexpand(true);
            ntp_status_label.set_halign(gtk4::Align::Start);
            ntp_row.append(&ntp_prefix);
            ntp_row.append(&ntp_status_label);
            tz_section.append(&ntp_row);

            // Initial NTP check
            let (synced, status_text) = get_ntp_status();
            ntp_status_label.set_text(&status_text);
            if synced {
                ntp_status_label.add_css_class("ntp-synced");
            } else {
                ntp_status_label.add_css_class("ntp-unsynced");
            }
        }

        popover_content.append(&tz_section);

        // Separator
        let sep_tz_viewer = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep_tz_viewer.add_css_class("clock-separator");
        popover_content.append(&sep_tz_viewer);

        // World timezone viewer
        let tz_viewer_section = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        tz_viewer_section.add_css_class("clock-tz-viewer-section");

        let tz_viewer_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        // ᚱ Raidho - Journey/Travel
        let tz_viewer_rune = gtk4::Label::new(Some("\u{16B1}"));
        tz_viewer_rune.add_css_class("clock-tz-viewer-rune");
        let tz_viewer_title = gtk4::Label::new(Some("Realm Viewer"));
        tz_viewer_title.add_css_class("clock-dst-label");
        tz_viewer_title.set_hexpand(true);
        tz_viewer_title.set_halign(gtk4::Align::Start);
        tz_viewer_header.append(&tz_viewer_rune);
        tz_viewer_header.append(&tz_viewer_title);
        tz_viewer_section.append(&tz_viewer_header);

        // Timezone dropdown
        let tz_labels: Vec<String> = WORLD_TIMEZONES.iter().map(|(_, name)| name.to_string()).collect();
        let string_list = gtk4::StringList::new(&tz_labels.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let tz_dropdown = gtk4::DropDown::new(Some(string_list), gtk4::Expression::NONE);
        tz_dropdown.add_css_class("clock-tz-dropdown");
        tz_dropdown.set_selected(0);
        tz_viewer_section.append(&tz_dropdown);

        // Remote time display
        let remote_time_label = gtk4::Label::new(None);
        remote_time_label.add_css_class("clock-remote-time");
        remote_time_label.set_halign(gtk4::Align::Center);
        tz_viewer_section.append(&remote_time_label);

        // Update remote time when timezone selection changes
        {
            let remote_label = remote_time_label.clone();
            let use_12h_ref = use_12h.clone();
            tz_dropdown.connect_selected_notify(move |dd| {
                let idx = dd.selected() as usize;
                if idx < WORLD_TIMEZONES.len() {
                    let (tz_id, tz_name) = WORLD_TIMEZONES[idx];
                    update_remote_time(&remote_label, tz_id, tz_name, *use_12h_ref.borrow());
                }
            });
        }

        popover_content.append(&tz_viewer_section);

        // Separator
        let sep3 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep3.add_css_class("clock-separator");
        popover_content.append(&sep3);

        // Calendar
        let calendar = gtk4::Calendar::new();
        calendar.add_css_class("clock-calendar");
        popover_content.append(&calendar);

        // Separator
        let sep4 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep4.add_css_class("clock-separator");
        popover_content.append(&sep4);

        // DST Section
        let dst_section = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        dst_section.add_css_class("clock-dst-section");

        let dst_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        // ᛊ Sowilo - Sun
        let dst_rune = gtk4::Label::new(Some("\u{16CA}"));
        dst_rune.add_css_class("clock-dst-rune");
        let dst_title = gtk4::Label::new(Some("Daylight Saving"));
        dst_title.add_css_class("clock-dst-label");
        dst_title.set_hexpand(true);
        dst_title.set_halign(gtk4::Align::Start);
        dst_header.append(&dst_rune);
        dst_header.append(&dst_title);
        dst_section.append(&dst_header);

        let dst_status_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let dst_status_prefix = gtk4::Label::new(Some("Status:"));
        dst_status_prefix.add_css_class("clock-dst-label");
        let dst_status_label = gtk4::Label::new(None);
        dst_status_label.add_css_class("clock-dst-status");
        dst_status_label.set_hexpand(true);
        dst_status_label.set_halign(gtk4::Align::Start);
        dst_status_row.append(&dst_status_prefix);
        dst_status_row.append(&dst_status_label);

        let dst_switch = gtk4::Switch::new();
        dst_switch.set_valign(gtk4::Align::Center);
        dst_switch.add_css_class("clock-dst-switch");
        dst_status_row.append(&dst_switch);
        dst_section.append(&dst_status_row);

        popover_content.append(&dst_section);

        // Separator before 12h/24h toggle
        let sep5 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep5.add_css_class("clock-separator");
        popover_content.append(&sep5);

        // 12h/24h Toggle section
        let format_section = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        format_section.add_css_class("clock-format-section");

        let format_label = gtk4::Label::new(Some("12-Hour Format"));
        format_label.add_css_class("clock-dst-label");
        format_label.set_hexpand(true);
        format_label.set_halign(gtk4::Align::Start);
        format_section.append(&format_label);

        let format_switch = gtk4::Switch::new();
        format_switch.set_valign(gtk4::Align::Center);
        format_switch.add_css_class("clock-format-switch");
        format_switch.set_active(config.use_12h);
        format_section.append(&format_switch);

        popover_content.append(&format_section);

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        // Store original IANA timezone for DST restore
        let original_tz: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        // --- Refresh helpers ---
        let refresh_bar = {
            let time_label = time_label.clone();
            let date_label = date_label.clone();
            let use_12h = use_12h.clone();
            move || {
                let now = glib::DateTime::now_local().unwrap();
                let fmt = if *use_12h.borrow() { "%I:%M %p" } else { "%H:%M" };
                let time_str = now.format(fmt).unwrap_or_default();
                time_label.set_text(&time_str);
                let date_str = now.format("%a, %b %e").unwrap_or_default();
                date_label.set_text(&date_str);
            }
        };

        let refresh_popover = {
            let large_time_label = large_time_label.clone();
            let full_date_label = full_date_label.clone();
            let tz_name_label = tz_name_label.clone();
            let tz_offset_label = tz_offset_label.clone();
            let dst_status_label = dst_status_label.clone();
            let dst_switch = dst_switch.clone();
            let original_tz = original_tz.clone();
            let use_12h = use_12h.clone();
            let remote_time_label = remote_time_label.clone();
            let tz_dropdown = tz_dropdown.clone();
            move || {
                let now = glib::DateTime::now_local().unwrap();
                let is_12h = *use_12h.borrow();

                // Large time
                let fmt = if is_12h { "%I:%M:%S %p" } else { "%H:%M:%S" };
                let time_str = now.format(fmt).unwrap_or_default();
                large_time_label.set_text(&time_str);

                // Full date
                let date_str = now.format("%A, %B %e, %Y").unwrap_or_default();
                full_date_label.set_text(&date_str);

                // Timezone info
                let tz_name = get_timezone_name();
                let tz_abbrev = now.format("%Z").unwrap_or_default();
                tz_name_label.set_text(&format!("{} ({})", tz_name, tz_abbrev));

                let offset = now.format("UTC%:z").unwrap_or_default();
                tz_offset_label.set_text(&offset);

                // Update remote time viewer
                let idx = tz_dropdown.selected() as usize;
                if idx < WORLD_TIMEZONES.len() {
                    let (tz_id, tz_display) = WORLD_TIMEZONES[idx];
                    update_remote_time(&remote_time_label, tz_id, tz_display, is_12h);
                }

                // DST status
                let is_dst_capable = !tz_name.starts_with("Etc/GMT");
                if is_dst_capable {
                    let is_currently_dst = now.is_daylight_savings();
                    if is_currently_dst {
                        dst_status_label.set_text("Active (in DST period)");
                        dst_status_label.add_css_class("dst-active");
                        dst_status_label.remove_css_class("dst-inactive");
                    } else {
                        dst_status_label.set_text("Active (standard time)");
                        dst_status_label.add_css_class("dst-active");
                        dst_status_label.remove_css_class("dst-inactive");
                    }
                    dst_switch.set_active(true);
                    *original_tz.borrow_mut() = Some(tz_name);
                } else {
                    dst_status_label.set_text("Inactive");
                    dst_status_label.remove_css_class("dst-active");
                    dst_status_label.add_css_class("dst-inactive");
                    dst_switch.set_active(false);
                }
            }
        };

        // Initial refresh
        refresh_bar();
        refresh_popover();

        // 12h/24h format switch handler
        {
            let use_12h = use_12h.clone();
            let time_label = time_label.clone();
            let date_label = date_label.clone();
            let large_time_label = large_time_label.clone();
            let full_date_label = full_date_label.clone();
            let remote_time_label = remote_time_label.clone();
            let tz_dropdown = tz_dropdown.clone();
            format_switch.connect_state_set(move |_, state| {
                *use_12h.borrow_mut() = state;

                // Immediately refresh all time displays
                let now = glib::DateTime::now_local().unwrap();
                let bar_fmt = if state { "%I:%M %p" } else { "%H:%M" };
                time_label.set_text(&now.format(bar_fmt).unwrap_or_default());
                date_label.set_text(&now.format("%a, %b %e").unwrap_or_default());

                let large_fmt = if state { "%I:%M:%S %p" } else { "%H:%M:%S" };
                large_time_label.set_text(&now.format(large_fmt).unwrap_or_default());
                full_date_label.set_text(&now.format("%A, %B %e, %Y").unwrap_or_default());

                // Update remote time
                let idx = tz_dropdown.selected() as usize;
                if idx < WORLD_TIMEZONES.len() {
                    let (tz_id, tz_display) = WORLD_TIMEZONES[idx];
                    update_remote_time(&remote_time_label, tz_id, tz_display, state);
                }

                glib::Propagation::Proceed
            });
        }

        // DST switch handler
        {
            let original_tz = original_tz.clone();
            let tz_name_label = tz_name_label.clone();
            let tz_offset_label = tz_offset_label.clone();
            let dst_status_label = dst_status_label.clone();
            let large_time_label = large_time_label.clone();
            let full_date_label = full_date_label.clone();
            let time_label = time_label.clone();
            let date_label = date_label.clone();
            let use_12h = use_12h.clone();
            dst_switch.connect_state_set(move |_, state| {
                if state {
                    let tz = original_tz.borrow();
                    if let Some(ref iana_tz) = *tz {
                        let _ = std::process::Command::new("timedatectl")
                            .args(["set-timezone", iana_tz])
                            .status();
                    }
                } else {
                    let current_tz = get_timezone_name();
                    if !current_tz.starts_with("Etc/GMT") {
                        *original_tz.borrow_mut() = Some(current_tz);
                    }
                    let now = glib::DateTime::now_local().unwrap();
                    let offset_secs = now.utc_offset().as_seconds() as i64;
                    let offset_hours = offset_secs / 3600;
                    let etc_tz = if offset_hours == 0 {
                        "Etc/GMT".to_string()
                    } else if offset_hours > 0 {
                        format!("Etc/GMT-{}", offset_hours)
                    } else {
                        format!("Etc/GMT+{}", -offset_hours)
                    };
                    let _ = std::process::Command::new("timedatectl")
                        .args(["set-timezone", &etc_tz])
                        .status();
                }

                let tz_name_label = tz_name_label.clone();
                let tz_offset_label = tz_offset_label.clone();
                let dst_status_label = dst_status_label.clone();
                let large_time_label = large_time_label.clone();
                let full_date_label = full_date_label.clone();
                let time_label = time_label.clone();
                let date_label = date_label.clone();
                let original_tz = original_tz.clone();
                let use_12h = use_12h.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    let now = glib::DateTime::now_local().unwrap();
                    let is_12h = *use_12h.borrow();

                    let large_fmt = if is_12h { "%I:%M:%S %p" } else { "%H:%M:%S" };
                    large_time_label.set_text(&now.format(large_fmt).unwrap_or_default());

                    let date_str = now.format("%A, %B %e, %Y").unwrap_or_default();
                    full_date_label.set_text(&date_str);

                    let bar_fmt = if is_12h { "%I:%M %p" } else { "%H:%M" };
                    time_label.set_text(&now.format(bar_fmt).unwrap_or_default());
                    date_label.set_text(&now.format("%a, %b %e").unwrap_or_default());

                    let tz_name = get_timezone_name();
                    let tz_abbrev = now.format("%Z").unwrap_or_default();
                    tz_name_label.set_text(&format!("{} ({})", tz_name, tz_abbrev));

                    let offset = now.format("UTC%:z").unwrap_or_default();
                    tz_offset_label.set_text(&offset);

                    let is_dst_capable = !tz_name.starts_with("Etc/GMT");
                    if is_dst_capable {
                        let is_currently_dst = now.is_daylight_savings();
                        if is_currently_dst {
                            dst_status_label.set_text("Active (in DST period)");
                        } else {
                            dst_status_label.set_text("Active (standard time)");
                        }
                        dst_status_label.add_css_class("dst-active");
                        dst_status_label.remove_css_class("dst-inactive");
                        *original_tz.borrow_mut() = Some(tz_name);
                    } else {
                        dst_status_label.set_text("Inactive");
                        dst_status_label.remove_css_class("dst-active");
                        dst_status_label.add_css_class("dst-inactive");
                    }
                });

                glib::Propagation::Proceed
            });
        }

        // Refresh popover on show
        {
            let refresh_popover = refresh_popover.clone();
            popover.connect_show(move |_| {
                refresh_popover();
            });
        }

        // Periodic timer for bar labels + popover if visible
        let source_id = {
            let time_label = time_label.clone();
            let date_label = date_label.clone();
            let large_time_label = large_time_label.clone();
            let full_date_label = full_date_label.clone();
            let popover = popover.clone();
            let use_12h = use_12h.clone();
            Some(glib::timeout_add_seconds_local(interval_secs, move || {
                let now = glib::DateTime::now_local().unwrap();
                let is_12h = *use_12h.borrow();

                let bar_fmt = if is_12h { "%I:%M %p" } else { "%H:%M" };
                time_label.set_text(&now.format(bar_fmt).unwrap_or_default());
                date_label.set_text(&now.format("%a, %b %e").unwrap_or_default());

                if popover.is_visible() {
                    let large_fmt = if is_12h { "%I:%M:%S %p" } else { "%H:%M:%S" };
                    large_time_label.set_text(&now.format(large_fmt).unwrap_or_default());
                    full_date_label.set_text(&now.format("%A, %B %e, %Y").unwrap_or_default());
                }

                glib::ControlFlow::Continue
            }))
        };

        Self { widget, source_id }
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn get_timezone_name() -> String {
    // Try /etc/timezone first
    if let Ok(tz) = std::fs::read_to_string("/etc/timezone") {
        let tz = tz.trim().to_string();
        if !tz.is_empty() {
            return tz;
        }
    }

    // Try /etc/localtime symlink
    if let Ok(link) = std::fs::read_link("/etc/localtime") {
        let path = link.to_string_lossy().to_string();
        if let Some(pos) = path.find("zoneinfo/") {
            return path[pos + 9..].to_string();
        }
    }

    // Fallback to timedatectl
    if let Ok(output) = std::process::Command::new("timedatectl")
        .args(["show", "--property=Timezone", "--value"])
        .output()
    {
        let tz = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !tz.is_empty() {
            return tz;
        }
    }

    "Unknown".to_string()
}

fn get_ntp_status() -> (bool, String) {
    if let Ok(output) = std::process::Command::new("timedatectl")
        .args(["show", "--property=NTPSynchronized", "--value"])
        .output()
    {
        let value = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        if value == "yes" {
            return (true, "Active \u{2713}".to_string());
        }
    }
    (false, "Inactive".to_string())
}

fn update_remote_time(label: &gtk4::Label, tz_id: &str, tz_display: &str, use_12h: bool) {
    let tz = glib::TimeZone::new(Some(tz_id));
    if let Ok(dt) = glib::DateTime::now(&tz) {
        let fmt = if use_12h { "%I:%M %p" } else { "%H:%M" };
        let time_str = dt.format(fmt).unwrap_or_default();
        label.set_text(&format!("{}: {}", tz_display, time_str));
        return;
    }
    label.set_text(&format!("{}: --:--", tz_display));
}
