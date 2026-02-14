use crate::system::app_finder;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct AppFinder {
    pub widget: gtk4::Box,
    entries: Rc<RefCell<Vec<app_finder::DesktopEntry>>>,
}

impl AppFinder {
    pub fn new() -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        widget.add_css_class("app-finder");

        let entries = Rc::new(RefCell::new(app_finder::load_desktop_entries()));

        let menu_button = gtk4::MenuButton::new();
        menu_button.add_css_class("app-finder-button");
        menu_button.set_has_frame(false);

        // Perthro rune
        let rune = gtk4::Label::new(Some("\u{16C8}"));
        rune.add_css_class("module-rune");
        menu_button.set_child(Some(&rune));

        // Popover
        let popover = gtk4::Popover::new();
        popover.add_css_class("app-finder-popover");
        popover.set_autohide(true);

        let popover_content = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        popover_content.set_margin_top(12);
        popover_content.set_margin_bottom(12);
        popover_content.set_margin_start(12);
        popover_content.set_margin_end(12);
        popover_content.set_size_request(320, -1);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let header_rune = gtk4::Label::new(Some("\u{16C8}"));
        header_rune.add_css_class("slider-header-rune");
        let header_label = gtk4::Label::new(Some("Yggdrasil (Apps)"));
        header_label.add_css_class("slider-header-label");
        header.append(&header_rune);
        header.append(&header_label);
        popover_content.append(&header);

        // Search entry
        let search_entry = gtk4::Entry::new();
        search_entry.set_placeholder_text(Some("Search the realms..."));
        search_entry.add_css_class("app-finder-search");
        search_entry.set_hexpand(true);
        popover_content.append(&search_entry);

        // Results list
        let results_scroll = gtk4::ScrolledWindow::new();
        results_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        results_scroll.set_max_content_height(400);
        results_scroll.set_propagate_natural_height(true);

        let results_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        results_scroll.set_child(Some(&results_box));
        popover_content.append(&results_scroll);

        // Populate initial results
        let entries_clone = entries.clone();
        let results_clone = results_box.clone();
        let popover_clone = popover.clone();
        populate_results(&results_clone, &entries_clone.borrow(), "", &popover_clone);

        // Search as user types with debounce
        let debounce_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let entries_search = entries.clone();
        let results_search = results_box.clone();
        let popover_search = popover.clone();
        search_entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let entries_c = entries_search.clone();
            let results_c = results_search.clone();
            let popover_c = popover_search.clone();
            let debounce = debounce_id.clone();

            // Cancel previous debounce if it hasn't fired yet
            if let Some(id) = debounce.borrow_mut().take() {
                id.remove();
            }

            // Share debounce ref with the timeout so it clears itself after firing
            // (timeout_add_local_once auto-removes the source, so we must not call
            // remove() on it again — clearing the stored ID prevents that)
            let debounce_for_timeout = debounce.clone();
            let id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(100),
                move || {
                    debounce_for_timeout.borrow_mut().take();
                    populate_results(&results_c, &entries_c.borrow(), &query, &popover_c);
                },
            );
            *debounce.borrow_mut() = Some(id);
        });

        // Reload entries when popover opens
        let entries_show = entries.clone();
        let results_show = results_box.clone();
        let search_show = search_entry.clone();
        let popover_show = popover.clone();
        popover.connect_show(move |_| {
            *entries_show.borrow_mut() = app_finder::load_desktop_entries();
            search_show.set_text("");
            populate_results(&results_show, &entries_show.borrow(), "", &popover_show);
            search_show.grab_focus();
        });

        popover.set_child(Some(&popover_content));
        menu_button.set_popover(Some(&popover));

        widget.append(&menu_button);

        Self { widget, entries }
    }
}

fn populate_results(
    results_box: &gtk4::Box,
    entries: &[app_finder::DesktopEntry],
    query: &str,
    popover: &gtk4::Popover,
) {
    // Clear existing
    while let Some(child) = results_box.first_child() {
        results_box.remove(&child);
    }

    let results = app_finder::search_entries(entries, query);
    if results.is_empty() {
        let empty = gtk4::Label::new(Some("No applications found"));
        empty.add_css_class("connectivity-empty");
        results_box.append(&empty);
        return;
    }

    for entry in results {
        let row = create_app_row(entry, popover);
        results_box.append(&row);
    }
}

fn create_app_row(entry: &app_finder::DesktopEntry, popover: &gtk4::Popover) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.add_css_class("app-finder-row");
    row.set_margin_top(2);
    row.set_margin_bottom(2);

    // Icon
    if let Some(icon_name) = &entry.icon {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_pixel_size(32);
        icon.add_css_class("app-finder-icon");
        row.append(&icon);
    } else {
        let icon = gtk4::Image::from_icon_name("application-x-executable");
        icon.set_pixel_size(32);
        icon.add_css_class("app-finder-icon");
        row.append(&icon);
    }

    // Name + comment
    let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    info_box.set_hexpand(true);

    let name_label = gtk4::Label::new(Some(&entry.name));
    name_label.add_css_class("app-finder-name");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    info_box.append(&name_label);

    if let Some(comment) = &entry.comment {
        let comment_label = gtk4::Label::new(Some(comment));
        comment_label.add_css_class("app-finder-comment");
        comment_label.set_halign(gtk4::Align::Start);
        comment_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        info_box.append(&comment_label);
    }

    row.append(&info_box);

    // Make the row clickable via a GestureClick
    let gesture = gtk4::GestureClick::new();
    let exec = entry.exec.clone();
    let name = entry.name.clone();
    let icon = entry.icon.clone();
    let comment = entry.comment.clone();
    let popover_clone = popover.clone();
    gesture.connect_released(move |_, _, _, _| {
        let entry = app_finder::DesktopEntry {
            name: name.clone(),
            exec: exec.clone(),
            icon: icon.clone(),
            comment: comment.clone(),
            categories: Vec::new(),
            no_display: false,
        };
        app_finder::launch_app(&entry);
        popover_clone.popdown();
    });
    row.add_controller(gesture);

    // Hover effect via CSS
    let motion = gtk4::EventControllerMotion::new();
    let row_hover = row.clone();
    motion.connect_enter(move |_, _, _| {
        row_hover.add_css_class("app-finder-row-hover");
    });
    let row_leave = row.clone();
    motion.connect_leave(move |_| {
        row_leave.remove_css_class("app-finder-row-hover");
    });
    row.add_controller(motion);

    row
}
