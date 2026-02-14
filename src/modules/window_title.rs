use crate::hyprland::HyprlandClient;
use gtk4::pango;
use gtk4::prelude::*;
use std::rc::Rc;

pub struct WindowTitle {
    pub widget: gtk4::Label,
    client: Rc<HyprlandClient>,
}

impl WindowTitle {
    pub fn new(client: Rc<HyprlandClient>) -> Self {
        let widget = gtk4::Label::new(None);
        widget.add_css_class("window-title");
        widget.set_ellipsize(pango::EllipsizeMode::End);
        widget.set_max_width_chars(50);
        widget.set_halign(gtk4::Align::Start);
        widget.set_hexpand(true);

        let wt = Self { widget, client };
        wt.refresh();
        wt
    }

    pub fn refresh(&self) {
        let Ok(window) = self.client.active_window() else {
            self.widget.set_text("");
            return;
        };

        let title = if window.title.is_empty() {
            &window.class
        } else {
            &window.title
        };
        self.widget.set_text(title);
    }
}
