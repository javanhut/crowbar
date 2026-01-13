package status

import (
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/diamondburned/gotk4/pkg/pango"
	"github.com/javanhut/crowbar/src/hyprland"
)

// WindowTitle displays the active window title
type WindowTitle struct {
	Label  *gtk.Label
	client *hyprland.Client
}

// NewWindowTitle creates a new window title widget
func NewWindowTitle(client *hyprland.Client) *WindowTitle {
	wt := &WindowTitle{
		Label:  gtk.NewLabel(""),
		client: client,
	}

	wt.Label.AddCSSClass("window-title")
	wt.Label.SetEllipsize(pango.EllipsizeEnd)
	wt.Label.SetMaxWidthChars(50)
	wt.Label.SetHAlign(gtk.AlignStart)
	wt.Label.SetHExpand(true)

	wt.Refresh()

	return wt
}

// Refresh updates the window title display
func (wt *WindowTitle) Refresh() {
	if wt.client == nil {
		wt.Label.SetLabel("")
		return
	}

	window, err := wt.client.ActiveWindow()
	if err != nil {
		wt.Label.SetLabel("")
		return
	}

	title := window.Title
	if title == "" {
		title = window.Class
	}
	wt.Label.SetLabel(title)
}

// SetTitle sets the window title directly
func (wt *WindowTitle) SetTitle(title string) {
	wt.Label.SetLabel(title)
}
