package connectivity

import (
	"os/exec"
	"strings"
)

// BluetoothInfo contains Bluetooth status information
type BluetoothInfo struct {
	Available bool   // Whether Bluetooth hardware is available
	Powered   bool   // Whether Bluetooth is powered on
	Connected bool   // Whether any device is connected
	Device    string // Connected device name (if any)
}

// GetBluetoothInfo returns current Bluetooth status
func GetBluetoothInfo() *BluetoothInfo {
	info := &BluetoothInfo{}

	// Check if bluetoothctl is available
	_, err := exec.LookPath("bluetoothctl")
	if err != nil {
		return info
	}
	info.Available = true

	// Check power state
	out, err := exec.Command("bluetoothctl", "show").Output()
	if err != nil {
		return info
	}

	lines := strings.Split(string(out), "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if strings.HasPrefix(line, "Powered:") {
			info.Powered = strings.Contains(line, "yes")
		}
	}

	if !info.Powered {
		return info
	}

	// Check for connected devices
	out, err = exec.Command("bluetoothctl", "devices", "Connected").Output()
	if err != nil {
		// Fallback: try older bluetoothctl format
		out, err = exec.Command("bluetoothctl", "info").Output()
		if err != nil {
			return info
		}
		if strings.Contains(string(out), "Connected: yes") {
			info.Connected = true
			// Try to get device name
			for _, line := range strings.Split(string(out), "\n") {
				if strings.Contains(line, "Name:") {
					parts := strings.SplitN(line, ":", 2)
					if len(parts) == 2 {
						info.Device = strings.TrimSpace(parts[1])
					}
					break
				}
			}
		}
		return info
	}

	// Parse connected devices
	lines = strings.Split(strings.TrimSpace(string(out)), "\n")
	for _, line := range lines {
		if strings.HasPrefix(line, "Device") {
			info.Connected = true
			// Format: "Device XX:XX:XX:XX:XX:XX DeviceName"
			parts := strings.SplitN(line, " ", 3)
			if len(parts) >= 3 {
				info.Device = parts[2]
			}
			break
		}
	}

	return info
}

// SetBluetoothEnabled enables or disables Bluetooth
func SetBluetoothEnabled(enabled bool) error {
	state := "off"
	if enabled {
		state = "on"
	}
	cmd := exec.Command("bluetoothctl", "power", state)
	return cmd.Run()
}

// ToggleBluetooth toggles Bluetooth on/off
func ToggleBluetooth() error {
	info := GetBluetoothInfo()
	return SetBluetoothEnabled(!info.Powered)
}

// GetBluetoothIcon returns the appropriate icon for Bluetooth state
func GetBluetoothIcon(info *BluetoothInfo) string {
	if !info.Available || !info.Powered {
		return "bluetooth-disabled-symbolic"
	}
	if info.Connected {
		return "bluetooth-active-symbolic"
	}
	return "bluetooth-symbolic"
}
