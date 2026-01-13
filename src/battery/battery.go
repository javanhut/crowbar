package battery

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

// Status represents the battery charging status
type Status string

const (
	StatusCharging    Status = "Charging"
	StatusDischarging Status = "Discharging"
	StatusFull        Status = "Full"
	StatusNotCharging Status = "Not charging"
	StatusUnknown     Status = "Unknown"
)

// Info contains battery information
type Info struct {
	Name          string        // Battery name (e.g., "BAT0")
	Present       bool          // Whether battery is present
	Capacity      int           // Percentage (0-100)
	Status        Status        // Charging status
	EnergyNow     int64         // Current energy in μWh
	EnergyFull    int64         // Full capacity in μWh
	PowerNow      int64         // Current power in μW
	TimeRemaining time.Duration // Estimated time remaining
	ACOnline      bool          // Whether AC adapter is connected
}

// basePath is the sysfs power supply path
const basePath = "/sys/class/power_supply"

// readFile reads a sysfs file and returns its trimmed content
func readFile(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(data)), nil
}

// readInt reads an integer from a sysfs file
func readInt(path string) (int64, error) {
	s, err := readFile(path)
	if err != nil {
		return 0, err
	}
	return strconv.ParseInt(s, 10, 64)
}

// FindBatteries discovers available battery devices
func FindBatteries() ([]string, error) {
	entries, err := os.ReadDir(basePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read power_supply: %w", err)
	}

	var batteries []string
	for _, entry := range entries {
		typePath := filepath.Join(basePath, entry.Name(), "type")
		typeStr, err := readFile(typePath)
		if err != nil {
			continue
		}
		if typeStr == "Battery" {
			batteries = append(batteries, entry.Name())
		}
	}

	return batteries, nil
}

// isACOnline checks if any AC adapter is connected
func isACOnline() bool {
	entries, err := os.ReadDir(basePath)
	if err != nil {
		return false
	}

	for _, entry := range entries {
		typePath := filepath.Join(basePath, entry.Name(), "type")
		typeStr, _ := readFile(typePath)
		if typeStr == "Mains" {
			onlinePath := filepath.Join(basePath, entry.Name(), "online")
			online, _ := readFile(onlinePath)
			if online == "1" {
				return true
			}
		}
	}
	return false
}

// GetInfo reads information for a specific battery
func GetInfo(name string) (*Info, error) {
	batteryPath := filepath.Join(basePath, name)

	// Check if battery exists
	if _, err := os.Stat(batteryPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("battery %s not found", name)
	}

	info := &Info{
		Name:     name,
		ACOnline: isACOnline(),
	}

	// Read present status
	if present, err := readFile(filepath.Join(batteryPath, "present")); err == nil {
		info.Present = present == "1"
	}

	if !info.Present {
		return info, nil
	}

	// Read capacity (percentage)
	if cap, err := readInt(filepath.Join(batteryPath, "capacity")); err == nil {
		info.Capacity = int(cap)
	}

	// Read status
	if status, err := readFile(filepath.Join(batteryPath, "status")); err == nil {
		info.Status = Status(status)
	} else {
		info.Status = StatusUnknown
	}

	// Read energy values for time remaining calculation
	info.EnergyNow, _ = readInt(filepath.Join(batteryPath, "energy_now"))
	info.EnergyFull, _ = readInt(filepath.Join(batteryPath, "energy_full"))
	info.PowerNow, _ = readInt(filepath.Join(batteryPath, "power_now"))

	// Calculate time remaining
	info.TimeRemaining = calculateTimeRemaining(info)

	return info, nil
}

// calculateTimeRemaining estimates time to empty/full
func calculateTimeRemaining(info *Info) time.Duration {
	if info.PowerNow <= 0 {
		return 0
	}

	var hours float64
	switch info.Status {
	case StatusDischarging:
		// Time to empty
		hours = float64(info.EnergyNow) / float64(info.PowerNow)
	case StatusCharging:
		// Time to full
		remaining := info.EnergyFull - info.EnergyNow
		if remaining > 0 {
			hours = float64(remaining) / float64(info.PowerNow)
		}
	default:
		return 0
	}

	return time.Duration(hours * float64(time.Hour))
}

// GetFirstBattery returns info for the first available battery
func GetFirstBattery() (*Info, error) {
	batteries, err := FindBatteries()
	if err != nil {
		return nil, err
	}
	if len(batteries) == 0 {
		return nil, fmt.Errorf("no batteries found")
	}
	return GetInfo(batteries[0])
}

// FormatTimeRemaining formats duration as "Xh Ym" or "Ym"
func FormatTimeRemaining(d time.Duration) string {
	if d <= 0 {
		return ""
	}

	hours := int(d.Hours())
	minutes := int(d.Minutes()) % 60

	if hours > 0 {
		return fmt.Sprintf("%dh %dm", hours, minutes)
	}
	return fmt.Sprintf("%dm", minutes)
}
