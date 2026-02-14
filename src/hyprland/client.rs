use serde::Deserialize;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub windows: i32,
    #[serde(default)]
    pub monitor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Window {
    pub address: String,
    #[serde(default)]
    pub title: String,
    pub class: String,
    #[serde(default)]
    pub workspace: WorkspaceRef,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WorkspaceRef {
    pub id: i32,
    pub name: String,
}

pub struct HyprlandClient {
    socket_path: PathBuf,
}

impl HyprlandClient {
    pub fn new() -> Result<Self, String> {
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set".to_string())?;

        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/run/user/1000".to_string());

        let socket_path = PathBuf::from(format!("{xdg_runtime}/hypr/{signature}/.socket.sock"));

        if !socket_path.exists() {
            return Err(format!("Hyprland socket not found: {}", socket_path.display()));
        }

        Ok(Self { socket_path })
    }

    fn send_command(&self, cmd: &str) -> Result<String, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to Hyprland: {e}"))?;

        stream
            .write_all(cmd.as_bytes())
            .map_err(|e| format!("Failed to send command: {e}"))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| format!("Failed to read response: {e}"))?;

        Ok(response)
    }

    fn json_command<T: serde::de::DeserializeOwned>(&self, cmd: &str) -> Result<T, String> {
        let response = self.send_command(&format!("j/{cmd}"))?;
        serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse JSON: {e}"))
    }

    pub fn workspaces(&self) -> Result<Vec<Workspace>, String> {
        self.json_command("workspaces")
    }

    pub fn active_workspace(&self) -> Result<Workspace, String> {
        self.json_command("activeworkspace")
    }

    pub fn active_window(&self) -> Result<Window, String> {
        self.json_command("activewindow")
    }

    pub fn clients(&self) -> Result<Vec<Window>, String> {
        self.json_command("clients")
    }

    pub fn switch_workspace(&self, id: i32) -> Result<(), String> {
        self.send_command(&format!("dispatch workspace {id}"))?;
        Ok(())
    }

    pub fn focus_window(&self, address: &str) -> Result<(), String> {
        self.send_command(&format!("dispatch focuswindow address:{address}"))?;
        Ok(())
    }

    pub fn close_window(&self, address: &str) -> Result<(), String> {
        self.send_command(&format!("dispatch closewindow address:{address}"))?;
        Ok(())
    }

    pub fn minimize_window(&self, address: &str) -> Result<(), String> {
        self.send_command(&format!(
            "dispatch movetoworkspacesilent special:minimized,address:{address}"
        ))?;
        Ok(())
    }

    pub fn restore_window(&self, address: &str) -> Result<(), String> {
        self.send_command(&format!("dispatch movetoworkspace e+0,address:{address}"))?;
        self.focus_window(address)
    }

    pub fn dispatch(&self, cmd: &str) -> Result<(), String> {
        self.send_command(&format!("dispatch {cmd}"))?;
        Ok(())
    }
}
