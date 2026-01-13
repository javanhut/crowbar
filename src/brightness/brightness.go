package brightness

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

// Info contains brightness information
type Info struct {
	Device        string // Device name (e.g., "amdgpu_bl1")
	Brightness    int    // Current brightness (raw value)
	MaxBrightness int    // Maximum brightness (raw value)
	Percent       int    // Brightness percentage (0-100)
	Available     bool   // Whether backlight is available
}

// basePath is the sysfs backlight path
const basePath = "/sys/class/backlight"

// readFile reads a sysfs file and returns its trimmed content
func readFile(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(data)), nil
}

// readInt reads an integer from a sysfs file
func readInt(path string) (int, error) {
	s, err := readFile(path)
	if err != nil {
		return 0, err
	}
	val, err := strconv.ParseInt(s, 10, 64)
	return int(val), err
}

// FindBacklights discovers available backlight devices
func FindBacklights() ([]string, error) {
	entries, err := os.ReadDir(basePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read backlight directory: %w", err)
	}

	var devices []string
	for _, entry := range entries {
		devices = append(devices, entry.Name())
	}

	return devices, nil
}

// GetInfo reads information for a specific backlight device
func GetInfo(device string) (*Info, error) {
	devicePath := filepath.Join(basePath, device)

	// Check if device exists
	if _, err := os.Stat(devicePath); os.IsNotExist(err) {
		return nil, fmt.Errorf("backlight device %s not found", device)
	}

	info := &Info{
		Device:    device,
		Available: true,
	}

	// Read max brightness
	max, err := readInt(filepath.Join(devicePath, "max_brightness"))
	if err != nil {
		info.Available = false
		return info, nil
	}
	info.MaxBrightness = max

	// Read current brightness
	current, err := readInt(filepath.Join(devicePath, "brightness"))
	if err != nil {
		info.Available = false
		return info, nil
	}
	info.Brightness = current

	// Calculate percentage
	if info.MaxBrightness > 0 {
		info.Percent = (info.Brightness * 100) / info.MaxBrightness
	}

	return info, nil
}

// GetFirstBacklight returns info for the first available backlight
func GetFirstBacklight() (*Info, error) {
	devices, err := FindBacklights()
	if err != nil {
		return nil, err
	}
	if len(devices) == 0 {
		return nil, fmt.Errorf("no backlight devices found")
	}
	return GetInfo(devices[0])
}

// SetBrightness sets brightness to a percentage (0-100)
// Uses brightnessctl if available (handles permissions), falls back to sysfs
func SetBrightness(device string, percent int) error {
	if percent < 1 {
		percent = 1 // Minimum 1% to avoid black screen
	}
	if percent > 100 {
		percent = 100
	}

	// Try brightnessctl first (handles permissions via udev)
	cmd := exec.Command("brightnessctl", "set", fmt.Sprintf("%d%%", percent))
	if err := cmd.Run(); err == nil {
		return nil
	}

	// Fallback to direct sysfs write (requires permissions)
	devicePath := filepath.Join(basePath, device)

	// Read max brightness to calculate raw value
	max, err := readInt(filepath.Join(devicePath, "max_brightness"))
	if err != nil {
		return err
	}

	// Calculate raw value
	raw := (percent * max) / 100
	if raw < 1 {
		raw = 1
	}

	// Write new brightness
	brightnessPath := filepath.Join(devicePath, "brightness")
	return os.WriteFile(brightnessPath, []byte(strconv.Itoa(raw)), 0644)
}

// GetBrightnessIcon returns the appropriate icon name for the brightness level
func GetBrightnessIcon(percent int) string {
	if percent <= 0 {
		return "display-brightness-off-symbolic"
	}
	if percent < 33 {
		return "display-brightness-low-symbolic"
	}
	if percent < 66 {
		return "display-brightness-medium-symbolic"
	}
	return "display-brightness-high-symbolic"
}
