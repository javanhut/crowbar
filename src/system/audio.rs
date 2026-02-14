use std::io::BufRead;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AudioInfo {
    pub volume: i32,
    pub muted: bool,
    pub available: bool,
}

pub struct AudioDevice {
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

pub fn get_info() -> AudioInfo {
    let volume = match get_volume() {
        Ok(v) => v,
        Err(_) => return AudioInfo { volume: 0, muted: false, available: false },
    };

    AudioInfo {
        volume,
        muted: get_muted(),
        available: true,
    }
}

fn get_volume() -> Result<i32, String> {
    let output = Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Parse volume percentage from output like: "Volume: front-left: 55707 /  85% / ..."
    for part in text.split_whitespace() {
        if let Some(pct) = part.strip_suffix('%') {
            if let Ok(vol) = pct.parse::<i32>() {
                return Ok(vol);
            }
        }
    }
    Err("Could not parse volume".into())
}

fn get_muted() -> bool {
    Command::new("pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("yes"))
        .unwrap_or(false)
}

pub fn set_volume(percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{percent}%")])
        .status();
}

pub fn toggle_mute() {
    let _ = Command::new("pactl")
        .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        .status();
}

pub fn get_default_sink_name() -> Option<String> {
    Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn get_default_source_name() -> Option<String> {
    Command::new("pactl")
        .args(["get-default-source"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn list_sinks() -> Vec<AudioDevice> {
    let default = get_default_sink_name().unwrap_or_default();
    parse_devices("sinks", &default)
}

pub fn list_sources() -> Vec<AudioDevice> {
    let default = get_default_source_name().unwrap_or_default();
    let mut devices = parse_devices("sources", &default);
    // Filter out monitor sources (virtual loopback devices)
    devices.retain(|d| !d.name.ends_with(".monitor"));
    devices
}

fn parse_devices(kind: &str, default_name: &str) -> Vec<AudioDevice> {
    let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", kind])
        .output()
    else {
        return Vec::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };

    let Some(arr) = json.as_array() else {
        return Vec::new();
    };

    arr.iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.to_string();
            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(&name)
                .to_string();
            Some(AudioDevice {
                is_default: name == default_name,
                name,
                description,
            })
        })
        .collect()
}

pub fn set_default_sink(name: &str) {
    let _ = Command::new("pactl")
        .args(["set-default-sink", name])
        .status();
}

pub fn set_default_source(name: &str) {
    let _ = Command::new("pactl")
        .args(["set-default-source", name])
        .status();
}

pub struct AudioEventListener {
    running: Arc<AtomicBool>,
}

impl AudioEventListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, sender: async_channel::Sender<()>) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        std::thread::spawn(move || {
            let Ok(mut child) = Command::new("pactl")
                .arg("subscribe")
                .stdout(std::process::Stdio::piped())
                .spawn()
            else {
                return;
            };

            let stdout = child.stdout.take().unwrap();
            let reader = std::io::BufReader::new(stdout);

            for line in reader.lines() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(line) = line else { break };
                if line.contains("sink") || line.contains("source") || line.contains("server") {
                    let _ = sender.send_blocking(());
                }
            }

            let _ = child.kill();
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
