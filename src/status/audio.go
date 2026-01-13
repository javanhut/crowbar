package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/audio"
)

// Audio represents the audio/volume status widget
type Audio struct {
	Box           *gtk.Box
	menuButton    *gtk.MenuButton
	label         *gtk.Label
	slider        *gtk.Scale
	muteBtn       *gtk.Button
	popover       *gtk.Popover
	eventListener *audio.EventListener
	available     bool
	updating      bool // prevent slider feedback loop
}

// NewAudio creates a new audio widget
func NewAudio() *Audio {
	a := &Audio{
		Box:   gtk.NewBox(gtk.OrientationHorizontal, 0),
		label: gtk.NewLabel("--"),
	}

	a.Box.AddCSSClass("audio")

	// Create menu button for popover
	a.menuButton = gtk.NewMenuButton()
	a.menuButton.AddCSSClass("audio-button")
	a.menuButton.SetHasFrame(false)

	// ᚨ Ansuz - Voice of Odin, spoken word, sound
	rune := gtk.NewLabel("\u16A8")
	rune.AddCSSClass("module-rune")

	// Create button content
	btnContent := gtk.NewBox(gtk.OrientationHorizontal, 6)
	btnContent.Append(rune)
	btnContent.Append(a.label)
	a.menuButton.SetChild(btnContent)

	// Create popover with slider
	a.popover = gtk.NewPopover()
	a.popover.AddCSSClass("audio-popover")

	popoverContent := gtk.NewBox(gtk.OrientationVertical, 8)
	popoverContent.SetMarginTop(12)
	popoverContent.SetMarginBottom(12)
	popoverContent.SetMarginStart(12)
	popoverContent.SetMarginEnd(12)

	// Header with rune and title
	header := gtk.NewBox(gtk.OrientationHorizontal, 8)
	headerRune := gtk.NewLabel("\u16A8")
	headerRune.AddCSSClass("slider-header-rune")
	headerLabel := gtk.NewLabel("Gjallarhorn")
	headerLabel.AddCSSClass("slider-header-label")
	header.Append(headerRune)
	header.Append(headerLabel)
	popoverContent.Append(header)

	// Volume slider
	a.slider = gtk.NewScaleWithRange(gtk.OrientationHorizontal, 0, 100, 1)
	a.slider.AddCSSClass("audio-slider")
	a.slider.SetDrawValue(true)
	a.slider.SetValuePos(gtk.PosRight)
	a.slider.SetHExpand(true)
	a.slider.SetSizeRequest(200, -1)

	// Connect slider value change
	a.slider.ConnectValueChanged(func() {
		if a.updating {
			return
		}
		vol := int(a.slider.Value())
		audio.SetVolume(vol)
	})

	popoverContent.Append(a.slider)

	// Mute button
	a.muteBtn = gtk.NewButton()
	a.muteBtn.AddCSSClass("audio-mute-btn")
	muteBox := gtk.NewBox(gtk.OrientationHorizontal, 8)
	muteRune := gtk.NewLabel("\u16C1") // ᛁ Isa - silence/stillness
	muteRune.AddCSSClass("mute-rune")
	muteLabel := gtk.NewLabel("Mute")
	muteLabel.AddCSSClass("mute-label")
	muteBox.Append(muteRune)
	muteBox.Append(muteLabel)
	muteBox.SetHAlign(gtk.AlignCenter)
	a.muteBtn.SetChild(muteBox)

	a.muteBtn.ConnectClicked(func() {
		audio.ToggleMute()
		a.Refresh()
	})

	popoverContent.Append(a.muteBtn)

	a.popover.SetChild(popoverContent)
	a.menuButton.SetPopover(a.popover)

	a.Box.Append(a.menuButton)

	// Check if audio is available
	info := audio.GetInfo()
	a.available = info.Available

	if !a.available {
		a.Box.SetVisible(false)
		return a
	}

	// Initial refresh
	a.Refresh()

	return a
}

// SetupEvents configures the audio event listener for real-time updates
func (a *Audio) SetupEvents() {
	if !a.available {
		return
	}

	a.eventListener = audio.NewEventListener(func() {
		glib.IdleAdd(func() {
			a.Refresh()
		})
	})

	a.eventListener.Start()
}

// Stop stops the event listener
func (a *Audio) Stop() {
	if a.eventListener != nil {
		a.eventListener.Stop()
	}
}

// Refresh updates the audio display
func (a *Audio) Refresh() {
	if !a.available {
		return
	}

	info := audio.GetInfo()
	if !info.Available {
		a.label.SetText("--")
		return
	}

	// Update label and CSS classes
	a.Box.RemoveCSSClass("muted")
	if info.Muted {
		a.label.SetText("Muted")
		a.Box.AddCSSClass("muted")
	} else {
		a.label.SetText(fmt.Sprintf("%d%%", info.Volume))
	}

	// Update slider without triggering callback
	a.updating = true
	a.slider.SetValue(float64(info.Volume))
	a.updating = false

	// Update mute button text
	if info.Muted {
		a.muteBtn.AddCSSClass("muted")
	} else {
		a.muteBtn.RemoveCSSClass("muted")
	}

	// Update tooltip
	a.Box.SetTooltipText(fmt.Sprintf("Volume: %d%%\nClick to adjust", info.Volume))
}
