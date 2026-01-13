package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/battery"
)

const (
	// BatteryUpdateInterval is how often to refresh battery status (in seconds)
	BatteryUpdateInterval = 30
)

// Battery represents the battery status widget
type Battery struct {
	Box       *gtk.Box
	icon      *gtk.Image
	label     *gtk.Label
	available bool
	sourceID  glib.SourceHandle
}

// NewBattery creates a new battery widget
func NewBattery() *Battery {
	b := &Battery{
		Box:   gtk.NewBox(gtk.OrientationHorizontal, 4),
		icon:  gtk.NewImageFromIconName("battery-symbolic"),
		label: gtk.NewLabel("--"),
	}

	b.Box.AddCSSClass("battery")
	b.icon.AddCSSClass("battery-icon")
	b.label.AddCSSClass("battery-label")

	// ᚠ Fehu - Wealth, energy, abundance - perfect for battery
	rune := gtk.NewLabel("\u16A0")
	rune.AddCSSClass("module-rune")
	rune.SetTooltipText("ᚠ Fehu - Energy")

	b.Box.Append(rune)
	// Icon hidden - rune serves as visual indicator
	b.Box.Append(b.label)

	// Check if batteries are available
	batteries, err := battery.FindBatteries()
	b.available = err == nil && len(batteries) > 0

	if !b.available {
		b.Box.SetVisible(false)
		return b
	}

	// Initial refresh
	b.Refresh()

	// Start periodic updates
	b.startUpdates()

	return b
}

// startUpdates begins periodic battery status updates
func (b *Battery) startUpdates() {
	b.sourceID = glib.TimeoutAdd(BatteryUpdateInterval*1000, func() bool {
		b.Refresh()
		return true // Continue calling
	})
}

// Stop stops the periodic updates
func (b *Battery) Stop() {
	if b.sourceID > 0 {
		glib.SourceRemove(b.sourceID)
		b.sourceID = 0
	}
}

// Refresh updates the battery display
func (b *Battery) Refresh() {
	if !b.available {
		return
	}

	info, err := battery.GetFirstBattery()
	if err != nil {
		b.label.SetText("--")
		b.icon.SetFromIconName("battery-missing-symbolic")
		return
	}

	// Update percentage label
	b.label.SetText(fmt.Sprintf("%d%%", info.Capacity))

	// Update icon based on level and charging status
	iconName := b.getIconName(info)
	b.icon.SetFromIconName(iconName)

	// Update CSS classes based on state
	b.Box.RemoveCSSClass("charging")
	b.Box.RemoveCSSClass("low")
	b.Box.RemoveCSSClass("critical")

	if info.Status == battery.StatusCharging {
		b.Box.AddCSSClass("charging")
	} else if info.Capacity <= 10 {
		b.Box.AddCSSClass("critical")
	} else if info.Capacity <= 20 {
		b.Box.AddCSSClass("low")
	}

	// Update tooltip with detailed info
	tooltip := b.formatTooltip(info)
	b.Box.SetTooltipText(tooltip)
}

// getIconName returns the appropriate battery icon name
func (b *Battery) getIconName(info *battery.Info) string {
	// Round to nearest 10 for icon levels
	level := (info.Capacity / 10) * 10
	if level > 100 {
		level = 100
	}

	charging := info.Status == battery.StatusCharging

	// Special cases
	if info.Status == battery.StatusFull {
		return "battery-full-charged-symbolic"
	}
	if info.Capacity <= 5 {
		if charging {
			return "battery-empty-charging-symbolic"
		}
		return "battery-empty-symbolic"
	}

	// Standard level icons
	if charging {
		return fmt.Sprintf("battery-level-%d-charging-symbolic", level)
	}
	return fmt.Sprintf("battery-level-%d-symbolic", level)
}

// formatTooltip creates detailed tooltip text
func (b *Battery) formatTooltip(info *battery.Info) string {
	var status string
	switch info.Status {
	case battery.StatusCharging:
		status = "Charging"
	case battery.StatusDischarging:
		status = "Discharging"
	case battery.StatusFull:
		status = "Fully charged"
	case battery.StatusNotCharging:
		status = "Not charging"
	default:
		status = string(info.Status)
	}

	tooltip := fmt.Sprintf("%d%% - %s", info.Capacity, status)

	// Add time remaining if available
	if timeStr := battery.FormatTimeRemaining(info.TimeRemaining); timeStr != "" {
		if info.Status == battery.StatusCharging {
			tooltip += fmt.Sprintf("\n%s until full", timeStr)
		} else if info.Status == battery.StatusDischarging {
			tooltip += fmt.Sprintf("\n%s remaining", timeStr)
		}
	}

	return tooltip
}
