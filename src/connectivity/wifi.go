package connectivity

import (
	"fmt"
	"os/exec"
	"strings"
)

// WiFiInfo contains WiFi status information
type WiFiInfo struct {
	Enabled   bool   // Whether WiFi radio is enabled
	Connected bool   // Whether connected to a network
	SSID      string // Current network name (if connected)
	Signal    int    // Signal strength percentage (0-100)
}

// GetWiFiInfo returns current WiFi status
func GetWiFiInfo() *WiFiInfo {
	info := &WiFiInfo{}

	// Check if WiFi is enabled using nmcli
	out, err := exec.Command("nmcli", "radio", "wifi").Output()
	if err != nil {
		return info
	}
	info.Enabled = strings.TrimSpace(string(out)) == "enabled"

	if !info.Enabled {
		return info
	}

	// Get connection info
	out, err = exec.Command("nmcli", "-t", "-f", "ACTIVE,SSID,SIGNAL", "dev", "wifi").Output()
	if err != nil {
		return info
	}

	lines := strings.Split(string(out), "\n")
	for _, line := range lines {
		if strings.HasPrefix(line, "yes:") {
			parts := strings.Split(line, ":")
			if len(parts) >= 3 {
				info.Connected = true
				info.SSID = parts[1]
				// Parse signal strength
				var signal int
				if _, err := fmt.Sscanf(parts[2], "%d", &signal); err == nil {
					info.Signal = signal
				}
			}
			break
		}
	}

	return info
}

// SetWiFiEnabled enables or disables WiFi
func SetWiFiEnabled(enabled bool) error {
	state := "off"
	if enabled {
		state = "on"
	}
	cmd := exec.Command("nmcli", "radio", "wifi", state)
	return cmd.Run()
}

// ToggleWiFi toggles WiFi on/off
func ToggleWiFi() error {
	info := GetWiFiInfo()
	return SetWiFiEnabled(!info.Enabled)
}

// GetWiFiIcon returns the appropriate icon for WiFi state
func GetWiFiIcon(info *WiFiInfo) string {
	if !info.Enabled {
		return "network-wireless-disabled-symbolic"
	}
	if !info.Connected {
		return "network-wireless-offline-symbolic"
	}

	// Signal strength based icons
	if info.Signal >= 80 {
		return "network-wireless-signal-excellent-symbolic"
	} else if info.Signal >= 60 {
		return "network-wireless-signal-good-symbolic"
	} else if info.Signal >= 40 {
		return "network-wireless-signal-ok-symbolic"
	} else if info.Signal >= 20 {
		return "network-wireless-signal-weak-symbolic"
	}
	return "network-wireless-signal-none-symbolic"
}
