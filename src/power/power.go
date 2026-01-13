package power

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// Governor represents a CPU frequency governor
type Governor string

const (
	GovernorPerformance  Governor = "performance"
	GovernorPowersave    Governor = "powersave"
	GovernorOndemand     Governor = "ondemand"
	GovernorConservative Governor = "conservative"
	GovernorSchedules    Governor = "schedutil"
	GovernorUnknown      Governor = "unknown"
)

// Info contains power and thermal information
type Info struct {
	Governor    Governor // Current CPU governor
	Temperature float64  // CPU temperature in Celsius
	FrequencyMHz int     // Current CPU frequency in MHz
	HasTemp     bool     // Whether temperature reading is available
}

// Paths for sysfs interfaces
const (
	cpufreqPath = "/sys/devices/system/cpu/cpu0/cpufreq"
	hwmonPath   = "/sys/class/hwmon"
)

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

// GetGovernor returns the current CPU frequency governor
func GetGovernor() Governor {
	gov, err := readFile(filepath.Join(cpufreqPath, "scaling_governor"))
	if err != nil {
		return GovernorUnknown
	}
	return Governor(gov)
}

// GetAvailableGovernors returns available CPU governors
func GetAvailableGovernors() []Governor {
	govs, err := readFile(filepath.Join(cpufreqPath, "scaling_available_governors"))
	if err != nil {
		return nil
	}

	var result []Governor
	for _, g := range strings.Fields(govs) {
		result = append(result, Governor(g))
	}
	return result
}

// GetFrequencyMHz returns the current CPU frequency in MHz
func GetFrequencyMHz() int {
	freq, err := readInt(filepath.Join(cpufreqPath, "scaling_cur_freq"))
	if err != nil {
		return 0
	}
	// scaling_cur_freq is in kHz, convert to MHz
	return int(freq / 1000)
}

// findCPUTempHwmon finds the hwmon device for CPU temperature
// Looks for k10temp (AMD) or coretemp (Intel)
func findCPUTempHwmon() string {
	entries, err := os.ReadDir(hwmonPath)
	if err != nil {
		return ""
	}

	for _, entry := range entries {
		namePath := filepath.Join(hwmonPath, entry.Name(), "name")
		name, err := readFile(namePath)
		if err != nil {
			continue
		}

		// Check for CPU temperature sensors
		if name == "k10temp" || name == "coretemp" || name == "cpu_thermal" {
			return filepath.Join(hwmonPath, entry.Name())
		}
	}

	return ""
}

// GetTemperature returns the CPU temperature in Celsius
func GetTemperature() (float64, bool) {
	hwmon := findCPUTempHwmon()
	if hwmon == "" {
		return 0, false
	}

	// Read temp1_input (in millidegrees Celsius)
	temp, err := readInt(filepath.Join(hwmon, "temp1_input"))
	if err != nil {
		return 0, false
	}

	return float64(temp) / 1000.0, true
}

// GetInfo returns complete power and thermal information
func GetInfo() *Info {
	info := &Info{
		Governor:     GetGovernor(),
		FrequencyMHz: GetFrequencyMHz(),
	}

	temp, hasTemp := GetTemperature()
	info.Temperature = temp
	info.HasTemp = hasTemp

	return info
}

// GovernorDisplayName returns a user-friendly name for the governor
func (g Governor) DisplayName() string {
	switch g {
	case GovernorPerformance:
		return "Performance"
	case GovernorPowersave:
		return "Power Saver"
	case GovernorOndemand:
		return "On Demand"
	case GovernorConservative:
		return "Conservative"
	case GovernorSchedules:
		return "Balanced"
	default:
		return string(g)
	}
}

// GovernorIcon returns an icon name for the governor
func (g Governor) IconName() string {
	switch g {
	case GovernorPerformance:
		return "power-profile-performance-symbolic"
	case GovernorPowersave:
		return "power-profile-power-saver-symbolic"
	default:
		return "power-profile-balanced-symbolic"
	}
}

// FormatTemperature formats temperature for display
func FormatTemperature(temp float64) string {
	return fmt.Sprintf("%.0f°C", temp)
}
