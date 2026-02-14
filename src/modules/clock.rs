use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Clock {
    pub widget: gtk4::Box,
    source_id: Option<glib::SourceId>,
}

impl Clock {
    pub fn new(interval_secs: u32) -> Self {
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

        popover_content.append(&tz_section);

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

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        // Store original IANA timezone for DST restore
        let original_tz: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        // --- Refresh helpers ---
        let refresh_bar = {
            let time_label = time_label.clone();
            let date_label = date_label.clone();
            move || {
                let now = glib::DateTime::now_local().unwrap();
                let time_str = now.format("%H:%M").unwrap_or_default();
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
            move || {
                let now = glib::DateTime::now_local().unwrap();

                // Large time
                let time_str = now.format("%H:%M:%S").unwrap_or_default();
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

                // DST status
                let is_dst_capable = !tz_name.starts_with("Etc/GMT");
                if is_dst_capable {
                    // IANA timezone — DST is "on" (timezone observes DST rules)
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
                    // Store the current IANA tz for restore
                    *original_tz.borrow_mut() = Some(tz_name);
                } else {
                    // Fixed-offset timezone — DST is "off"
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
            dst_switch.connect_state_set(move |_, state| {
                if state {
                    // Restore IANA timezone (DST on)
                    let tz = original_tz.borrow();
                    if let Some(ref iana_tz) = *tz {
                        let _ = std::process::Command::new("timedatectl")
                            .args(["set-timezone", iana_tz])
                            .status();
                    }
                } else {
                    // Switch to fixed offset (DST off)
                    let current_tz = get_timezone_name();
                    if !current_tz.starts_with("Etc/GMT") {
                        *original_tz.borrow_mut() = Some(current_tz);
                    }
                    let now = glib::DateTime::now_local().unwrap();
                    let offset_secs = now.utc_offset().as_seconds() as i64;
                    let offset_hours = offset_secs / 3600;
                    // Etc/GMT signs are inverted: UTC-5 = Etc/GMT+5
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

                // Refresh labels after a short delay to let timezone change take effect
                let tz_name_label = tz_name_label.clone();
                let tz_offset_label = tz_offset_label.clone();
                let dst_status_label = dst_status_label.clone();
                let large_time_label = large_time_label.clone();
                let full_date_label = full_date_label.clone();
                let time_label = time_label.clone();
                let date_label = date_label.clone();
                let original_tz = original_tz.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    let now = glib::DateTime::now_local().unwrap();

                    let time_str = now.format("%H:%M:%S").unwrap_or_default();
                    large_time_label.set_text(&time_str);

                    let date_str = now.format("%A, %B %e, %Y").unwrap_or_default();
                    full_date_label.set_text(&date_str);

                    let bar_time = now.format("%H:%M").unwrap_or_default();
                    time_label.set_text(&bar_time);
                    let bar_date = now.format("%a, %b %e").unwrap_or_default();
                    date_label.set_text(&bar_date);

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
            Some(glib::timeout_add_seconds_local(interval_secs, move || {
                let now = glib::DateTime::now_local().unwrap();

                let time_str = now.format("%H:%M").unwrap_or_default();
                time_label.set_text(&time_str);
                let date_str = now.format("%a, %b %e").unwrap_or_default();
                date_label.set_text(&date_str);

                if popover.is_visible() {
                    let time_str = now.format("%H:%M:%S").unwrap_or_default();
                    large_time_label.set_text(&time_str);
                    let date_str = now.format("%A, %B %e, %Y").unwrap_or_default();
                    full_date_label.set_text(&date_str);
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
