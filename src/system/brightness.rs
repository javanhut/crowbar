use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASE_PATH: &str = "/sys/class/backlight";

#[allow(dead_code)]
pub struct BrightnessInfo {
    pub device: String,
    pub brightness: i32,
    pub max_brightness: i32,
    pub percent: i32,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NightModeBackend {
    Wlsunset,
    Gammastep,
    None,
}

fn read_sysfs(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_sysfs_int(path: &Path) -> Option<i32> {
    read_sysfs(path)?.parse().ok()
}

pub fn find_backlights() -> Vec<String> {
    let Ok(entries) = fs::read_dir(BASE_PATH) else {
        return Vec::new();
    };

    entries
        .filter_map(|e| {
            let entry = e.ok()?;
            Some(entry.file_name().to_string_lossy().to_string())
        })
        .collect()
}

pub fn get_info(device: &str) -> Option<BrightnessInfo> {
    let device_path = PathBuf::from(BASE_PATH).join(device);
    if !device_path.exists() {
        return None;
    }

    let max = read_sysfs_int(&device_path.join("max_brightness"))?;
    let current = read_sysfs_int(&device_path.join("brightness"))?;

    let percent = if max > 0 { (current * 100) / max } else { 0 };

    Some(BrightnessInfo {
        device: device.to_string(),
        brightness: current,
        max_brightness: max,
        percent,
        available: true,
    })
}

pub fn set_brightness(_device: &str, percent: i32) {
    let percent = percent.clamp(1, 100);
    // Try brightnessctl first (handles permissions)
    let _ = Command::new("brightnessctl")
        .args(["set", &format!("{percent}%")])
        .status();
}

pub fn detect_night_backend() -> NightModeBackend {
    if Command::new("which")
        .arg("wlsunset")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return NightModeBackend::Wlsunset;
    }

    if Command::new("which")
        .arg("gammastep")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return NightModeBackend::Gammastep;
    }

    NightModeBackend::None
}

pub fn is_night_mode_active() -> bool {
    Command::new("pgrep")
        .args(["-x", "wlsunset"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("pgrep")
            .args(["-x", "gammastep"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

pub fn start_night_mode(temperature: i32) -> Result<(), String> {
    // Kill any existing instances first
    stop_night_mode();

    let backend = detect_night_backend();
    match backend {
        NightModeBackend::Wlsunset => {
            let temp_str = temperature.to_string();
            Command::new("wlsunset")
                .args(["-T", &temp_str, "-t", &temp_str])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to start wlsunset: {e}"))?;
            Ok(())
        }
        NightModeBackend::Gammastep => {
            let temp_str = temperature.to_string();
            Command::new("gammastep")
                .args(["-O", &temp_str])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to start gammastep: {e}"))?;
            Ok(())
        }
        NightModeBackend::None => Err("No night mode backend available".to_string()),
    }
}

pub fn stop_night_mode() {
    let _ = Command::new("pkill").args(["-x", "wlsunset"]).status();
    let _ = Command::new("pkill").args(["-x", "gammastep"]).status();
}

pub fn set_night_temperature(temperature: i32) {
    if is_night_mode_active() {
        let _ = start_night_mode(temperature);
    }
}
