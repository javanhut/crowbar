use std::fs;
use std::path::{Path, PathBuf};

const CPUFREQ_PATH: &str = "/sys/devices/system/cpu/cpu0/cpufreq";
const HWMON_PATH: &str = "/sys/class/hwmon";

#[derive(Debug, Clone, PartialEq)]
pub enum Governor {
    Performance,
    Powersave,
    Ondemand,
    Conservative,
    Schedutil,
    Unknown(String),
}

pub struct PowerInfo {
    pub governor: Governor,
    pub temperature: f64,
    pub frequency_mhz: i32,
    pub has_temp: bool,
}

fn read_sysfs(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_sysfs_int(path: &Path) -> Option<i64> {
    read_sysfs(path)?.parse().ok()
}

pub fn get_governor() -> Governor {
    let path = PathBuf::from(CPUFREQ_PATH).join("scaling_governor");
    match read_sysfs(&path).as_deref() {
        Some("performance") => Governor::Performance,
        Some("powersave") => Governor::Powersave,
        Some("ondemand") => Governor::Ondemand,
        Some("conservative") => Governor::Conservative,
        Some("schedutil") => Governor::Schedutil,
        Some(other) => Governor::Unknown(other.to_string()),
        None => Governor::Unknown("unknown".to_string()),
    }
}

pub fn get_frequency_mhz() -> i32 {
    let path = PathBuf::from(CPUFREQ_PATH).join("scaling_cur_freq");
    read_sysfs_int(&path).map(|f| (f / 1000) as i32).unwrap_or(0)
}

fn find_cpu_temp_hwmon() -> Option<PathBuf> {
    let entries = fs::read_dir(HWMON_PATH).ok()?;

    for entry in entries.flatten() {
        let name_path = entry.path().join("name");
        if let Some(name) = read_sysfs(&name_path) {
            if name == "k10temp" || name == "coretemp" || name == "cpu_thermal" {
                return Some(entry.path());
            }
        }
    }
    None
}

pub fn get_temperature() -> Option<f64> {
    let hwmon = find_cpu_temp_hwmon()?;
    let temp = read_sysfs_int(&hwmon.join("temp1_input"))?;
    Some(temp as f64 / 1000.0)
}

pub fn get_info() -> PowerInfo {
    let (temperature, has_temp) = match get_temperature() {
        Some(t) => (t, true),
        None => (0.0, false),
    };

    PowerInfo {
        governor: get_governor(),
        temperature,
        frequency_mhz: get_frequency_mhz(),
        has_temp,
    }
}

impl Governor {
    pub fn display_name(&self) -> &str {
        match self {
            Governor::Performance => "Performance",
            Governor::Powersave => "Power Saver",
            Governor::Ondemand => "On Demand",
            Governor::Conservative => "Conservative",
            Governor::Schedutil => "Balanced",
            Governor::Unknown(s) => s,
        }
    }
}

pub fn format_temperature(temp: f64) -> String {
    format!("{temp:.0}°C")
}
