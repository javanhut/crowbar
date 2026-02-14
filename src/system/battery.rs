use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const BASE_PATH: &str = "/sys/class/power_supply";

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct BatteryInfo {
    pub name: String,
    pub capacity: i32,
    pub status: BatteryStatus,
    pub energy_now: i64,
    pub energy_full: i64,
    pub power_now: i64,
    pub time_remaining: Duration,
}

fn read_sysfs(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_sysfs_int(path: &Path) -> Option<i64> {
    read_sysfs(path)?.parse().ok()
}

pub fn find_batteries() -> Vec<String> {
    let Ok(entries) = fs::read_dir(BASE_PATH) else {
        return Vec::new();
    };

    entries
        .filter_map(|e| {
            let entry = e.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let type_path = PathBuf::from(BASE_PATH).join(&name).join("type");
            let type_str = read_sysfs(&type_path)?;
            if type_str == "Battery" {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

pub fn get_info(name: &str) -> Option<BatteryInfo> {
    let battery_path = PathBuf::from(BASE_PATH).join(name);
    if !battery_path.exists() {
        return None;
    }

    let present = read_sysfs(&battery_path.join("present"));
    if present.as_deref() != Some("1") {
        return None;
    }

    let capacity = read_sysfs_int(&battery_path.join("capacity")).unwrap_or(0) as i32;

    let status = match read_sysfs(&battery_path.join("status")).as_deref() {
        Some("Charging") => BatteryStatus::Charging,
        Some("Discharging") => BatteryStatus::Discharging,
        Some("Full") => BatteryStatus::Full,
        Some("Not charging") => BatteryStatus::NotCharging,
        _ => BatteryStatus::Unknown,
    };

    let energy_now = read_sysfs_int(&battery_path.join("energy_now")).unwrap_or(0);
    let energy_full = read_sysfs_int(&battery_path.join("energy_full")).unwrap_or(0);
    let power_now = read_sysfs_int(&battery_path.join("power_now")).unwrap_or(0);

    let time_remaining = calculate_time_remaining(&status, energy_now, energy_full, power_now);

    Some(BatteryInfo {
        name: name.to_string(),
        capacity,
        status,
        energy_now,
        energy_full,
        power_now,
        time_remaining,
    })
}

pub fn get_first_battery() -> Option<BatteryInfo> {
    let batteries = find_batteries();
    batteries.first().and_then(|name| get_info(name))
}

fn calculate_time_remaining(
    status: &BatteryStatus,
    energy_now: i64,
    energy_full: i64,
    power_now: i64,
) -> Duration {
    if power_now <= 0 {
        return Duration::ZERO;
    }

    let hours = match status {
        BatteryStatus::Discharging => energy_now as f64 / power_now as f64,
        BatteryStatus::Charging => {
            let remaining = energy_full - energy_now;
            if remaining > 0 {
                remaining as f64 / power_now as f64
            } else {
                0.0
            }
        }
        _ => return Duration::ZERO,
    };

    Duration::from_secs_f64(hours * 3600.0)
}

pub fn format_time_remaining(d: Duration) -> String {
    if d.is_zero() {
        return String::new();
    }
    let total_minutes = d.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
