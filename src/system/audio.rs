use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

// === Backend detection ===

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BackendType {
    None  = 0,
    Pactl = 1,
    Wpctl = 2,
}

impl BackendType {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Pactl,
            2 => Self::Wpctl,
            _ => Self::None,
        }
    }
}

static BACKEND: OnceLock<Arc<AtomicU8>> = OnceLock::new();

fn get_backend_arc() -> Arc<AtomicU8> {
    BACKEND.get_or_init(|| Arc::new(AtomicU8::new(0))).clone()
}

pub fn current_backend() -> BackendType {
    BackendType::from_u8(get_backend_arc().load(Ordering::Relaxed))
}

fn set_backend(b: BackendType) {
    get_backend_arc().store(b as u8, Ordering::Relaxed);
}

pub fn detect_backend() -> BackendType {
    // Try pactl (preferred: full feature set)
    if Command::new("pactl")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return BackendType::Pactl;
    }
    // Fall back to wpctl (WirePlumber native)
    if Command::new("wpctl")
        .arg("status")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return BackendType::Wpctl;
    }
    BackendType::None
}

// === Data types ===

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

// === Public dispatch functions ===

pub fn get_info() -> AudioInfo {
    match current_backend() {
        BackendType::Pactl => get_info_pactl(),
        BackendType::Wpctl => get_info_wpctl(),
        BackendType::None  => AudioInfo { volume: 0, muted: false, available: false },
    }
}

pub fn set_volume(percent: i32) {
    match current_backend() {
        BackendType::Pactl => set_volume_pactl(percent),
        BackendType::Wpctl => set_volume_wpctl(percent),
        BackendType::None  => {}
    }
}

pub fn toggle_mute() {
    match current_backend() {
        BackendType::Pactl => toggle_mute_pactl(),
        BackendType::Wpctl => toggle_mute_wpctl(),
        BackendType::None  => {}
    }
}

pub fn get_source_info() -> SourceInfo {
    match current_backend() {
        BackendType::Pactl => get_source_info_pactl(),
        BackendType::Wpctl => get_source_info_wpctl(),
        BackendType::None  => SourceInfo { volume: 0, muted: false, available: false },
    }
}

pub fn set_source_volume(percent: i32) {
    match current_backend() {
        BackendType::Pactl => set_source_volume_pactl(percent),
        BackendType::Wpctl => set_source_volume_wpctl(percent),
        BackendType::None  => {}
    }
}

pub fn toggle_source_mute() {
    match current_backend() {
        BackendType::Pactl => toggle_source_mute_pactl(),
        BackendType::Wpctl => toggle_source_mute_wpctl(),
        BackendType::None  => {}
    }
}

pub fn list_sink_inputs() -> Vec<SinkInput> {
    match current_backend() {
        BackendType::Pactl => list_sink_inputs_pactl(),
        _ => Vec::new(),
    }
}

pub fn list_source_outputs() -> Vec<SourceOutput> {
    match current_backend() {
        BackendType::Pactl => list_source_outputs_pactl(),
        _ => Vec::new(),
    }
}

pub fn set_sink_input_volume(index: u32, percent: i32) {
    if current_backend() == BackendType::Pactl {
        set_sink_input_volume_pactl(index, percent);
    }
}

pub fn set_sink_input_mute(index: u32, toggle: &str) {
    if current_backend() == BackendType::Pactl {
        set_sink_input_mute_pactl(index, toggle);
    }
}

pub fn set_source_output_volume(index: u32, percent: i32) {
    if current_backend() == BackendType::Pactl {
        set_source_output_volume_pactl(index, percent);
    }
}

pub fn set_source_output_mute(index: u32, toggle: &str) {
    if current_backend() == BackendType::Pactl {
        set_source_output_mute_pactl(index, toggle);
    }
}

pub fn move_sink_input(index: u32, sink_name: &str) {
    if current_backend() == BackendType::Pactl {
        move_sink_input_pactl(index, sink_name);
    }
}

pub fn move_source_output(index: u32, source_name: &str) {
    if current_backend() == BackendType::Pactl {
        move_source_output_pactl(index, source_name);
    }
}

pub fn list_cards() -> Vec<AudioCard> {
    match current_backend() {
        BackendType::Pactl => list_cards_pactl(),
        _ => Vec::new(),
    }
}

pub fn set_card_profile(card_name: &str, profile: &str) {
    if current_backend() == BackendType::Pactl {
        set_card_profile_pactl(card_name, profile);
    }
}

pub fn list_sinks() -> Vec<AudioDevice> {
    match current_backend() {
        BackendType::Pactl => list_sinks_pactl(),
        BackendType::Wpctl => list_sinks_wpctl(),
        BackendType::None  => Vec::new(),
    }
}

pub fn list_sources() -> Vec<AudioDevice> {
    match current_backend() {
        BackendType::Pactl => list_sources_pactl(),
        BackendType::Wpctl => list_sources_wpctl(),
        BackendType::None  => Vec::new(),
    }
}

pub fn set_default_sink(name: &str) {
    match current_backend() {
        BackendType::Pactl => set_default_sink_pactl(name),
        BackendType::Wpctl => set_default_sink_wpctl(name),
        BackendType::None  => {}
    }
}

pub fn set_default_source(name: &str) {
    match current_backend() {
        BackendType::Pactl => set_default_source_pactl(name),
        BackendType::Wpctl => set_default_source_wpctl(name),
        BackendType::None  => {}
    }
}

// === pactl (PulseAudio / pipewire-pulse) backend ===

fn get_info_pactl() -> AudioInfo {
    let volume = match get_volume_pactl() {
        Ok(v) => v,
        Err(_) => return AudioInfo { volume: 0, muted: false, available: false },
    };
    AudioInfo {
        volume,
        muted: get_muted_pactl(),
        available: true,
    }
}

fn get_volume_pactl() -> Result<i32, String> {
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

fn get_muted_pactl() -> bool {
    Command::new("pactl")
        .args(["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("yes"))
        .unwrap_or(false)
}

fn set_volume_pactl(percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{percent}%")])
        .status();
}

fn toggle_mute_pactl() {
    let _ = Command::new("pactl")
        .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        .status();
}

fn get_source_info_pactl() -> SourceInfo {
    let volume = match get_source_volume_pactl() {
        Ok(v) => v,
        Err(_) => return SourceInfo { volume: 0, muted: false, available: false },
    };
    SourceInfo {
        volume,
        muted: get_source_muted_pactl(),
        available: true,
    }
}

fn get_source_volume_pactl() -> Result<i32, String> {
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

fn get_source_muted_pactl() -> bool {
    Command::new("pactl")
        .args(["get-source-mute", "@DEFAULT_SOURCE@"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("yes"))
        .unwrap_or(false)
}

fn set_source_volume_pactl(percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args(["set-source-volume", "@DEFAULT_SOURCE@", &format!("{percent}%")])
        .status();
}

fn toggle_source_mute_pactl() {
    let _ = Command::new("pactl")
        .args(["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])
        .status();
}

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

fn list_sink_inputs_pactl() -> Vec<SinkInput> {
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

            Some(SinkInput { index, name, sink_name, volume, muted })
        })
        .collect()
}

fn list_source_outputs_pactl() -> Vec<SourceOutput> {
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

            Some(SourceOutput { index, name, source_name, volume, muted })
        })
        .collect()
}

fn set_sink_input_volume_pactl(index: u32, percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args([
            "set-sink-input-volume",
            &index.to_string(),
            &format!("{percent}%"),
        ])
        .status();
}

fn set_sink_input_mute_pactl(index: u32, toggle: &str) {
    let _ = Command::new("pactl")
        .args(["set-sink-input-mute", &index.to_string(), toggle])
        .status();
}

fn set_source_output_volume_pactl(index: u32, percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("pactl")
        .args([
            "set-source-output-volume",
            &index.to_string(),
            &format!("{percent}%"),
        ])
        .status();
}

fn set_source_output_mute_pactl(index: u32, toggle: &str) {
    let _ = Command::new("pactl")
        .args(["set-source-output-mute", &index.to_string(), toggle])
        .status();
}

fn move_sink_input_pactl(index: u32, sink_name: &str) {
    let _ = Command::new("pactl")
        .args(["move-sink-input", &index.to_string(), sink_name])
        .status();
}

fn move_source_output_pactl(index: u32, source_name: &str) {
    let _ = Command::new("pactl")
        .args(["move-source-output", &index.to_string(), source_name])
        .status();
}

fn list_cards_pactl() -> Vec<AudioCard> {
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

            Some(AudioCard { index, name, description, active_profile, profiles })
        })
        .collect()
}

fn set_card_profile_pactl(card_name: &str, profile: &str) {
    let _ = Command::new("pactl")
        .args(["set-card-profile", card_name, profile])
        .status();
}

fn get_default_sink_name_pactl() -> Option<String> {
    Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn get_default_source_name_pactl() -> Option<String> {
    Command::new("pactl")
        .args(["get-default-source"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn list_sinks_pactl() -> Vec<AudioDevice> {
    let default = get_default_sink_name_pactl().unwrap_or_default();
    parse_devices_pactl("sinks", &default)
}

fn list_sources_pactl() -> Vec<AudioDevice> {
    let default = get_default_source_name_pactl().unwrap_or_default();
    let mut devices = parse_devices_pactl("sources", &default);
    // Filter out monitor sources (virtual loopback devices)
    devices.retain(|d| !d.name.ends_with(".monitor"));
    devices
}

fn parse_devices_pactl(kind: &str, default_name: &str) -> Vec<AudioDevice> {
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

fn set_default_sink_pactl(name: &str) {
    let _ = Command::new("pactl")
        .args(["set-default-sink", name])
        .status();
}

fn set_default_source_pactl(name: &str) {
    let _ = Command::new("pactl")
        .args(["set-default-source", name])
        .status();
}

// === wpctl (WirePlumber native) backend ===

// Parses "Volume: 0.40" or "Volume: 0.40 [MUTED]" → (percent, muted)
fn parse_wpctl_volume(text: &str) -> Option<(i32, bool)> {
    let line = text.lines().next()?.trim();
    let after = line.strip_prefix("Volume:")?.trim();
    let muted = after.contains("[MUTED]");
    let frac: f64 = after.split_whitespace().next()?.parse().ok()?;
    Some((((frac * 100.0).round() as i32).clamp(0, 150), muted))
}

// Parses `wpctl status` Sinks/Sources sections → Vec<AudioDevice>
// AudioDevice.name stores the numeric node ID (needed for wpctl set-default)
fn parse_wpctl_devices(status: &str, section: &str) -> Vec<AudioDevice> {
    let mut devices = Vec::new();
    let mut in_section = false;

    for line in status.lines() {
        if !in_section {
            if line.contains(section) && line.contains(':') {
                in_section = true;
            }
            continue;
        }

        // Exit section when line doesn't start with the tree branch character
        if !line.starts_with(" \u{2502}") {
            break;
        }

        // Device lines contain the ". " pattern (number followed by dot-space)
        if let Some(dot_pos) = line.find(". ") {
            let is_default = line.contains('*');

            // Extract numeric ID from before the dot
            let before_dot = &line[..dot_pos];
            let id: String = before_dot.chars().filter(|c| c.is_ascii_digit()).collect();
            if id.is_empty() {
                continue;
            }

            // Extract description: after ". " up to " [" bracket
            let after_dot = &line[dot_pos + 2..];
            let description = if let Some(bracket_pos) = after_dot.find(" [") {
                after_dot[..bracket_pos].trim().to_string()
            } else {
                after_dot.trim().to_string()
            };

            if !description.is_empty() {
                devices.push(AudioDevice {
                    name: id,
                    description,
                    is_default,
                });
            }
        }
    }

    devices
}

fn get_info_wpctl() -> AudioInfo {
    let Ok(output) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    else {
        return AudioInfo { volume: 0, muted: false, available: false };
    };
    let text = String::from_utf8_lossy(&output.stdout);
    match parse_wpctl_volume(&text) {
        Some((volume, muted)) => AudioInfo { volume, muted, available: true },
        None => AudioInfo { volume: 0, muted: false, available: false },
    }
}

fn get_source_info_wpctl() -> SourceInfo {
    let Ok(output) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"])
        .output()
    else {
        return SourceInfo { volume: 0, muted: false, available: false };
    };
    let text = String::from_utf8_lossy(&output.stdout);
    match parse_wpctl_volume(&text) {
        Some((volume, muted)) => SourceInfo { volume, muted, available: true },
        None => SourceInfo { volume: 0, muted: false, available: false },
    }
}

fn set_volume_wpctl(percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{percent}%")])
        .status();
}

fn toggle_mute_wpctl() {
    let _ = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
        .status();
}

fn set_source_volume_wpctl(percent: i32) {
    let percent = percent.clamp(0, 150);
    let _ = Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &format!("{percent}%")])
        .status();
}

fn toggle_source_mute_wpctl() {
    let _ = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
        .status();
}

fn list_sinks_wpctl() -> Vec<AudioDevice> {
    let Ok(output) = Command::new("wpctl").arg("status").output() else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    parse_wpctl_devices(&text, "Sinks")
}

fn list_sources_wpctl() -> Vec<AudioDevice> {
    let Ok(output) = Command::new("wpctl").arg("status").output() else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = parse_wpctl_devices(&text, "Sources");
    // Filter out monitor sources (loopback devices)
    devices.retain(|d| !d.description.contains("Monitor of"));
    devices
}

fn set_default_sink_wpctl(id: &str) {
    let _ = Command::new("wpctl").args(["set-default", id]).status();
}

fn set_default_source_wpctl(id: &str) {
    let _ = Command::new("wpctl").args(["set-default", id]).status();
}

// === Event listener ===

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
            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                let detected = detect_backend();
                set_backend(detected);

                match detected {
                    BackendType::None => {
                        // No audio backend available — retry after a pause
                        std::thread::sleep(Duration::from_secs(3));
                    }
                    BackendType::Pactl => {
                        // Send initial refresh, then stream events
                        let _ = sender.send_blocking(());
                        let Ok(mut child) = Command::new("pactl")
                            .arg("subscribe")
                            .stdout(Stdio::piped())
                            .spawn()
                        else {
                            std::thread::sleep(Duration::from_secs(3));
                            continue;
                        };

                        let stdout = child.stdout.take().unwrap();
                        for line in BufReader::new(stdout).lines() {
                            if !running.load(Ordering::SeqCst) {
                                child.kill().ok();
                                return;
                            }
                            let Ok(line) = line else { break };
                            if line.contains("sink")
                                || line.contains("source")
                                || line.contains("server")
                            {
                                let _ = sender.send_blocking(());
                            }
                        }

                        child.kill().ok();

                        // pactl subscribe exited (daemon restarted?) — retry after a pause
                        if running.load(Ordering::SeqCst) {
                            std::thread::sleep(Duration::from_secs(2));
                            let _ = sender.send_blocking(());
                        }
                    }
                    BackendType::Wpctl => {
                        // WirePlumber has no subscribe-style event stream — poll every 2s
                        loop {
                            if !running.load(Ordering::SeqCst) {
                                return;
                            }
                            let _ = sender.send_blocking(());
                            std::thread::sleep(Duration::from_secs(2));
                            // Check if backend has changed (e.g. pipewire-pulse came up)
                            let now = detect_backend();
                            if now != BackendType::Wpctl {
                                set_backend(now);
                                let _ = sender.send_blocking(());
                                break; // back to outer loop to re-detect
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
