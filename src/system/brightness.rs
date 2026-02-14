use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASE_PATH: &str = "/sys/class/backlight";

pub struct BrightnessInfo {
    pub device: String,
    pub brightness: i32,
    pub max_brightness: i32,
    pub percent: i32,
    pub available: bool,
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

pub fn get_first_backlight() -> Option<BrightnessInfo> {
    let devices = find_backlights();
    devices.first().and_then(|d| get_info(d))
}

pub fn set_brightness(_device: &str, percent: i32) {
    let percent = percent.clamp(1, 100);
    // Try brightnessctl first (handles permissions)
    let _ = Command::new("brightnessctl")
        .args(["set", &format!("{percent}%")])
        .status();
}
