use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(Debug, Clone, PartialEq)]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSaver,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProfileMethod {
    PowerProfilesDaemon,
    CpuGovernor,
    None,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProfileInfo {
    pub available: Vec<PowerProfile>,
    pub active: PowerProfile,
    pub method: ProfileMethod,
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

pub fn detect_profile_method() -> ProfileMethod {
    // Try powerprofilesctl first
    if let Ok(output) = Command::new("powerprofilesctl").arg("list").output() {
        if output.status.success() {
            return ProfileMethod::PowerProfilesDaemon;
        }
    }

    // Fall back to checking cpu governor availability
    let path = PathBuf::from(CPUFREQ_PATH).join("scaling_available_governors");
    if read_sysfs(&path).is_some() {
        return ProfileMethod::CpuGovernor;
    }

    ProfileMethod::None
}

pub fn get_profiles() -> ProfileInfo {
    let method = detect_profile_method();

    match method {
        ProfileMethod::PowerProfilesDaemon => get_profiles_ppd(),
        ProfileMethod::CpuGovernor => get_profiles_governor(),
        ProfileMethod::None => ProfileInfo {
            available: Vec::new(),
            active: PowerProfile::Balanced,
            method: ProfileMethod::None,
        },
    }
}

fn get_profiles_ppd() -> ProfileInfo {
    let mut available = Vec::new();
    let mut active = PowerProfile::Balanced;

    if let Ok(output) = Command::new("powerprofilesctl").arg("list").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let trimmed = line.trim().trim_start_matches('*').trim();
                match trimmed {
                    "performance" => {
                        available.push(PowerProfile::Performance);
                        if line.contains('*') {
                            active = PowerProfile::Performance;
                        }
                    }
                    "balanced" => {
                        available.push(PowerProfile::Balanced);
                        if line.contains('*') {
                            active = PowerProfile::Balanced;
                        }
                    }
                    "power-saver" => {
                        available.push(PowerProfile::PowerSaver);
                        if line.contains('*') {
                            active = PowerProfile::PowerSaver;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Also try `powerprofilesctl get` for active profile
    if let Ok(output) = Command::new("powerprofilesctl").arg("get").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match text.as_str() {
                "performance" => active = PowerProfile::Performance,
                "balanced" => active = PowerProfile::Balanced,
                "power-saver" => active = PowerProfile::PowerSaver,
                _ => {}
            }
        }
    }

    if available.is_empty() {
        available = vec![
            PowerProfile::Performance,
            PowerProfile::Balanced,
            PowerProfile::PowerSaver,
        ];
    }

    ProfileInfo {
        available,
        active,
        method: ProfileMethod::PowerProfilesDaemon,
    }
}

fn get_profiles_governor() -> ProfileInfo {
    let path = PathBuf::from(CPUFREQ_PATH).join("scaling_available_governors");
    let governors_str = read_sysfs(&path).unwrap_or_default();
    let governors: Vec<&str> = governors_str.split_whitespace().collect();

    let mut available = Vec::new();

    if governors.contains(&"performance") {
        available.push(PowerProfile::Performance);
    }
    if governors.contains(&"schedutil") || governors.contains(&"ondemand") {
        available.push(PowerProfile::Balanced);
    }
    if governors.contains(&"powersave") {
        available.push(PowerProfile::PowerSaver);
    }

    let current_governor = get_governor();
    let active = match current_governor {
        Governor::Performance => PowerProfile::Performance,
        Governor::Powersave => PowerProfile::PowerSaver,
        _ => PowerProfile::Balanced,
    };

    ProfileInfo {
        available,
        active,
        method: ProfileMethod::CpuGovernor,
    }
}

pub fn set_profile(profile: &PowerProfile) -> Result<(), String> {
    let method = detect_profile_method();

    match method {
        ProfileMethod::PowerProfilesDaemon => {
            let name = match profile {
                PowerProfile::Performance => "performance",
                PowerProfile::Balanced => "balanced",
                PowerProfile::PowerSaver => "power-saver",
            };
            let status = Command::new("powerprofilesctl")
                .args(["set", name])
                .status()
                .map_err(|e| format!("Failed to run powerprofilesctl: {e}"))?;

            if status.success() {
                Ok(())
            } else {
                Err("powerprofilesctl set failed".to_string())
            }
        }
        ProfileMethod::CpuGovernor => {
            let governor = match profile {
                PowerProfile::Performance => "performance",
                PowerProfile::Balanced => "schedutil",
                PowerProfile::PowerSaver => "powersave",
            };

            // Use pkexec to write to sysfs (requires root)
            let num_cpus = num_cpus_available();
            for i in 0..num_cpus {
                let path = format!("/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_governor");
                let status = Command::new("pkexec")
                    .args(["tee", &path])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(ref mut stdin) = child.stdin {
                            stdin.write_all(governor.as_bytes())?;
                        }
                        child.wait()
                    })
                    .map_err(|e| format!("Failed to set governor for cpu{i}: {e}"))?;

                if !status.success() {
                    return Err(format!("Failed to set governor for cpu{i}"));
                }
            }
            Ok(())
        }
        ProfileMethod::None => Err("No profile method available".to_string()),
    }
}

fn num_cpus_available() -> usize {
    let path = PathBuf::from("/sys/devices/system/cpu");
    if let Ok(entries) = fs::read_dir(path) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit())
            })
            .count()
    } else {
        1
    }
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

impl PowerProfile {
    pub fn display_name(&self) -> &str {
        match self {
            PowerProfile::Performance => "Berserker",
            PowerProfile::Balanced => "Shieldwall",
            PowerProfile::PowerSaver => "Hibernation",
        }
    }
}

pub fn format_temperature(temp: f64) -> String {
    format!("{temp:.0}°C")
}
