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

pub struct SourceInfo {
    pub volume: i32,
    pub muted: bool,
    pub available: bool,
}

pub struct SinkInput {
    pub index: u32,
    pub name: String,
    pub sink_name: String,
    pub volume: i32,
    pub muted: bool,
}

pub struct SourceOutput {
    pub index: u32,
    pub name: String,
    pub source_name: String,
    pub volume: i32,
    pub muted: bool,
}

pub struct AudioCard {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub active_profile: String,
    pub profiles: Vec<AudioProfile>,
}

pub struct AudioProfile {
    pub name: String,
    pub description: String,
    pub available: bool,
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

// === Source (Microphone) Controls ===

pub fn get_source_info() -> SourceInfo {
    let volume = match get_source_volume() {
        Ok(v) => v,
        Err(_) => return SourceInfo { volume: 0, muted: false, available: false },
    };

    SourceInfo {
        volume,
        muted: get_source_muted(),
        available: true,
    }
}

fn get_source_volume() -> Result<i32, String> {
    let output = Command::new("pactl")
        .args(["get-source-volume", "@DEFAULT_SOURCE@"])
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);
    for part in text.split_whitespace() {
        if let Some(pct) = part.strip_suffix('%') {
            if let Ok(vol) = pct.parse::<i32>() {
                return Ok(vol);
            }
        }
    }
    Err("Could not parse source volume".into())
}

fn get_source_muted() -> bool {
    Command::new("pactl")
        .args(["get-source-mute", "@DEFAULT_SOURCE@"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("yes"))
        .unwrap_or(false)
}

pub fn set_source_volume(percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args(["set-source-volume", "@DEFAULT_SOURCE@", &format!("{percent}%")])
        .status();
}

pub fn toggle_source_mute() {
    let _ = Command::new("pactl")
        .args(["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])
        .status();
}

// === Per-App Stream Controls ===

fn parse_volume_percent(volume_obj: &serde_json::Value) -> i32 {
    // pactl JSON volume objects have channel entries like {"front-left": {"value": 65536, "value_percent": "100%", ...}}
    if let Some(obj) = volume_obj.as_object() {
        for (_channel, val) in obj {
            if let Some(pct_str) = val.get("value_percent").and_then(|v| v.as_str()) {
                if let Some(num) = pct_str.strip_suffix('%') {
                    if let Ok(v) = num.trim().parse::<i32>() {
                        return v;
                    }
                }
            }
        }
    }
    100
}

pub fn list_sink_inputs() -> Vec<SinkInput> {
    let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sink-inputs"])
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
            let index = item.get("index")?.as_u64()? as u32;
            let props = item.get("properties");

            // Get application name with fallbacks
            let name = props
                .and_then(|p| p.get("application.name"))
                .and_then(|v| v.as_str())
                .or_else(|| {
                    props
                        .and_then(|p| p.get("media.name"))
                        .and_then(|v| v.as_str())
                })
                .or_else(|| item.get("name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown")
                .to_string();

            // Filter out internal PipeWire streams
            let module = props
                .and_then(|p| p.get("application.process.binary"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if module == "pipewire" || module == "pipewire-pulse" {
                return None;
            }

            let sink_name = item
                .get("sink")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let volume = item
                .get("volume")
                .map(|v| parse_volume_percent(v))
                .unwrap_or(100);

            let muted = item
                .get("mute")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Some(SinkInput {
                index,
                name,
                sink_name,
                volume,
                muted,
            })
        })
        .collect()
}

pub fn list_source_outputs() -> Vec<SourceOutput> {
    let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "source-outputs"])
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
            let index = item.get("index")?.as_u64()? as u32;
            let props = item.get("properties");

            let name = props
                .and_then(|p| p.get("application.name"))
                .and_then(|v| v.as_str())
                .or_else(|| {
                    props
                        .and_then(|p| p.get("media.name"))
                        .and_then(|v| v.as_str())
                })
                .or_else(|| item.get("name").and_then(|v| v.as_str()))
                .unwrap_or("Unknown")
                .to_string();

            let module = props
                .and_then(|p| p.get("application.process.binary"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if module == "pipewire" || module == "pipewire-pulse" {
                return None;
            }

            let source_name = item
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let volume = item
                .get("volume")
                .map(|v| parse_volume_percent(v))
                .unwrap_or(100);

            let muted = item
                .get("mute")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Some(SourceOutput {
                index,
                name,
                source_name,
                volume,
                muted,
            })
        })
        .collect()
}

pub fn set_sink_input_volume(index: u32, percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args([
            "set-sink-input-volume",
            &index.to_string(),
            &format!("{percent}%"),
        ])
        .status();
}

pub fn set_sink_input_mute(index: u32, toggle: &str) {
    let _ = Command::new("pactl")
        .args(["set-sink-input-mute", &index.to_string(), toggle])
        .status();
}

pub fn set_source_output_volume(index: u32, percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args([
            "set-source-output-volume",
            &index.to_string(),
            &format!("{percent}%"),
        ])
        .status();
}

pub fn set_source_output_mute(index: u32, toggle: &str) {
    let _ = Command::new("pactl")
        .args(["set-source-output-mute", &index.to_string(), toggle])
        .status();
}

pub fn move_sink_input(index: u32, sink_name: &str) {
    let _ = Command::new("pactl")
        .args(["move-sink-input", &index.to_string(), sink_name])
        .status();
}

pub fn move_source_output(index: u32, source_name: &str) {
    let _ = Command::new("pactl")
        .args(["move-source-output", &index.to_string(), source_name])
        .status();
}

// === Audio Card/Profile Controls ===

pub fn list_cards() -> Vec<AudioCard> {
    let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "cards"])
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
            let index = item.get("index")?.as_u64()? as u32;
            let name = item.get("name")?.as_str()?.to_string();

            let description = item
                .get("properties")
                .and_then(|p| p.get("device.description"))
                .and_then(|v| v.as_str())
                .unwrap_or(&name)
                .to_string();

            let active_profile = item
                .get("active_profile")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let profiles = item
                .get("profiles")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(prof_name, prof_val)| {
                            let available = prof_val
                                .get("available")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true);
                            let prof_desc = prof_val
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or(prof_name)
                                .to_string();
                            Some(AudioProfile {
                                name: prof_name.clone(),
                                description: prof_desc,
                                available,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            Some(AudioCard {
                index,
                name,
                description,
                active_profile,
                profiles,
            })
        })
        .collect()
}

pub fn set_card_profile(card_name: &str, profile: &str) {
    let _ = Command::new("pactl")
        .args(["set-card-profile", card_name, profile])
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
            while running.load(Ordering::SeqCst) {
                let Ok(mut child) = Command::new("pactl")
                    .arg("subscribe")
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                else {
                    // pactl not available yet — wait and retry
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    continue;
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

                // pactl subscribe exited (daemon restarted?) — retry after a pause
                if running.load(Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    // Trigger a refresh so UI picks up new state after reconnect
                    let _ = sender.send_blocking(());
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
