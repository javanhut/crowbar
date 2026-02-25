use std::io::BufRead;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HyprEvent {
    Workspace(String),
    ActiveWindow(String),
    OpenWindow(String),
    CloseWindow(String),
    WindowTitle(String),
    CreateWorkspace(String),
    DestroyWorkspace(String),
}

pub struct EventListener {
    socket_path: PathBuf,
    running: Arc<AtomicBool>,
}

impl EventListener {
    pub fn new() -> Result<Self, String> {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set".to_string())?;

        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/run/user/1000".to_string());

        let socket_path =
            PathBuf::from(format!("{xdg_runtime}/hypr/{signature}/.socket2.sock"));

        Ok(Self {
            socket_path,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(
        &self,
        sender: async_channel::Sender<HyprEvent>,
    ) -> Result<(), String> {
        let stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to event socket: {e}"))?;

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stream);
            for line in reader.lines() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                let Ok(line) = line else {
                    break;
                };

                let line = line.trim().to_string();
                let Some((event_type, data)) = line.split_once(">>") else {
                    continue;
                };

                let event = match event_type {
                    "workspace" | "workspacev2" => {
                        HyprEvent::Workspace(data.to_string())
                    }
                    "activewindow" | "activewindowv2" => {
                        HyprEvent::ActiveWindow(data.to_string())
                    }
                    "openwindow" => HyprEvent::OpenWindow(data.to_string()),
                    "closewindow" => HyprEvent::CloseWindow(data.to_string()),
                    "windowtitle" | "windowtitlev2" => {
                        HyprEvent::WindowTitle(data.to_string())
                    }
                    "createworkspace" | "createworkspacev2" => {
                        HyprEvent::CreateWorkspace(data.to_string())
                    }
                    "destroyworkspace" | "destroyworkspacev2" => {
                        HyprEvent::DestroyWorkspace(data.to_string())
                    }
                    _ => continue,
                };

                if sender.send_blocking(event).is_err() {
                    break;
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
