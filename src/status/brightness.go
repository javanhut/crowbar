package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/brightness"
)

const (
	// BrightnessUpdateInterval is how often to refresh brightness (in seconds)
	BrightnessUpdateInterval = 5
)

// Brightness represents the screen brightness widget
type Brightness struct {
	Box        *gtk.Box
	menuButton *gtk.MenuButton
	label      *gtk.Label
	slider     *gtk.Scale
	popover    *gtk.Popover
	available  bool
	sourceID   glib.SourceHandle
	device     string   // Current backlight device
	updating   bool     // Prevent slider feedback loop
}

// NewBrightness creates a new brightness widget
func NewBrightness() *Brightness {
	b := &Brightness{
		Box:   gtk.NewBox(gtk.OrientationHorizontal, 0),
		label: gtk.NewLabel("--"),
	}

	b.Box.AddCSSClass("brightness")

	// Create menu button for popover
	b.menuButton = gtk.NewMenuButton()
	b.menuButton.AddCSSClass("brightness-button")
	b.menuButton.SetHasFrame(false)

	// ᛊ Sowilo - Sun, light, victory, enlightenment - perfect for brightness
	rune := gtk.NewLabel("\u16CA")
	rune.AddCSSClass("module-rune")

	// Create button content
	btnContent := gtk.NewBox(gtk.OrientationHorizontal, 6)
	btnContent.Append(rune)
	btnContent.Append(b.label)
	b.menuButton.SetChild(btnContent)

	// Create popover with slider
	b.popover = gtk.NewPopover()
	b.popover.AddCSSClass("brightness-popover")
	b.popover.SetAutohide(true)

	popoverContent := gtk.NewBox(gtk.OrientationVertical, 8)
	popoverContent.SetMarginTop(12)
	popoverContent.SetMarginBottom(12)
	popoverContent.SetMarginStart(12)
	popoverContent.SetMarginEnd(12)

	// Header with rune and title
	header := gtk.NewBox(gtk.OrientationHorizontal, 8)
	headerRune := gtk.NewLabel("\u16CA")
	headerRune.AddCSSClass("slider-header-rune")
	headerLabel := gtk.NewLabel("Sunlight")
	headerLabel.AddCSSClass("slider-header-label")
	header.Append(headerRune)
	header.Append(headerLabel)
	popoverContent.Append(header)

	// Brightness slider
	b.slider = gtk.NewScaleWithRange(gtk.OrientationHorizontal, 1, 100, 1)
	b.slider.AddCSSClass("brightness-slider")
	b.slider.SetDrawValue(true)
	b.slider.SetValuePos(gtk.PosRight)
	b.slider.SetHExpand(true)
	b.slider.SetSizeRequest(200, -1)

	// Connect slider value change
	b.slider.ConnectValueChanged(func() {
		if b.updating {
			return
		}
		val := int(b.slider.Value())
		if b.device != "" {
			brightness.SetBrightness(b.device, val)
		}
	})

	popoverContent.Append(b.slider)

	b.popover.SetChild(popoverContent)
	b.menuButton.SetPopover(b.popover)

	b.Box.Append(b.menuButton)

	// Check if backlight is available
	devices, err := brightness.FindBacklights()
	b.available = err == nil && len(devices) > 0

	if !b.available {
		b.Box.SetVisible(false)
		return b
	}

	// Store first device
	if len(devices) > 0 {
		b.device = devices[0]
	}

	// Initial refresh
	b.Refresh()

	// Start periodic updates
	b.startUpdates()

	return b
}

// startUpdates begins periodic brightness updates
func (b *Brightness) startUpdates() {
	b.sourceID = glib.TimeoutAdd(BrightnessUpdateInterval*1000, func() bool {
		b.Refresh()
		return true // Continue calling
	})
}

// Stop stops the periodic updates
func (b *Brightness) Stop() {
	if b.sourceID > 0 {
		glib.SourceRemove(b.sourceID)
		b.sourceID = 0
	}
}

// Refresh updates the brightness display
func (b *Brightness) Refresh() {
	if !b.available {
		return
	}

	info, err := brightness.GetFirstBacklight()
	if err != nil || !info.Available {
		b.label.SetText("--")
		return
	}

	// Update label
	b.label.SetText(fmt.Sprintf("%d%%", info.Percent))

	// Update slider without triggering callback
	b.updating = true
	b.slider.SetValue(float64(info.Percent))
	b.updating = false

	// Update tooltip
	tooltip := fmt.Sprintf("Brightness: %d%%\nDevice: %s\nClick to adjust", info.Percent, info.Device)
	b.Box.SetTooltipText(tooltip)
}
