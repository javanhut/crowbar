package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/power"
)

const (
	// PowerUpdateInterval is how often to refresh power status (in seconds)
	PowerUpdateInterval = 10
)

// Power represents the power/thermal status widget
type Power struct {
	Box      *gtk.Box
	icon     *gtk.Image
	label    *gtk.Label
	sourceID glib.SourceHandle
}

// NewPower creates a new power widget
func NewPower() *Power {
	p := &Power{
		Box:   gtk.NewBox(gtk.OrientationHorizontal, 4),
		icon:  gtk.NewImageFromIconName("power-profile-balanced-symbolic"),
		label: gtk.NewLabel("--"),
	}

	p.Box.AddCSSClass("power")
	p.icon.AddCSSClass("power-icon")
	p.label.AddCSSClass("power-label")

	// ᚢ Uruz - Wild ox, primal power, physical strength - perfect for CPU power
	rune := gtk.NewLabel("\u16A2")
	rune.AddCSSClass("module-rune")
	rune.SetTooltipText("ᚢ Uruz - Power")

	p.Box.Append(rune)
	// Icon hidden - rune serves as visual indicator
	p.Box.Append(p.label)

	// Initial refresh
	p.Refresh()

	// Start periodic updates
	p.startUpdates()

	return p
}

// startUpdates begins periodic power status updates
func (p *Power) startUpdates() {
	p.sourceID = glib.TimeoutAdd(PowerUpdateInterval*1000, func() bool {
		p.Refresh()
		return true // Continue calling
	})
}

// Stop stops the periodic updates
func (p *Power) Stop() {
	if p.sourceID > 0 {
		glib.SourceRemove(p.sourceID)
		p.sourceID = 0
	}
}

// Refresh updates the power display
func (p *Power) Refresh() {
	info := power.GetInfo()

	// Update icon based on governor
	iconName := info.Governor.IconName()
	p.icon.SetFromIconName(iconName)

	// Build label text
	var labelText string
	if info.HasTemp {
		labelText = power.FormatTemperature(info.Temperature)
	} else {
		labelText = info.Governor.DisplayName()
	}
	p.label.SetText(labelText)

	// Update CSS classes based on temperature
	p.Box.RemoveCSSClass("hot")
	p.Box.RemoveCSSClass("critical")
	if info.HasTemp {
		if info.Temperature >= 85 {
			p.Box.AddCSSClass("critical")
		} else if info.Temperature >= 70 {
			p.Box.AddCSSClass("hot")
		}
	}

	// Update tooltip with detailed info
	tooltip := p.formatTooltip(info)
	p.Box.SetTooltipText(tooltip)
}

// formatTooltip creates detailed tooltip text
func (p *Power) formatTooltip(info *power.Info) string {
	tooltip := fmt.Sprintf("Governor: %s", info.Governor.DisplayName())

	if info.HasTemp {
		tooltip += fmt.Sprintf("\nCPU: %s", power.FormatTemperature(info.Temperature))
	}

	if info.FrequencyMHz > 0 {
		if info.FrequencyMHz >= 1000 {
			tooltip += fmt.Sprintf("\nFrequency: %.1f GHz", float64(info.FrequencyMHz)/1000.0)
		} else {
			tooltip += fmt.Sprintf("\nFrequency: %d MHz", info.FrequencyMHz)
		}
	}

	return tooltip
}
