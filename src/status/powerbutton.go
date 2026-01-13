package status

import (
	"os/exec"

	"github.com/diamondburned/gotk4/pkg/gtk/v4"
)

// PowerButton represents the power/session control widget
type PowerButton struct {
	Button   *gtk.MenuButton
	popover  *gtk.Popover
}

// NewPowerButton creates a new power button widget
func NewPowerButton() *PowerButton {
	p := &PowerButton{}

	// Create menu button with power icon
	p.Button = gtk.NewMenuButton()
	p.Button.AddCSSClass("power-button")
	p.Button.SetTooltipText("Power Menu - ᚦ Thurisaz")

	// Use Thurisaz rune (ᚦ) - Giant/thorn, protective power, strength
	// This rune represents the power to face challenges and make transformative changes
	runeLabel := gtk.NewLabel("\u16A6") // ᚦ Thurisaz
	runeLabel.AddCSSClass("power-rune")
	p.Button.SetChild(runeLabel)

	// Create popover menu
	p.popover = gtk.NewPopover()
	p.popover.AddCSSClass("power-menu")
	p.popover.SetAutohide(true)

	// Create menu content
	menuBox := gtk.NewBox(gtk.OrientationVertical, 4)
	menuBox.AddCSSClass("power-menu-content")
	menuBox.SetMarginTop(8)
	menuBox.SetMarginBottom(8)
	menuBox.SetMarginStart(8)
	menuBox.SetMarginEnd(8)

	// Lock button - ᛁ Isa (Ice, stillness, pause)
	lockBtn := p.createMenuItemWithRune("\u16C1", "Lock", func() {
		p.popover.Popdown()
		exec.Command("loginctl", "lock-session").Start()
	})
	menuBox.Append(lockBtn)

	// Logout button - ᚱ Raidho (Journey, departure)
	logoutBtn := p.createMenuItemWithRune("\u16B1", "Logout", func() {
		p.popover.Popdown()
		exec.Command("hyprctl", "dispatch", "exit").Start()
	})
	menuBox.Append(logoutBtn)

	// Separator
	sep := gtk.NewSeparator(gtk.OrientationHorizontal)
	sep.AddCSSClass("power-menu-separator")
	menuBox.Append(sep)

	// Suspend button - ᚾ Nauthiz (Need, constraint, rest)
	suspendBtn := p.createMenuItemWithRune("\u16BE", "Suspend", func() {
		p.popover.Popdown()
		exec.Command("systemctl", "suspend").Start()
	})
	menuBox.Append(suspendBtn)

	// Reboot button - ᛟ Othala (Home, return, cycle)
	rebootBtn := p.createMenuItemWithRune("\u16DF", "Reboot", func() {
		p.popover.Popdown()
		exec.Command("systemctl", "reboot").Start()
	})
	menuBox.Append(rebootBtn)

	// Shutdown button - ᚺ Hagalaz (Hail, transformation, end)
	shutdownBtn := p.createMenuItemWithRune("\u16BA", "Shutdown", func() {
		p.popover.Popdown()
		exec.Command("systemctl", "poweroff").Start()
	})
	shutdownBtn.AddCSSClass("power-menu-shutdown")
	menuBox.Append(shutdownBtn)

	p.popover.SetChild(menuBox)
	p.Button.SetPopover(p.popover)

	return p
}

// createMenuItemWithRune creates a styled menu item button with a rune symbol
func (p *PowerButton) createMenuItemWithRune(rune, label string, onClick func()) *gtk.Button {
	btn := gtk.NewButton()
	btn.AddCSSClass("power-menu-item")

	box := gtk.NewBox(gtk.OrientationHorizontal, 12)

	// Use rune as icon
	runeLabel := gtk.NewLabel(rune)
	runeLabel.AddCSSClass("power-menu-rune")

	lbl := gtk.NewLabel(label)
	lbl.AddCSSClass("power-menu-label")
	lbl.SetHAlign(gtk.AlignStart)
	lbl.SetHExpand(true)

	box.Append(runeLabel)
	box.Append(lbl)

	btn.SetChild(box)
	btn.ConnectClicked(onClick)

	return btn
}
