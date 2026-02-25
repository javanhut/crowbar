use crate::hyprland::HyprlandClient;
use gtk4::prelude::*;
use std::rc::Rc;

pub struct Workspaces {
    pub widget: gtk4::Box,
    client: Rc<HyprlandClient>,
}

impl Workspaces {
    pub fn new(client: Rc<HyprlandClient>) -> Self {
        let widget = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        widget.add_css_class("workspaces");

        let ws = Self { widget, client };
        ws.refresh();
        ws
    }

    pub fn refresh(&self) {
        // Clear existing buttons
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }

        let Ok(workspaces) = self.client.workspaces() else {
            return;
        };
        let Ok(active) = self.client.active_workspace() else {
            return;
        };

        let mut workspaces = workspaces;
        workspaces.sort_by_key(|w| w.id);

        for ws in &workspaces {
            // Skip special workspaces
            if ws.id < 0 {
                continue;
            }

            let btn = gtk4::Button::with_label(&ws.id.to_string());
            btn.add_css_class("workspace-btn");

            if ws.id == active.id {
                btn.add_css_class("active");
            } else if ws.windows > 0 {
                btn.add_css_class("occupied");
            }

            let client = self.client.clone();
            let ws_id = ws.id;
            btn.connect_clicked(move |_| {
                let _ = client.switch_workspace(ws_id);
            });

            self.widget.append(&btn);
        }
    }
}
