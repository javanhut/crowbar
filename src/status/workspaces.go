package status

import (
	"fmt"
	"sort"

	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/hyprland"
)

// Workspaces represents the workspace indicator widget
type Workspaces struct {
	Box      *gtk.Box
	buttons  map[int]*gtk.Button
	client   *hyprland.Client
	activeID int
}

// NewWorkspaces creates a new workspaces widget
func NewWorkspaces(client *hyprland.Client) *Workspaces {
	ws := &Workspaces{
		Box:     gtk.NewBox(gtk.OrientationHorizontal, 2),
		buttons: make(map[int]*gtk.Button),
		client:  client,
	}

	ws.Box.AddCSSClass("workspaces")
	ws.Refresh()

	return ws
}

// Refresh updates the workspace display
func (ws *Workspaces) Refresh() {
	if ws.client == nil {
		return
	}

	workspaces, err := ws.client.Workspaces()
	if err != nil {
		return
	}

	active, err := ws.client.ActiveWorkspace()
	if err != nil {
		return
	}
	ws.activeID = active.Id

	// Clear existing buttons
	for ws.Box.FirstChild() != nil {
		ws.Box.Remove(ws.Box.FirstChild())
	}
	ws.buttons = make(map[int]*gtk.Button)

	// Sort workspaces by ID
	sort.Slice(workspaces, func(i, j int) bool {
		return workspaces[i].Id < workspaces[j].Id
	})

	// Create buttons for each workspace
	for _, workspace := range workspaces {
		btn := gtk.NewButton()
		btn.SetLabel(fmt.Sprintf("%d", workspace.Id))
		btn.AddCSSClass("workspace-btn")

		if workspace.Id == ws.activeID {
			btn.AddCSSClass("active")
		} else if workspace.Windows > 0 {
			btn.AddCSSClass("occupied")
		}

		// Capture workspace ID for click handler
		wsID := workspace.Id
		btn.ConnectClicked(func() {
			ws.client.SwitchWorkspace(wsID)
		})

		ws.buttons[workspace.Id] = btn
		ws.Box.Append(btn)
	}
}

// SetActive updates the active workspace display
func (ws *Workspaces) SetActive(id int) {
	if oldBtn, ok := ws.buttons[ws.activeID]; ok {
		oldBtn.RemoveCSSClass("active")
	}
	ws.activeID = id
	if newBtn, ok := ws.buttons[id]; ok {
		newBtn.AddCSSClass("active")
	}
}
