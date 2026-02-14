use crate::system::connectivity;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct Connectivity {
    pub widget: gtk4::Box,
    wifi_icon: gtk4::Image,
    bt_icon: gtk4::Image,
    wifi_switch: gtk4::Switch,
    bt_switch: gtk4::Switch,
    wifi_label: gtk4::Label,
    bt_label: gtk4::Label,
    _wifi_list_box: gtk4::Box,
    _bt_paired_list: gtk4::Box,
    _bt_scan_list: gtk4::Box,
    updating: Rc<Cell<bool>>,
    source_id: Option<glib::SourceId>,
}

impl Connectivity {
    pub fn new(interval_secs: u32) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("connectivity");

        let updating = Rc::new(Cell::new(false));

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("connectivity-button");
        menu_button.set_has_frame(false);

        // Raidho rune
        let rune = gtk4::Label::new(Some("\u{16B1}"));
        rune.add_css_class("module-rune");

        let btn_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        btn_content.append(&rune);

        let wifi_icon = gtk4::Image::from_icon_name("network-wireless-symbolic");
        wifi_icon.add_css_class("connectivity-icon");
        btn_content.append(&wifi_icon);

        let bt_icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
        bt_icon.add_css_class("connectivity-icon");
        btn_content.append(&bt_icon);

        menu_button.set_child(Some(&btn_content));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("connectivity-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);
        popover_content.set_size_request(320, -1);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16B1}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Bifrost (Network)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        // === WiFi Section ===
        let wifi_label = gtk4::Label::new(Some("Disabled"));
        wifi_label.add_css_class("connectivity-status");
        wifi_label.set_halign(gtk4::Align::Start);

        let wifi_switch = gtk4::Switch::new();
        wifi_switch.add_css_class("connectivity-switch");
        wifi_switch.set_valign(gtk4::Align::Center);

        let wifi_section = create_section(
            "\u{16B9}", // Wunjo
            "network-wireless-symbolic",
            "WiFi",
            &wifi_label,
            &wifi_switch,
        );
        popover_content.append(&wifi_section);

        // WiFi network list
        let wifi_list_scroll = gtk4::ScrolledWindow::new();
        wifi_list_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        wifi_list_scroll.set_max_content_height(300);
        wifi_list_scroll.set_propagate_natural_height(true);
        wifi_list_scroll.add_css_class("wifi-network-list");

        let wifi_list_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        wifi_list_scroll.set_child(Some(&wifi_list_box));
        popover_content.append(&wifi_list_scroll);

        // WiFi scan button
        let wifi_scan_btn = gtk4::Button::new();
        wifi_scan_btn.add_css_class("wifi-scan-btn");
        let scan_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let scan_rune = gtk4::Label::new(Some("\u{16B1}")); // Raidho
        scan_rune.add_css_class("connectivity-rune");
        let scan_label = gtk4::Label::new(Some("Scan Networks"));
        scan_content.append(&scan_rune);
        scan_content.append(&scan_label);
        scan_content.set_halign(gtk4::Align::Center);
        wifi_scan_btn.set_child(Some(&scan_content));

        let wifi_list_clone = wifi_list_box.clone();
        let popover_clone = popover.clone();
        wifi_scan_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            let wifi_list = wifi_list_clone.clone();
            let pop = popover_clone.clone();
            let btn_clone = btn.clone();

            // Show scanning indicator
            clear_children(&wifi_list);
            let scanning = gtk4::Label::new(Some("Scanning networks..."));
            scanning.add_css_class("connectivity-empty");
            wifi_list.append(&scanning);

            let wifi_list_c = wifi_list.clone();
            let (sender, receiver) = async_channel::bounded::<Vec<connectivity::WiFiNetwork>>(1);
            std::thread::spawn(move || {
                let networks = connectivity::scan_wifi_networks();
                let _ = sender.send_blocking(networks);
            });
            glib::spawn_future_local(async move {
                if let Ok(networks) = receiver.recv().await {
                    clear_children(&wifi_list_c);
                    if networks.is_empty() {
                        let empty = gtk4::Label::new(Some("No networks found"));
                        empty.add_css_class("connectivity-empty");
                        wifi_list_c.append(&empty);
                    } else {
                        for network in &networks {
                            let row = create_wifi_row(network, &wifi_list_c, &pop);
                            wifi_list_c.append(&row);
                        }
                    }
                    btn_clone.set_sensitive(true);
                }
            });
        });
        popover_content.append(&wifi_scan_btn);

        // Hidden network button
        let hidden_btn = gtk4::Button::new();
        hidden_btn.add_css_class("wifi-scan-btn");
        let hidden_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let hidden_label = gtk4::Label::new(Some("Add Hidden Network"));
        hidden_content.append(&hidden_label);
        hidden_content.set_halign(gtk4::Align::Center);
        hidden_btn.set_child(Some(&hidden_content));

        let wifi_list_clone2 = wifi_list_box.clone();
        let popover_clone2 = popover.clone();
        hidden_btn.connect_clicked(move |_| {
            show_hidden_network_dialog(&wifi_list_clone2, &popover_clone2);
        });
        popover_content.append(&hidden_btn);

        let updating_wifi = updating.clone();
        let wifi_list_for_switch = wifi_list_box.clone();
        wifi_switch.connect_state_set(move |_, state| {
            if updating_wifi.get() {
                return glib::Propagation::Proceed;
            }
            connectivity::set_wifi_enabled(state);
            if !state {
                clear_children(&wifi_list_for_switch);
            }
            glib::Propagation::Proceed
        });

        // Separator
        let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
        sep.add_css_class("connectivity-separator");
        popover_content.append(&sep);

        // === Bluetooth Section ===
        let bt_label = gtk4::Label::new(Some("Disabled"));
        bt_label.add_css_class("connectivity-status");
        bt_label.set_halign(gtk4::Align::Start);

        let bt_switch = gtk4::Switch::new();
        bt_switch.add_css_class("connectivity-switch");
        bt_switch.set_valign(gtk4::Align::Center);

        let bt_section = create_section(
            "\u{16D2}", // Berkano
            "bluetooth-symbolic",
            "Bluetooth",
            &bt_label,
            &bt_switch,
        );
        popover_content.append(&bt_section);

        // Paired devices list
        let bt_paired_label = gtk4::Label::new(Some("Paired Devices"));
        bt_paired_label.add_css_class("connectivity-subtitle");
        bt_paired_label.set_halign(gtk4::Align::Start);
        popover_content.append(&bt_paired_label);

        let bt_paired_scroll = gtk4::ScrolledWindow::new();
        bt_paired_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        bt_paired_scroll.set_max_content_height(200);
        bt_paired_scroll.set_propagate_natural_height(true);

        let bt_paired_list = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        bt_paired_scroll.set_child(Some(&bt_paired_list));
        popover_content.append(&bt_paired_scroll);

        // Scan results list
        let bt_scan_label = gtk4::Label::new(Some("Nearby Devices"));
        bt_scan_label.add_css_class("connectivity-subtitle");
        bt_scan_label.set_halign(gtk4::Align::Start);
        popover_content.append(&bt_scan_label);

        let bt_scan_scroll = gtk4::ScrolledWindow::new();
        bt_scan_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        bt_scan_scroll.set_max_content_height(150);
        bt_scan_scroll.set_propagate_natural_height(true);

        let bt_scan_list = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        bt_scan_scroll.set_child(Some(&bt_scan_list));
        popover_content.append(&bt_scan_scroll);

        // BT scan button
        let bt_scan_btn = gtk4::Button::new();
        bt_scan_btn.add_css_class("bt-scan-btn");
        let bt_scan_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let bt_scan_rune = gtk4::Label::new(Some("\u{16D2}")); // Berkano
        bt_scan_rune.add_css_class("connectivity-rune");
        let bt_scan_btn_label = gtk4::Label::new(Some("Scan for Devices"));
        bt_scan_content.append(&bt_scan_rune);
        bt_scan_content.append(&bt_scan_btn_label);
        bt_scan_content.set_halign(gtk4::Align::Center);
        bt_scan_btn.set_child(Some(&bt_scan_content));

        let bt_paired_clone = bt_paired_list.clone();
        let bt_scan_clone = bt_scan_list.clone();
        bt_scan_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            let btn_label_clone = bt_scan_btn_label.clone();
            btn_label_clone.set_text("Scanning...");
            let paired_list = bt_paired_clone.clone();
            let scan_list = bt_scan_clone.clone();
            let btn_clone = btn.clone();
            // Run scan in a thread to avoid blocking UI
            let (sender, receiver) = async_channel::bounded::<(Vec<connectivity::BluetoothDevice>, Vec<connectivity::BluetoothDevice>)>(1);
            std::thread::spawn(move || {
                let paired = connectivity::get_paired_devices();
                let scanned = connectivity::scan_bluetooth_devices();
                let _ = sender.send_blocking((paired, scanned));
            });
            glib::spawn_future_local(async move {
                if let Ok((paired, scanned)) = receiver.recv().await {
                    populate_bt_paired_list(&paired_list, &paired);
                    populate_bt_scan_list(&scan_list, &scanned, &paired_list);
                    btn_clone.set_sensitive(true);
                    btn_label_clone.set_text("Scan for Devices");
                }
            });
        });
        popover_content.append(&bt_scan_btn);

        let updating_bt = updating.clone();
        bt_switch.connect_state_set(move |_, state| {
            if updating_bt.get() {
                return glib::Propagation::Proceed;
            }
            connectivity::set_bluetooth_enabled(state);
            glib::Propagation::Proceed
        });

        // Populate lists when popover opens (async to avoid blocking UI)
        let wifi_list_show = wifi_list_box.clone();
        let bt_paired_show = bt_paired_list.clone();
        let popover_show = popover.clone();
        popover.connect_show(move |_| {
            // Show loading indicators immediately
            clear_children(&wifi_list_show);
            let wifi_loading = gtk4::Label::new(Some("Scanning networks..."));
            wifi_loading.add_css_class("connectivity-empty");
            wifi_list_show.append(&wifi_loading);

            clear_children(&bt_paired_show);
            let bt_loading = gtk4::Label::new(Some("Loading devices..."));
            bt_loading.add_css_class("connectivity-empty");
            bt_paired_show.append(&bt_loading);

            // Fetch data in background thread
            let wifi_list_c = wifi_list_show.clone();
            let bt_paired_c = bt_paired_show.clone();
            let popover_c = popover_show.clone();
            let (sender, receiver) = async_channel::bounded::<(Vec<connectivity::WiFiNetwork>, Vec<connectivity::BluetoothDevice>)>(1);
            std::thread::spawn(move || {
                let networks = connectivity::scan_wifi_networks();
                let paired = connectivity::get_paired_devices();
                let _ = sender.send_blocking((networks, paired));
            });
            glib::spawn_future_local(async move {
                if let Ok((networks, paired)) = receiver.recv().await {
                    clear_children(&wifi_list_c);
                    if networks.is_empty() {
                        let empty = gtk4::Label::new(Some("No networks found"));
                        empty.add_css_class("connectivity-empty");
                        wifi_list_c.append(&empty);
                    } else {
                        for network in &networks {
                            let row = create_wifi_row(network, &wifi_list_c, &popover_c);
                            wifi_list_c.append(&row);
                        }
                    }
                    populate_bt_paired_list(&bt_paired_c, &paired);
                }
            });
        });

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        let mut module = Self {
            widget,
            wifi_icon,
            bt_icon,
            wifi_switch,
            bt_switch,
            wifi_label,
            bt_label,
            _wifi_list_box: wifi_list_box,
            _bt_paired_list: bt_paired_list,
            _bt_scan_list: bt_scan_list,
            updating,
            source_id: None,
        };

        module.refresh();
        module.start_updates(interval_secs);
        module
    }

    fn start_updates(&mut self, interval_secs: u32) {
        let widget = self.widget.clone();
        let wifi_icon = self.wifi_icon.clone();
        let bt_icon = self.bt_icon.clone();
        let wifi_switch = self.wifi_switch.clone();
        let bt_switch = self.bt_switch.clone();
        let wifi_label = self.wifi_label.clone();
        let bt_label = self.bt_label.clone();
        let updating = self.updating.clone();

        self.source_id = Some(glib::timeout_add_seconds_local(interval_secs, move || {
            refresh_connectivity(
                &widget,
                &wifi_icon,
                &bt_icon,
                &wifi_switch,
                &bt_switch,
                &wifi_label,
                &bt_label,
                &updating,
            );
            glib::ControlFlow::Continue
        }));
    }

    fn refresh(&self) {
        refresh_connectivity(
            &self.widget,
            &self.wifi_icon,
            &self.bt_icon,
            &self.wifi_switch,
            &self.bt_switch,
            &self.wifi_label,
            &self.bt_label,
            &self.updating,
        );
    }

    pub fn stop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}

fn create_section(
    rune_char: &str,
    icon_name: &str,
    title: &str,
    status_label: &gtk4::Label,
    switch: &gtk4::Switch,
) -> gtk4::Box {
    let section = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    section.add_css_class("connectivity-section");

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let rune = gtk4::Label::new(Some(rune_char));
    rune.add_css_class("connectivity-rune");
    row.append(&rune);

    let icon = gtk4::Image::from_icon_name(icon_name);
    icon.add_css_class("connectivity-section-icon");
    row.append(&icon);

    let label_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    label_box.set_hexpand(true);

    let title_label = gtk4::Label::new(Some(title));
    title_label.add_css_class("connectivity-title");
    title_label.set_halign(gtk4::Align::Start);
    label_box.append(&title_label);
    label_box.append(status_label);

    row.append(&label_box);
    row.append(switch);
    section.append(&row);

    section
}

fn clear_children(container: &gtk4::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn populate_wifi_list(list_box: &gtk4::Box, popover: &gtk4::Popover) {
    clear_children(list_box);

    let networks = connectivity::scan_wifi_networks();
    if networks.is_empty() {
        let empty = gtk4::Label::new(Some("No networks found"));
        empty.add_css_class("connectivity-empty");
        list_box.append(&empty);
        return;
    }

    for network in &networks {
        let row = create_wifi_row(network, list_box, popover);
        list_box.append(&row);
    }
}

fn create_wifi_row(
    network: &connectivity::WiFiNetwork,
    list_box: &gtk4::Box,
    popover: &gtk4::Popover,
) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    row.add_css_class("wifi-network-row");
    if network.connected {
        row.add_css_class("wifi-connected");
    }
    row.set_margin_top(2);
    row.set_margin_bottom(2);

    let main_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    main_row.set_margin_start(4);
    main_row.set_margin_end(4);

    // Signal icon
    let signal_icon = gtk4::Image::from_icon_name(connectivity::get_wifi_signal_icon(network.signal));
    signal_icon.add_css_class("wifi-signal-icon");
    main_row.append(&signal_icon);

    // SSID + info
    let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    info_box.set_hexpand(true);

    let ssid_label = gtk4::Label::new(Some(&network.ssid));
    ssid_label.add_css_class("wifi-ssid");
    ssid_label.set_halign(gtk4::Align::Start);
    ssid_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    info_box.append(&ssid_label);

    let details = format!(
        "{}% {}{}",
        network.signal,
        network.security,
        if network.saved { " (Saved)" } else { "" }
    );
    let detail_label = gtk4::Label::new(Some(&details));
    detail_label.add_css_class("wifi-detail");
    detail_label.set_halign(gtk4::Align::Start);
    info_box.append(&detail_label);

    main_row.append(&info_box);

    // Action button
    let ssid = network.ssid.clone();
    let is_connected = network.connected;
    let is_saved = network.saved;
    let security = network.security.clone();

    if is_connected {
        let disconnect_btn = gtk4::Button::with_label("Disconnect");
        disconnect_btn.add_css_class("wifi-action-btn");
        let list_clone = list_box.clone();
        let pop_clone = popover.clone();
        disconnect_btn.connect_clicked(move |_| {
            let _ = connectivity::disconnect_wifi();
            let lc = list_clone.clone();
            let pc = pop_clone.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                populate_wifi_list(&lc, &pc);
            });
        });
        main_row.append(&disconnect_btn);

        // Forget button for connected saved network
        if is_saved {
            let forget_btn = gtk4::Button::with_label("Forget");
            forget_btn.add_css_class("wifi-forget-btn");
            let ssid_clone = ssid.clone();
            let list_clone2 = list_box.clone();
            let pop_clone2 = popover.clone();
            forget_btn.connect_clicked(move |_| {
                let _ = connectivity::disconnect_wifi();
                let _ = connectivity::forget_wifi(&ssid_clone);
                let lc = list_clone2.clone();
                let pc = pop_clone2.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    populate_wifi_list(&lc, &pc);
                });
            });
            main_row.append(&forget_btn);
        }
    } else {
        let connect_btn = gtk4::Button::with_label("Connect");
        connect_btn.add_css_class("wifi-action-btn");

        let needs_password = !security.is_empty()
            && security != "--"
            && !is_saved;

        let row_ref = row.clone();
        let list_clone = list_box.clone();
        let pop_clone = popover.clone();
        let ssid_clone = ssid.clone();

        connect_btn.connect_clicked(move |_| {
            if needs_password {
                show_password_entry(&row_ref, &ssid_clone, &list_clone, &pop_clone);
            } else {
                let ssid_c = ssid_clone.clone();
                let list_c = list_clone.clone();
                let pop_c = pop_clone.clone();
                match connectivity::connect_wifi(&ssid_c, None) {
                    Ok(()) => {
                        glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                            populate_wifi_list(&list_c, &pop_c);
                        });
                    }
                    Err(e) => eprintln!("WiFi connect error: {e}"),
                }
            }
        });
        main_row.append(&connect_btn);

        // Forget button for saved but not connected
        if is_saved {
            let forget_btn = gtk4::Button::with_label("Forget");
            forget_btn.add_css_class("wifi-forget-btn");
            let ssid_clone2 = ssid.clone();
            let list_clone2 = list_box.clone();
            let pop_clone2 = popover.clone();
            forget_btn.connect_clicked(move |_| {
                let _ = connectivity::forget_wifi(&ssid_clone2);
                let lc = list_clone2.clone();
                let pc = pop_clone2.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    populate_wifi_list(&lc, &pc);
                });
            });
            main_row.append(&forget_btn);
        }
    }

    row.append(&main_row);
    row
}

fn show_password_entry(
    row: &gtk4::Box,
    ssid: &str,
    list_box: &gtk4::Box,
    popover: &gtk4::Popover,
) {
    // Check if password entry already shown
    let child_count = {
        let mut count = 0;
        let mut child = row.first_child();
        while let Some(c) = child {
            count += 1;
            child = c.next_sibling();
        }
        count
    };
    if child_count > 1 {
        return; // Already showing
    }

    let pw_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    pw_box.add_css_class("wifi-password-entry");
    pw_box.set_margin_start(4);
    pw_box.set_margin_end(4);

    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Password"));
    entry.set_visibility(false);
    entry.set_hexpand(true);
    entry.add_css_class("wifi-password-input");
    pw_box.append(&entry);

    let ok_btn = gtk4::Button::with_label("Connect");
    ok_btn.add_css_class("wifi-action-btn");

    let ssid_clone = ssid.to_string();
    let list_clone = list_box.clone();
    let pop_clone = popover.clone();
    let entry_clone = entry.clone();
    ok_btn.connect_clicked(move |_| {
        let pw = entry_clone.text().to_string();
        if pw.is_empty() {
            return;
        }
        let ssid_c = ssid_clone.clone();
        let list_c = list_clone.clone();
        let pop_c = pop_clone.clone();
        match connectivity::connect_wifi(&ssid_c, Some(&pw)) {
            Ok(()) => {
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    populate_wifi_list(&list_c, &pop_c);
                });
            }
            Err(e) => eprintln!("WiFi connect error: {e}"),
        }
    });
    pw_box.append(&ok_btn);

    // Also connect on Enter key
    let ssid_clone2 = ssid.to_string();
    let list_clone2 = list_box.clone();
    let pop_clone2 = popover.clone();
    entry.connect_activate(move |entry| {
        let pw = entry.text().to_string();
        if pw.is_empty() {
            return;
        }
        let ssid_c = ssid_clone2.clone();
        let list_c = list_clone2.clone();
        let pop_c = pop_clone2.clone();
        match connectivity::connect_wifi(&ssid_c, Some(&pw)) {
            Ok(()) => {
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    populate_wifi_list(&list_c, &pop_c);
                });
            }
            Err(e) => eprintln!("WiFi connect error: {e}"),
        }
    });

    row.append(&pw_box);
    entry.grab_focus();
}

fn show_hidden_network_dialog(list_box: &gtk4::Box, popover: &gtk4::Popover) {
    clear_children(list_box);

    let form = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    form.add_css_class("wifi-hidden-form");
    form.set_margin_start(4);
    form.set_margin_end(4);

    let title = gtk4::Label::new(Some("Hidden Network"));
    title.add_css_class("wifi-ssid");
    title.set_halign(gtk4::Align::Start);
    form.append(&title);

    let ssid_entry = gtk4::Entry::new();
    ssid_entry.set_placeholder_text(Some("Network Name (SSID)"));
    ssid_entry.add_css_class("wifi-password-input");
    form.append(&ssid_entry);

    let pw_entry = gtk4::Entry::new();
    pw_entry.set_placeholder_text(Some("Password"));
    pw_entry.set_visibility(false);
    pw_entry.add_css_class("wifi-password-input");
    form.append(&pw_entry);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let connect_btn = gtk4::Button::with_label("Connect");
    connect_btn.add_css_class("wifi-action-btn");

    let cancel_btn = gtk4::Button::with_label("Cancel");
    cancel_btn.add_css_class("wifi-forget-btn");

    let list_clone = list_box.clone();
    let pop_clone = popover.clone();
    let ssid_e = ssid_entry.clone();
    let pw_e = pw_entry.clone();
    connect_btn.connect_clicked(move |_| {
        let ssid = ssid_e.text().to_string();
        let pw = pw_e.text().to_string();
        if ssid.is_empty() || pw.is_empty() {
            return;
        }
        let list_c = list_clone.clone();
        let pop_c = pop_clone.clone();
        match connectivity::connect_hidden_wifi(&ssid, &pw) {
            Ok(()) => {
                glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                    populate_wifi_list(&list_c, &pop_c);
                });
            }
            Err(e) => eprintln!("Hidden WiFi connect error: {e}"),
        }
    });

    let list_clone2 = list_box.clone();
    let pop_clone2 = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        populate_wifi_list(&list_clone2, &pop_clone2);
    });

    btn_box.append(&connect_btn);
    btn_box.append(&cancel_btn);
    form.append(&btn_box);

    list_box.append(&form);
    ssid_entry.grab_focus();
}

// === Bluetooth List Functions ===

fn populate_bt_paired_list(list_box: &gtk4::Box, devices: &[connectivity::BluetoothDevice]) {
    clear_children(list_box);

    if devices.is_empty() {
        let empty = gtk4::Label::new(Some("No paired devices"));
        empty.add_css_class("connectivity-empty");
        list_box.append(&empty);
        return;
    }

    for device in devices {
        let row = create_bt_device_row(device, list_box, true);
        list_box.append(&row);
    }
}

fn populate_bt_scan_list(
    list_box: &gtk4::Box,
    devices: &[connectivity::BluetoothDevice],
    paired_list: &gtk4::Box,
) {
    clear_children(list_box);

    if devices.is_empty() {
        let empty = gtk4::Label::new(Some("No new devices found"));
        empty.add_css_class("connectivity-empty");
        list_box.append(&empty);
        return;
    }

    for device in devices {
        let row = create_bt_scan_row(device, list_box, paired_list);
        list_box.append(&row);
    }
}

fn create_bt_device_row(
    device: &connectivity::BluetoothDevice,
    list_box: &gtk4::Box,
    is_paired: bool,
) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.add_css_class("bt-device-row");
    if device.connected {
        row.add_css_class("bt-connected");
    }
    row.set_margin_top(2);
    row.set_margin_bottom(2);
    row.set_margin_start(4);
    row.set_margin_end(4);

    let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
    icon.add_css_class("connectivity-section-icon");
    row.append(&icon);

    let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    info_box.set_hexpand(true);

    let name_label = gtk4::Label::new(Some(&device.name));
    name_label.add_css_class("bt-device-name");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    info_box.append(&name_label);

    // Battery if available
    if let Some(battery) = device.battery {
        let battery_label = gtk4::Label::new(Some(&format!("Battery: {}%", battery)));
        battery_label.add_css_class("bt-battery");
        battery_label.set_halign(gtk4::Align::Start);
        info_box.append(&battery_label);
    }

    row.append(&info_box);

    let mac = device.mac.clone();
    let is_connected = device.connected;
    let list_clone = list_box.clone();

    if is_connected {
        let btn = gtk4::Button::with_label("Disconnect");
        btn.add_css_class("wifi-action-btn");
        let mac_clone = mac.clone();
        btn.connect_clicked(move |_| {
            let _ = connectivity::disconnect_bluetooth(&mac_clone);
            let list_c = list_clone.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                let devices = connectivity::get_paired_devices();
                populate_bt_paired_list(&list_c, &devices);
            });
        });
        row.append(&btn);
    } else if is_paired {
        let btn = gtk4::Button::with_label("Connect");
        btn.add_css_class("wifi-action-btn");
        let mac_clone = mac.clone();
        let list_clone2 = list_clone.clone();
        btn.connect_clicked(move |_| {
            let mac_c = mac_clone.clone();
            let list_c = list_clone2.clone();
            // Connect in background thread since it can take a few seconds
            let (sender, receiver) = async_channel::bounded::<Result<(), String>>(1);
            std::thread::spawn(move || {
                let result = connectivity::connect_bluetooth(&mac_c);
                let _ = sender.send_blocking(result);
            });
            let list_c2 = list_c.clone();
            glib::spawn_future_local(async move {
                if let Ok(_result) = receiver.recv().await {
                    let devices = connectivity::get_paired_devices();
                    populate_bt_paired_list(&list_c2, &devices);
                }
            });
        });
        row.append(&btn);

        // Remove button
        let remove_btn = gtk4::Button::with_label("Remove");
        remove_btn.add_css_class("wifi-forget-btn");
        let mac_clone2 = mac.clone();
        remove_btn.connect_clicked(move |_| {
            let _ = connectivity::remove_bluetooth(&mac_clone2);
            let list_c = list_clone.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                let devices = connectivity::get_paired_devices();
                populate_bt_paired_list(&list_c, &devices);
            });
        });
        row.append(&remove_btn);
    }

    row
}

fn create_bt_scan_row(
    device: &connectivity::BluetoothDevice,
    _scan_list: &gtk4::Box,
    paired_list: &gtk4::Box,
) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.add_css_class("bt-device-row");
    row.set_margin_top(2);
    row.set_margin_bottom(2);
    row.set_margin_start(4);
    row.set_margin_end(4);

    let icon = gtk4::Image::from_icon_name("bluetooth-symbolic");
    icon.add_css_class("connectivity-section-icon");
    row.append(&icon);

    let name_label = gtk4::Label::new(Some(&device.name));
    name_label.add_css_class("bt-device-name");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_hexpand(true);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    row.append(&name_label);

    let pair_btn = gtk4::Button::with_label("Pair");
    pair_btn.add_css_class("bt-pair-btn");
    let mac = device.mac.clone();
    let paired_clone = paired_list.clone();
    pair_btn.connect_clicked(move |btn| {
        btn.set_sensitive(false);
        let mac_c = mac.clone();
        let paired_c = paired_clone.clone();
        let btn_c = btn.clone();
        let (sender, receiver) = async_channel::bounded::<Result<(), String>>(1);
        std::thread::spawn(move || {
            let result = connectivity::pair_bluetooth(&mac_c);
            let _ = sender.send_blocking(result);
        });
        glib::spawn_future_local(async move {
            if let Ok(result) = receiver.recv().await {
                match result {
                    Ok(()) => {
                        btn_c.set_label("Paired!");
                        let devices = connectivity::get_paired_devices();
                        populate_bt_paired_list(&paired_c, &devices);
                    }
                    Err(e) => {
                        btn_c.set_label("Failed");
                        btn_c.set_sensitive(true);
                        eprintln!("BT pair error: {e}");
                    }
                }
            }
        });
    });
    row.append(&pair_btn);

    row
}

#[allow(clippy::too_many_arguments)]
fn refresh_connectivity(
    widget: &gtk4::Box,
    wifi_icon: &gtk4::Image,
    bt_icon: &gtk4::Image,
    wifi_switch: &gtk4::Switch,
    bt_switch: &gtk4::Switch,
    wifi_label: &gtk4::Label,
    bt_label: &gtk4::Label,
    updating: &Rc<Cell<bool>>,
) {
    updating.set(true);

    let wifi = connectivity::get_wifi_info();
    wifi_icon.set_icon_name(Some(connectivity::get_wifi_icon(&wifi)));
    wifi_switch.set_active(wifi.enabled);

    if !wifi.enabled {
        wifi_label.set_text("Disabled");
        wifi_label.remove_css_class("connected");
        widget.add_css_class("wifi-disabled");
    } else if wifi.connected {
        wifi_label.set_text(&format!("{} ({}%)", wifi.ssid, wifi.signal));
        wifi_label.add_css_class("connected");
        widget.remove_css_class("wifi-disabled");
    } else {
        wifi_label.set_text("Not connected");
        wifi_label.remove_css_class("connected");
        widget.remove_css_class("wifi-disabled");
    }

    let bt = connectivity::get_bluetooth_info();
    bt_icon.set_icon_name(Some(connectivity::get_bluetooth_icon(&bt)));

    if bt.available {
        bt_switch.set_sensitive(true);
        bt_switch.set_active(bt.powered);
    } else {
        bt_switch.set_sensitive(false);
        bt_switch.set_active(false);
    }

    if !bt.available {
        bt_label.set_text("Not available");
        bt_label.remove_css_class("connected");
    } else if !bt.powered {
        bt_label.set_text("Disabled");
        bt_label.remove_css_class("connected");
        widget.add_css_class("bt-disabled");
    } else if bt.connected {
        bt_label.set_text(&bt.device);
        bt_label.add_css_class("connected");
        widget.remove_css_class("bt-disabled");
    } else {
        bt_label.set_text("Not connected");
        bt_label.remove_css_class("connected");
        widget.remove_css_class("bt-disabled");
    }

    // Tooltip
    let mut tooltip = if wifi.enabled {
        if wifi.connected {
            format!("WiFi: {} ({}%)", wifi.ssid, wifi.signal)
        } else {
            "WiFi: Enabled (not connected)".to_string()
        }
    } else {
        "WiFi: Disabled".to_string()
    };

    if !bt.available {
        tooltip += "\nBluetooth: Not available";
    } else if !bt.powered {
        tooltip += "\nBluetooth: Disabled";
    } else if bt.connected {
        tooltip += &format!("\nBluetooth: {}", bt.device);
    } else {
        tooltip += "\nBluetooth: Enabled (not connected)";
    }

    tooltip += "\nClick to manage";
    widget.set_tooltip_text(Some(&tooltip));

    updating.set(false);
}
