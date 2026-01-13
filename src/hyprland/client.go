package hyprland

import (
	"fmt"

	hyprlandgo "github.com/thiagokokada/hyprland-go"
)

// Re-export types for convenience
type Workspace = hyprlandgo.Workspace
type Window = hyprlandgo.Window
type HyprClient = hyprlandgo.Client

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

// Clients returns all windows/clients
func (c *Client) Clients() ([]HyprClient, error) {
	return c.ipc.Clients()
}

// FocusWindow focuses a window by its address
func (c *Client) FocusWindow(address string) error {
	_, err := c.ipc.Dispatch(fmt.Sprintf("focuswindow address:%s", address))
	return err
}

// CloseWindow closes a window by its address
func (c *Client) CloseWindow(address string) error {
	_, err := c.ipc.Dispatch(fmt.Sprintf("closewindow address:%s", address))
	return err
}

// Dispatch sends a raw command to Hyprland
func (c *Client) Dispatch(cmd string) error {
	_, err := c.ipc.Dispatch(cmd)
	return err
}

// MinimizeWindow minimizes a window by moving it to a special workspace
func (c *Client) MinimizeWindow(address string) error {
	_, err := c.ipc.Dispatch(fmt.Sprintf("movetoworkspacesilent special:minimized,address:%s", address))
	return err
}

// RestoreWindow restores a minimized window to the current workspace
func (c *Client) RestoreWindow(address string) error {
	// First move to current workspace, then focus
	_, err := c.ipc.Dispatch(fmt.Sprintf("movetoworkspace e+0,address:%s", address))
	if err != nil {
		return err
	}
	return c.FocusWindow(address)
}
