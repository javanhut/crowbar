package hyprland

import (
	"fmt"

	hyprlandgo "github.com/thiagokokada/hyprland-go"
)

// Re-export types for convenience
type Workspace = hyprlandgo.Workspace
type Window = hyprlandgo.Window

// Client wraps the Hyprland IPC client
type Client struct {
	ipc *hyprlandgo.RequestClient
}

// NewClient creates a new Hyprland IPC client
func NewClient() (*Client, error) {
	ipc := hyprlandgo.MustClient()
	if ipc == nil {
		return nil, fmt.Errorf("failed to connect to Hyprland IPC")
	}
	return &Client{ipc: ipc}, nil
}

// ActiveWorkspace returns the currently focused workspace
func (c *Client) ActiveWorkspace() (Workspace, error) {
	return c.ipc.ActiveWorkspace()
}

// Workspaces returns all workspaces
func (c *Client) Workspaces() ([]Workspace, error) {
	return c.ipc.Workspaces()
}

// ActiveWindow returns the currently focused window
func (c *Client) ActiveWindow() (Window, error) {
	return c.ipc.ActiveWindow()
}

// SwitchWorkspace switches to the specified workspace
func (c *Client) SwitchWorkspace(id int) error {
	_, err := c.ipc.Dispatch(fmt.Sprintf("workspace %d", id))
	return err
}
