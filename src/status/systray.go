package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/systray"
)

// Systray represents the system tray widget
type Systray struct {
	Box       *gtk.Box
	host      *systray.Host
	icons     map[string]*gtk.Image
	available bool
}

// NewSystray creates a new system tray widget
func NewSystray() *Systray {
	s := &Systray{
		Box:   gtk.NewBox(gtk.OrientationHorizontal, 4),
		icons: make(map[string]*gtk.Image),
	}

	s.Box.AddCSSClass("systray")

	// ᛉ Algiz - Elk, protection, defense - perfect for system tray (background apps)
	rune := gtk.NewLabel("\u16C9")
	rune.AddCSSClass("module-rune")
	rune.SetTooltipText("ᛉ Algiz - Protection")
	s.Box.Append(rune)

	// Create host with callback for updates
	var err error
	s.host, err = systray.NewHost(func() {
		// Use IdleAdd to safely update GTK from callback
		glib.IdleAdd(func() {
			s.Refresh()
		})
	})

	if err != nil {
		s.available = false
		s.Box.SetVisible(false)
		return s
	}

	// Start the host
	if err := s.host.Start(); err != nil {
		s.available = false
		s.Box.SetVisible(false)
		return s
	}

	s.available = true

	// Initial refresh
	s.Refresh()

	return s
}

// Refresh updates the tray icons
func (s *Systray) Refresh() {
	if !s.available {
		return
	}

	items := s.host.GetItems()

	// Track which items are still present
	present := make(map[string]bool)

	for _, item := range items {
		key := item.Service + item.Path
		present[key] = true

		// Create icon if not exists
		if _, exists := s.icons[key]; !exists {
			icon := s.createIcon(item)
			s.icons[key] = icon
			s.Box.Append(icon)
		} else {
			// Update existing icon
			s.updateIcon(s.icons[key], item)
		}
	}

	// Remove icons for items that are gone
	for key, icon := range s.icons {
		if !present[key] {
			s.Box.Remove(icon)
			delete(s.icons, key)
		}
	}

	// Hide if no items
	s.Box.SetVisible(len(s.icons) > 0)
}

// createIcon creates an icon widget for a tray item
func (s *Systray) createIcon(item *systray.Item) *gtk.Image {
	var icon *gtk.Image

	if item.IconName != "" {
		icon = gtk.NewImageFromIconName(item.IconName)
	} else {
		// Fallback icon
		icon = gtk.NewImageFromIconName("application-x-executable-symbolic")
	}

	icon.AddCSSClass("systray-icon")
	icon.SetPixelSize(16)

	// Set tooltip
	tooltip := item.Title
	if tooltip == "" {
		tooltip = item.Service
	}
	icon.SetTooltipText(tooltip)

	return icon
}

// updateIcon updates an existing icon
func (s *Systray) updateIcon(icon *gtk.Image, item *systray.Item) {
	if item.IconName != "" {
		icon.SetFromIconName(item.IconName)
	}

	tooltip := item.Title
	if tooltip == "" {
		tooltip = item.Service
	}
	icon.SetTooltipText(tooltip)
}

// Stop stops the system tray host
func (s *Systray) Stop() {
	if s.host != nil {
		s.host.Stop()
	}
}

// formatTooltip creates tooltip for the tray area
func (s *Systray) formatTooltip() string {
	count := s.host.ItemCount()
	if count == 0 {
		return "No tray items"
	}
	return fmt.Sprintf("%d tray item(s)", count)
}
