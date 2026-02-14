use crate::system::battery;
use crate::system::power;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const GRAPH_HISTORY_SIZE: usize = 60;
const GRAPH_WIDTH: i32 = 280;
const GRAPH_HEIGHT: i32 = 120;

pub struct Power {
    pub widget: gtk4::Box,
    label: gtk4::Label,
    source_id: Option<glib::SourceId>,
}

impl Power {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("power");

        let label = gtk4::Label::new(Some("--"));

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("power-module-button");
        menu_button.set_has_frame(false);

        // ᚢ Uruz - Power
        let rune = gtk4::Label::new(Some("\u{16A2}"));
        rune.add_css_class("module-rune");
        rune.set_tooltip_text(Some("\u{16A2} Uruz - Power"));

        let btn_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_content.append(&rune);
        btn_content.append(&label);
        menu_button.set_child(Some(&btn_content));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("power-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16A2}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Nidavellir's Forge (Power)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        // Battery section
        let has_battery = !battery::find_batteries().is_empty();

        let battery_section = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        battery_section.add_css_class("power-battery-section");

        let bat_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let bat_rune = gtk4::Label::new(Some("\u{16C7}")); // ᛇ Eihwaz
        bat_rune.add_css_class("power-battery-rune");
        let bat_label = gtk4::Label::new(Some("Mjolnir's Lightning (Battery)"));
        bat_label.add_css_class("power-battery-title");
        bat_header.append(&bat_rune);
        bat_header.append(&bat_label);
        battery_section.append(&bat_header);

        let battery_status_label = gtk4::Label::new(Some("--"));
        battery_status_label.add_css_class("power-battery-status");
        battery_status_label.set_halign(gtk4::Align::Start);
        battery_section.append(&battery_status_label);

        let battery_time_label = gtk4::Label::new(Some(""));
        battery_time_label.add_css_class("power-battery-detail");
        battery_time_label.set_halign(gtk4::Align::Start);
        battery_section.append(&battery_time_label);

        let battery_power_label = gtk4::Label::new(Some(""));
        battery_power_label.add_css_class("power-battery-watts");
        battery_power_label.set_halign(gtk4::Align::Start);
        battery_section.append(&battery_power_label);

        if has_battery {
            popover_content.append(&battery_section);
        }

        // Graph separator
        if has_battery {
            let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            sep1.add_css_class("power-separator");
            popover_content.append(&sep1);
        }

        // Power graph (Cairo DrawingArea)
        let power_history: Rc<RefCell<VecDeque<f64>>> =
            Rc::new(RefCell::new(VecDeque::with_capacity(GRAPH_HISTORY_SIZE)));

        let graph_area = gtk4::DrawingArea::new();
        graph_area.add_css_class("power-graph");
        graph_area.set_size_request(GRAPH_WIDTH, GRAPH_HEIGHT);

        let history_for_draw = power_history.clone();
        graph_area.set_draw_func(move |_area, cr, width, height| {
            draw_power_graph(cr, width, height, &history_for_draw.borrow());
        });

        if has_battery {
            popover_content.append(&graph_area);
        }

        // Profile section separator
        let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep2.add_css_class("power-separator");
        popover_content.append(&sep2);

        // Profile section
        let profile_label = gtk4::Label::new(Some("FORGE MODES"));
        profile_label.add_css_class("power-profile-label");
        profile_label.set_halign(gtk4::Align::Start);
        popover_content.append(&profile_label);

        let profile_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        profile_box.set_homogeneous(true);

        let profiles_info = power::get_profiles();

        let profile_buttons: Vec<(power::PowerProfile, gtk4::Button)> = profiles_info
            .available
            .iter()
            .map(|profile| {
                let btn = gtk4::Button::with_label(profile.display_name());
                btn.add_css_class("settings-theme-btn");
                if *profile == profiles_info.active {
                    btn.add_css_class("active");
                }
                profile_box.append(&btn);
                (profile.clone(), btn)
            })
            .collect();

        // Connect profile button click handlers
        let profile_buttons_rc = Rc::new(profile_buttons);
        for (profile, btn) in profile_buttons_rc.iter() {
            let profile_clone = profile.clone();
            let buttons_ref = profile_buttons_rc.clone();
            btn.connect_clicked(move |_| {
                let _ = power::set_profile(&profile_clone);
                // Update button active states
                for (p, b) in buttons_ref.iter() {
                    if *p == profile_clone {
                        b.add_css_class("active");
                    } else {
                        b.remove_css_class("active");
                    }
                }
            });
        }

        popover_content.append(&profile_box);

        // Footer info
        let footer_label = gtk4::Label::new(Some(""));
        footer_label.add_css_class("power-info-footer");
        footer_label.set_halign(gtk4::Align::Start);
        popover_content.append(&footer_label);

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        // Refresh labels when popover opens
        let bat_status_ref = battery_status_label.clone();
        let bat_time_ref = battery_time_label.clone();
        let bat_power_ref = battery_power_label.clone();
        let footer_ref = footer_label.clone();
        let history_for_show = power_history.clone();
        let graph_ref = graph_area.clone();
        popover.connect_show(move |_| {
            refresh_popover(
                &bat_status_ref,
                &bat_time_ref,
                &bat_power_ref,
                &footer_ref,
                &history_for_show,
                &graph_ref,
            );
        });

        let mut module = Self {
            widget,
            label,
            source_id: None,
        };

        module.refresh_label();
        module.start_updates(
            interval_secs,
            battery_status_label,
            battery_time_label,
            battery_power_label,
            footer_label,
            power_history,
            graph_area,
        );
        module
    }

    fn start_updates(
        &mut self,
        interval_secs: u32,
        bat_status: gtk4::Label,
        bat_time: gtk4::Label,
        bat_power: gtk4::Label,
        footer: gtk4::Label,
        power_history: Rc<RefCell<VecDeque<f64>>>,
        graph_area: gtk4::DrawingArea,
    ) {
        let label = self.label.clone();
        let widget = self.widget.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_bar_label(&label, &widget);
            refresh_popover(
                &bat_status,
                &bat_time,
                &bat_power,
                &footer,
                &power_history,
                &graph_area,
            );
            glib::ControlFlow::Continue
        }));
    }

    fn refresh_label(&self) {
        refresh_bar_label(&self.label, &self.widget);
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn refresh_bar_label(label: &gtk4::Label, widget: &gtk4::Box) {
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

fn refresh_popover(
    bat_status: &gtk4::Label,
    bat_time: &gtk4::Label,
    bat_power: &gtk4::Label,
    footer: &gtk4::Label,
    power_history: &Rc<RefCell<VecDeque<f64>>>,
    graph_area: &gtk4::DrawingArea,
) {
    let info = power::get_info();

    // Update battery info
    if let Some(bat) = battery::get_first_battery() {
        let status_text = match bat.status {
            battery::BatteryStatus::Charging => "Charging",
            battery::BatteryStatus::Discharging => "Discharging",
            battery::BatteryStatus::Full => "Full",
            battery::BatteryStatus::NotCharging => "Not Charging",
            battery::BatteryStatus::Unknown => "Unknown",
        };
        bat_status.set_text(&format!("{}% - {}", bat.capacity, status_text));

        let time_str = battery::format_time_remaining(bat.time_remaining);
        if !time_str.is_empty() {
            let time_label = match bat.status {
                battery::BatteryStatus::Charging => format!("{time_str} until full"),
                battery::BatteryStatus::Discharging => format!("{time_str} remaining"),
                _ => time_str,
            };
            bat_time.set_text(&time_label);
            bat_time.set_visible(true);
        } else {
            bat_time.set_visible(false);
        }

        // Power draw in watts (power_now is in microwatts)
        let watts = bat.power_now as f64 / 1_000_000.0;
        if watts > 0.0 {
            bat_power.set_text(&format!("{watts:.1}W"));
            bat_power.set_visible(true);
        } else {
            bat_power.set_visible(false);
        }

        // Update history
        let mut history = power_history.borrow_mut();
        if history.len() >= GRAPH_HISTORY_SIZE {
            history.pop_front();
        }
        history.push_back(watts);
    }

    // Update footer
    let mut footer_parts = Vec::new();
    footer_parts.push(format!("Governor: {}", info.governor.display_name()));
    if info.has_temp {
        footer_parts.push(format!("CPU: {}", power::format_temperature(info.temperature)));
    }
    if info.frequency_mhz > 0 {
        if info.frequency_mhz >= 1000 {
            footer_parts.push(format!("{:.1}GHz", info.frequency_mhz as f64 / 1000.0));
        } else {
            footer_parts.push(format!("{}MHz", info.frequency_mhz));
        }
    }
    footer.set_text(&footer_parts.join(" | "));

    // Redraw graph
    graph_area.queue_draw();
}

fn draw_power_graph(cr: &gtk4::cairo::Context, width: i32, height: i32, history: &VecDeque<f64>) {
    let w = width as f64;
    let h = height as f64;
    let padding_top = 20.0;
    let padding_bottom = 10.0;
    let padding_left = 35.0;
    let padding_right = 10.0;

    let graph_w = w - padding_left - padding_right;
    let graph_h = h - padding_top - padding_bottom;

    // Background
    cr.set_source_rgba(0.051, 0.055, 0.078, 0.9); // void_deep
    let _ = cr.paint();

    // Find max value for Y-axis scaling
    let max_watts = history.iter().cloned().fold(5.0_f64, f64::max);
    let y_max = (max_watts * 1.2).max(5.0); // 20% headroom, min 5W

    // Grid lines (horizontal, dashed)
    cr.set_dash(&[4.0, 4.0], 0.0);
    cr.set_line_width(0.5);
    let num_grid_lines = 4;
    for i in 0..=num_grid_lines {
        let frac = i as f64 / num_grid_lines as f64;
        let y = padding_top + graph_h * (1.0 - frac);
        let value = y_max * frac;

        // Grid line
        cr.set_source_rgba(0.478, 0.635, 0.969, 0.15); // bifrost_blue faint
        cr.move_to(padding_left, y);
        cr.line_to(w - padding_right, y);
        let _ = cr.stroke();

        // Y-axis label
        cr.set_source_rgba(0.663, 0.694, 0.839, 0.7); // moonlight
        cr.set_font_size(9.0);
        cr.move_to(2.0, y + 3.0);
        let label = format!("{value:.0}W");
        let _ = cr.show_text(&label);
    }
    cr.set_dash(&[], 0.0);

    if history.len() < 2 {
        // No data - show placeholder text
        cr.set_source_rgba(0.663, 0.694, 0.839, 0.4);
        cr.set_font_size(11.0);
        cr.move_to(w / 2.0 - 40.0, h / 2.0);
        let _ = cr.show_text("Gathering data...");
        return;
    }

    // Build path points
    let points: Vec<(f64, f64)> = history
        .iter()
        .enumerate()
        .map(|(i, &watts)| {
            let x = padding_left + (i as f64 / (GRAPH_HISTORY_SIZE - 1) as f64) * graph_w;
            let y = padding_top + graph_h * (1.0 - (watts / y_max).min(1.0));
            (x, y)
        })
        .collect();

    // Fill under the line
    cr.move_to(points[0].0, padding_top + graph_h);
    for &(x, y) in &points {
        cr.line_to(x, y);
    }
    cr.line_to(points.last().unwrap().0, padding_top + graph_h);
    cr.close_path();
    cr.set_source_rgba(0.478, 0.635, 0.969, 0.1); // bifrost_blue, very transparent
    let _ = cr.fill();

    // Draw the line
    cr.set_line_width(1.5);
    cr.set_source_rgba(0.478, 0.635, 0.969, 0.8); // bifrost_blue
    cr.move_to(points[0].0, points[0].1);
    for &(x, y) in points.iter().skip(1) {
        cr.line_to(x, y);
    }
    let _ = cr.stroke();

    // Current value dot
    if let Some(&(x, y)) = points.last() {
        cr.arc(x, y, 3.0, 0.0, std::f64::consts::TAU);
        cr.set_source_rgba(0.478, 0.635, 0.969, 1.0);
        let _ = cr.fill();
    }
}
