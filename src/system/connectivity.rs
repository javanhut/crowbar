use std::process::Command;

// === WiFi Types ===

pub struct WiFiInfo {
    pub enabled: bool,
    pub connected: bool,
    pub ssid: String,
    pub signal: i32,
}

pub struct WiFiNetwork {
    pub ssid: String,
    pub signal: i32,
    pub security: String,
    pub connected: bool,
    pub saved: bool,
}

// === Bluetooth Types ===

pub struct BluetoothInfo {
    pub available: bool,
    pub powered: bool,
    pub connected: bool,
    pub device: String,
}

#[allow(dead_code)]
pub struct BluetoothDevice {
    pub mac: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
    pub battery: Option<i32>,
}

// === WiFi Functions ===

pub fn get_wifi_info() -> WiFiInfo {
    let mut info = WiFiInfo {
        enabled: false,
        connected: false,
        ssid: String::new(),
        signal: 0,
    };

    let Ok(output) = Command::new("nmcli").args(["radio", "wifi"]).output() else {
        return info;
    };

    info.enabled = String::from_utf8_lossy(&output.stdout).trim() == "enabled";
    if !info.enabled {
        return info;
    }

    let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID,SIGNAL", "dev", "wifi"])
        .output()
    else {
        return info;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with("yes:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                info.connected = true;
                info.ssid = parts[1].to_string();
                info.signal = parts[2].parse().unwrap_or(0);
            }
            break;
        }
    }

    info
}

pub fn scan_wifi_networks() -> Vec<WiFiNetwork> {
    let mut networks = Vec::new();

    // Get saved connections
    let saved_ssids = get_saved_wifi_connections();

    let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "SSID,SIGNAL,SECURITY,ACTIVE", "dev", "wifi", "list", "--rescan", "yes"])
        .output()
    else {
        return networks;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut seen = std::collections::HashSet::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // nmcli -t uses : as separator, but SSID can contain colons
        // Format: SSID:SIGNAL:SECURITY:ACTIVE
        // Parse from the end since SSID may contain colons
        let parts: Vec<&str> = line.rsplitn(4, ':').collect();
        if parts.len() < 4 {
            continue;
        }

        let active = parts[0];
        let security = parts[1];
        let signal_str = parts[2];
        let ssid = parts[3];

        if ssid.is_empty() {
            continue;
        }

        // Deduplicate by SSID (multiple APs for same network)
        if !seen.insert(ssid.to_string()) {
            continue;
        }

        networks.push(WiFiNetwork {
            ssid: ssid.to_string(),
            signal: signal_str.parse().unwrap_or(0),
            security: security.to_string(),
            connected: active == "yes",
            saved: saved_ssids.contains(&ssid.to_string()),
        });
    }

    // Sort: connected first, then by signal strength descending
    networks.sort_by(|a, b| {
        b.connected.cmp(&a.connected)
            .then(b.signal.cmp(&a.signal))
    });

    networks
}

fn get_saved_wifi_connections() -> Vec<String> {
    let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show"])
        .output()
    else {
        return Vec::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.rsplitn(2, ':').collect();
            if parts.len() == 2 && parts[0].contains("wireless") {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn connect_wifi(ssid: &str, password: Option<&str>) -> Result<(), String> {
    let mut args = vec!["dev", "wifi", "connect", ssid];
    let pw;
    if let Some(p) = password {
        pw = p.to_string();
        args.push("password");
        args.push(&pw);
    }

    let output = Command::new("nmcli")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Connection failed: {}", stderr.trim()))
    }
}

pub fn connect_hidden_wifi(ssid: &str, password: &str) -> Result<(), String> {
    let output = Command::new("nmcli")
        .args(["dev", "wifi", "connect", ssid, "password", password, "hidden", "yes"])
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Connection failed: {}", stderr.trim()))
    }
}

pub fn disconnect_wifi() -> Result<(), String> {
    // Find the wifi device name
    let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE", "device"])
        .output()
    else {
        return Err("Failed to list devices".into());
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let wifi_dev = text.lines()
        .find(|l| l.ends_with(":wifi"))
        .and_then(|l| l.split(':').next())
        .unwrap_or("wlan0");

    let output = Command::new("nmcli")
        .args(["dev", "disconnect", wifi_dev])
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Disconnect failed: {}", stderr.trim()))
    }
}

pub fn forget_wifi(ssid: &str) -> Result<(), String> {
    let output = Command::new("nmcli")
        .args(["connection", "delete", ssid])
        .output()
        .map_err(|e| format!("Failed to run nmcli: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Forget failed: {}", stderr.trim()))
    }
}

pub fn set_wifi_enabled(enabled: bool) {
    let state = if enabled { "on" } else { "off" };
    let _ = Command::new("nmcli").args(["radio", "wifi", state]).status();
}

pub fn get_wifi_icon(info: &WiFiInfo) -> &'static str {
    if !info.enabled {
        return "network-wireless-disabled-symbolic";
    }
    if !info.connected {
        return "network-wireless-offline-symbolic";
    }
    if info.signal >= 80 {
        "network-wireless-signal-excellent-symbolic"
    } else if info.signal >= 60 {
        "network-wireless-signal-good-symbolic"
    } else if info.signal >= 40 {
        "network-wireless-signal-ok-symbolic"
    } else if info.signal >= 20 {
        "network-wireless-signal-weak-symbolic"
    } else {
        "network-wireless-signal-none-symbolic"
    }
}

pub fn get_wifi_signal_icon(signal: i32) -> &'static str {
    if signal >= 80 {
        "network-wireless-signal-excellent-symbolic"
    } else if signal >= 60 {
        "network-wireless-signal-good-symbolic"
    } else if signal >= 40 {
        "network-wireless-signal-ok-symbolic"
    } else if signal >= 20 {
        "network-wireless-signal-weak-symbolic"
    } else {
        "network-wireless-signal-none-symbolic"
    }
}

// === Bluetooth Functions ===

pub fn get_bluetooth_info() -> BluetoothInfo {
    let mut info = BluetoothInfo {
        available: false,
        powered: false,
        connected: false,
        device: String::new(),
    };

    if Command::new("bluetoothctl").arg("--version").output().is_err() {
        return info;
    }
    info.available = true;

    let Ok(output) = Command::new("bluetoothctl").arg("show").output() else {
        return info;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Powered:") {
            info.powered = line.contains("yes");
        }
    }

    if !info.powered {
        return info;
    }

    if let Ok(output) = Command::new("bluetoothctl")
        .args(["devices", "Connected"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("Device") {
                info.connected = true;
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() >= 3 {
                    info.device = parts[2].to_string();
                }
                break;
            }
        }
    }

    info
}

pub fn get_paired_devices() -> Vec<BluetoothDevice> {
    let mut devices = Vec::new();

    let Ok(output) = Command::new("bluetoothctl")
        .args(["devices", "Paired"])
        .output()
    else {
        return devices;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if !line.starts_with("Device") {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        let mac = parts[1].to_string();
        let name = parts[2].to_string();

        // Get detailed info
        let (connected, battery) = get_device_info(&mac);

        devices.push(BluetoothDevice {
            mac,
            name,
            paired: true,
            connected,
            battery,
        });
    }

    // Sort: connected first, then by name
    devices.sort_by(|a, b| {
        b.connected.cmp(&a.connected)
            .then(a.name.cmp(&b.name))
    });

    devices
}

pub fn scan_bluetooth_devices() -> Vec<BluetoothDevice> {
    // Start scan for a few seconds
    let _ = Command::new("bluetoothctl")
        .args(["--timeout", "3", "scan", "on"])
        .output();

    let mut devices = Vec::new();

    let Ok(output) = Command::new("bluetoothctl")
        .arg("devices")
        .output()
    else {
        return devices;
    };

    // Get paired devices to filter them out
    let paired = get_paired_devices();
    let paired_macs: std::collections::HashSet<String> = paired.iter().map(|d| d.mac.clone()).collect();

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if !line.starts_with("Device") {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        let mac = parts[1].to_string();
        let name = parts[2].to_string();

        // Skip already paired
        if paired_macs.contains(&mac) {
            continue;
        }

        // Skip devices with no name (just MAC address)
        if name == mac {
            continue;
        }

        devices.push(BluetoothDevice {
            mac,
            name,
            paired: false,
            connected: false,
            battery: None,
        });
    }

    devices
}

fn get_device_info(mac: &str) -> (bool, Option<i32>) {
    let Ok(output) = Command::new("bluetoothctl")
        .args(["info", mac])
        .output()
    else {
        return (false, None);
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut connected = false;
    let mut battery = None;

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Connected:") {
            connected = line.contains("yes");
        }
        if line.starts_with("Battery Percentage:") {
            // Format: "Battery Percentage: 0x55 (85)"
            if let Some(paren_start) = line.rfind('(') {
                if let Some(paren_end) = line.rfind(')') {
                    if let Ok(val) = line[paren_start + 1..paren_end].parse::<i32>() {
                        battery = Some(val);
                    }
                }
            }
        }
    }

    (connected, battery)
}

pub fn connect_bluetooth(mac: &str) -> Result<(), String> {
    let output = Command::new("bluetoothctl")
        .args(["connect", mac])
        .output()
        .map_err(|e| format!("Failed to run bluetoothctl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Connection successful") {
        Ok(())
    } else {
        Err(format!("Connection failed: {}", stdout.trim()))
    }
}

pub fn disconnect_bluetooth(mac: &str) -> Result<(), String> {
    let output = Command::new("bluetoothctl")
        .args(["disconnect", mac])
        .output()
        .map_err(|e| format!("Failed to run bluetoothctl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Successful") {
        Ok(())
    } else {
        Err(format!("Disconnect failed: {}", stdout.trim()))
    }
}

pub fn pair_bluetooth(mac: &str) -> Result<(), String> {
    let output = Command::new("bluetoothctl")
        .args(["pair", mac])
        .output()
        .map_err(|e| format!("Failed to run bluetoothctl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Pairing successful") || stdout.contains("already exists") {
        // Also trust the device
        let _ = Command::new("bluetoothctl")
            .args(["trust", mac])
            .output();
        Ok(())
    } else {
        Err(format!("Pairing failed: {}", stdout.trim()))
    }
}

pub fn remove_bluetooth(mac: &str) -> Result<(), String> {
    let output = Command::new("bluetoothctl")
        .args(["remove", mac])
        .output()
        .map_err(|e| format!("Failed to run bluetoothctl: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Remove failed: {}", stderr.trim()))
    }
}

pub fn set_bluetooth_enabled(enabled: bool) {
    let state = if enabled { "on" } else { "off" };
    let _ = Command::new("bluetoothctl").args(["power", state]).status();
}

pub fn get_bluetooth_icon(info: &BluetoothInfo) -> &'static str {
    if !info.available || !info.powered {
        "bluetooth-disabled-symbolic"
    } else if info.connected {
        "bluetooth-active-symbolic"
    } else {
        "bluetooth-symbolic"
    }
}
