package status

import (
	"fmt"
	"sort"
	"strings"

	"github.com/diamondburned/gotk4/pkg/gdk/v4"
	"github.com/diamondburned/gotk4/pkg/gio/v2"
	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/hyprland"
)

// WindowInfo holds info about a single window
type WindowInfo struct {
	Address   string
	Minimized bool
}

// AppInfo represents a running application with its windows
type AppInfo struct {
	Class     string       // Application class (e.g., "firefox")
	Title     string       // Display title
	Windows   []WindowInfo // Window info with addresses and state
	Focused   bool         // Whether any window is focused
	AllMinimized bool      // Whether all windows are minimized
}

// AppTracker represents the running applications widget
type AppTracker struct {
	Box      *gtk.Box
	client   *hyprland.Client
	apps     map[string]*AppInfo // Keyed by class
	buttons  map[string]*gtk.Button
	sourceID glib.SourceHandle
}

// NewAppTracker creates a new app tracker widget
func NewAppTracker(client *hyprland.Client) *AppTracker {
	a := &AppTracker{
		Box:     gtk.NewBox(gtk.OrientationHorizontal, 2),
		client:  client,
		apps:    make(map[string]*AppInfo),
		buttons: make(map[string]*gtk.Button),
	}

	a.Box.AddCSSClass("app-tracker")

	// ᛗ Mannaz - Humanity, community of people/apps
	rune := gtk.NewLabel("\u16D7")
	rune.AddCSSClass("module-rune")
	rune.SetTooltipText("ᛗ Mannaz - Running Apps")
	a.Box.Append(rune)

	// Initial refresh
	a.Refresh()

	// Start periodic updates (every 2 seconds)
	a.startUpdates()

	return a
}

// startUpdates begins periodic app tracking updates
func (a *AppTracker) startUpdates() {
	a.sourceID = glib.TimeoutAdd(2000, func() bool {
		a.Refresh()
		return true
	})
}

// Stop stops the periodic updates
func (a *AppTracker) Stop() {
	if a.sourceID > 0 {
		glib.SourceRemove(a.sourceID)
		a.sourceID = 0
	}
}

// Refresh updates the app tracker display
func (a *AppTracker) Refresh() {
	if a.client == nil {
		return
	}

	// Get all clients
	clients, err := a.client.Clients()
	if err != nil {
		return
	}

	// Get active window to mark focused app
	activeWindow, _ := a.client.ActiveWindow()
	activeClass := strings.ToLower(activeWindow.Class)

	// Group windows by class
	newApps := make(map[string]*AppInfo)
	for _, client := range clients {
		class := strings.ToLower(client.Class)
		if class == "" {
			continue
		}

		// Check if window is minimized (on special:minimized workspace)
		isMinimized := strings.HasPrefix(client.Workspace.Name, "special:minimized")

		winInfo := WindowInfo{
			Address:   client.Address,
			Minimized: isMinimized,
		}

		if app, exists := newApps[class]; exists {
			app.Windows = append(app.Windows, winInfo)
			if class == activeClass {
				app.Focused = true
			}
		} else {
			newApps[class] = &AppInfo{
				Class:   class,
				Title:   client.Class, // Use original case for display
				Windows: []WindowInfo{winInfo},
				Focused: class == activeClass,
			}
		}
	}

	// Calculate AllMinimized for each app
	for _, app := range newApps {
		allMin := true
		for _, win := range app.Windows {
			if !win.Minimized {
				allMin = false
				break
			}
		}
		app.AllMinimized = allMin
	}

	// Check if apps changed
	appsChanged := len(newApps) != len(a.apps)
	if !appsChanged {
		for class := range newApps {
			if _, exists := a.apps[class]; !exists {
				appsChanged = true
				break
			}
			// Check if window count changed
			if len(newApps[class].Windows) != len(a.apps[class].Windows) {
				appsChanged = true
				break
			}
			// Check if minimized state changed
			if newApps[class].AllMinimized != a.apps[class].AllMinimized {
				appsChanged = true
				break
			}
		}
	}

	// Update app data
	a.apps = newApps

	// Rebuild UI if apps changed
	if appsChanged {
		a.rebuildUI()
	} else {
		// Just update focused state
		a.updateFocusState()
	}
}

// rebuildUI recreates all app buttons
func (a *AppTracker) rebuildUI() {
	// Remove old buttons
	for _, btn := range a.buttons {
		a.Box.Remove(btn)
	}
	a.buttons = make(map[string]*gtk.Button)

	// Sort apps alphabetically
	var classes []string
	for class := range a.apps {
		classes = append(classes, class)
	}
	sort.Strings(classes)

	// Create buttons for each app
	for _, class := range classes {
		app := a.apps[class]
		btn := a.createAppButton(app)
		a.buttons[class] = btn
		a.Box.Append(btn)
	}
}

// createAppButton creates a button for an application
func (a *AppTracker) createAppButton(app *AppInfo) *gtk.Button {
	btn := gtk.NewButton()
	btn.AddCSSClass("app-button")

	// Create content box
	content := gtk.NewBox(gtk.OrientationHorizontal, 4)

	// Try to get app icon
	icon := a.getAppIcon(app.Class)
	if icon != nil {
		icon.AddCSSClass("app-icon")
		content.Append(icon)
	}

	// Add window count if more than 1
	if len(app.Windows) > 1 {
		countLabel := gtk.NewLabel(fmt.Sprintf("%d", len(app.Windows)))
		countLabel.AddCSSClass("app-count")
		content.Append(countLabel)
	}

	btn.SetChild(content)

	// Count minimized windows
	minimizedCount := 0
	for _, win := range app.Windows {
		if win.Minimized {
			minimizedCount++
		}
	}

	// Set tooltip
	var tooltip string
	if minimizedCount > 0 {
		tooltip = fmt.Sprintf("%s\n%d window(s) (%d minimized)\nClick: Focus/Restore | Right-click: Options",
			app.Title, len(app.Windows), minimizedCount)
	} else {
		tooltip = fmt.Sprintf("%s\n%d window(s)\nClick: Focus | Right-click: Options",
			app.Title, len(app.Windows))
	}
	btn.SetTooltipText(tooltip)

	// Update focused state
	if app.Focused {
		btn.AddCSSClass("focused")
	}

	// Update minimized state
	if app.AllMinimized {
		btn.AddCSSClass("minimized")
	}

	// Store class in button name for event handlers
	btn.SetName(app.Class)

	// Left click - focus/cycle windows or restore if minimized
	btn.ConnectClicked(func() {
		a.onAppClicked(app.Class)
	})

	// Right click - context menu
	a.setupRightClick(btn, app.Class)

	return btn
}

// getAppIcon tries to find an icon for the application
func (a *AppTracker) getAppIcon(class string) *gtk.Image {
	// Common icon name mappings
	iconMappings := map[string]string{
		"firefox":          "firefox",
		"chromium":         "chromium",
		"google-chrome":    "google-chrome",
		"code":             "visual-studio-code",
		"code-oss":         "code-oss",
		"discord":          "discord",
		"spotify":          "spotify",
		"steam":            "steam",
		"telegram-desktop": "telegram",
		"thunar":           "thunar",
		"nautilus":         "nautilus",
		"kitty":            "kitty",
		"alacritty":        "Alacritty",
		"foot":             "foot",
		"wezterm":          "wezterm",
		"obsidian":         "obsidian",
		"vlc":              "vlc",
		"mpv":              "mpv",
		"gimp":             "gimp",
		"inkscape":         "inkscape",
		"blender":          "blender",
		"libreoffice":      "libreoffice-startcenter",
	}

	// Try mapped name first
	iconName := class
	if mapped, exists := iconMappings[strings.ToLower(class)]; exists {
		iconName = mapped
	}

	// Check if icon exists in theme
	iconTheme := gtk.IconThemeGetForDisplay(gdk.DisplayGetDefault())
	if iconTheme.HasIcon(iconName) {
		return gtk.NewImageFromIconName(iconName)
	}

	// Try lowercase
	if iconTheme.HasIcon(strings.ToLower(iconName)) {
		return gtk.NewImageFromIconName(strings.ToLower(iconName))
	}

	// Fallback to generic application icon
	return gtk.NewImageFromIconName("application-x-executable")
}

// onAppClicked handles left click - focus, cycle, or restore windows
func (a *AppTracker) onAppClicked(class string) {
	app, exists := a.apps[class]
	if !exists || len(app.Windows) == 0 {
		return
	}

	// If all windows are minimized, restore the first one
	if app.AllMinimized {
		a.client.RestoreWindow(app.Windows[0].Address)
		return
	}

	// Get non-minimized windows
	var visibleWindows []WindowInfo
	for _, win := range app.Windows {
		if !win.Minimized {
			visibleWindows = append(visibleWindows, win)
		}
	}

	if len(visibleWindows) == 0 {
		return
	}

	if len(visibleWindows) == 1 {
		// Single visible window - just focus it
		a.client.FocusWindow(visibleWindows[0].Address)
	} else {
		// Multiple visible windows - cycle through them
		activeWindow, _ := a.client.ActiveWindow()
		currentIdx := -1
		for i, win := range visibleWindows {
			if win.Address == activeWindow.Address {
				currentIdx = i
				break
			}
		}

		// Focus next window
		nextIdx := (currentIdx + 1) % len(visibleWindows)
		a.client.FocusWindow(visibleWindows[nextIdx].Address)
	}
}

// setupRightClick configures the right-click context menu
func (a *AppTracker) setupRightClick(btn *gtk.Button, class string) {
	// Create gesture for right-click
	gesture := gtk.NewGestureClick()
	gesture.SetButton(gdk.BUTTON_SECONDARY)

	gesture.ConnectPressed(func(nPress int, x, y float64) {
		a.showContextMenu(btn, class, x, y)
	})

	btn.AddController(gesture)
}

// showContextMenu displays the right-click menu
func (a *AppTracker) showContextMenu(btn *gtk.Button, class string, x, y float64) {
	app, exists := a.apps[class]
	if !exists {
		return
	}

	// Create popover menu
	popover := gtk.NewPopover()
	popover.AddCSSClass("app-menu")
	popover.SetParent(btn)
	popover.SetPosition(gtk.PosBottom)
	popover.SetAutohide(true)

	content := gtk.NewBox(gtk.OrientationVertical, 4)
	content.SetMarginTop(8)
	content.SetMarginBottom(8)
	content.SetMarginStart(8)
	content.SetMarginEnd(8)

	// Header
	header := gtk.NewLabel(app.Title)
	header.AddCSSClass("app-menu-header")
	content.Append(header)

	// Separator
	sep := gtk.NewSeparator(gtk.OrientationHorizontal)
	content.Append(sep)

	// Count minimized and visible windows
	minimizedCount := 0
	visibleCount := 0
	for _, win := range app.Windows {
		if win.Minimized {
			minimizedCount++
		} else {
			visibleCount++
		}
	}

	// Minimize option (if there are visible windows)
	if visibleCount > 0 {
		var minLabel string
		if visibleCount == 1 {
			minLabel = "Minimize"
		} else {
			minLabel = fmt.Sprintf("Minimize All (%d)", visibleCount)
		}
		minimizeBtn := a.createMenuItem("\u16BE", minLabel, func() {
			for _, win := range app.Windows {
				if !win.Minimized {
					a.client.MinimizeWindow(win.Address)
				}
			}
			popover.Popdown()
			glib.TimeoutAdd(100, func() bool {
				a.Refresh()
				return false
			})
		})
		content.Append(minimizeBtn)
	}

	// Restore option (if there are minimized windows)
	if minimizedCount > 0 {
		var restoreLabel string
		if minimizedCount == 1 {
			restoreLabel = "Restore"
		} else {
			restoreLabel = fmt.Sprintf("Restore All (%d)", minimizedCount)
		}
		restoreBtn := a.createMenuItem("\u16D2", restoreLabel, func() {
			for _, win := range app.Windows {
				if win.Minimized {
					a.client.RestoreWindow(win.Address)
				}
			}
			popover.Popdown()
			glib.TimeoutAdd(100, func() bool {
				a.Refresh()
				return false
			})
		})
		content.Append(restoreBtn)
	}

	// Separator before close options
	sep2 := gtk.NewSeparator(gtk.OrientationHorizontal)
	content.Append(sep2)

	// Close window option
	closeBtn := a.createMenuItem("\u16C1", "Close Window", func() {
		if len(app.Windows) > 0 {
			a.client.CloseWindow(app.Windows[0].Address)
			popover.Popdown()
			glib.TimeoutAdd(100, func() bool {
				a.Refresh()
				return false
			})
		}
	})
	content.Append(closeBtn)

	// Close all windows option (if more than 1)
	if len(app.Windows) > 1 {
		closeAllBtn := a.createMenuItem("\u16BA", fmt.Sprintf("Close All (%d)", len(app.Windows)), func() {
			for _, win := range app.Windows {
				a.client.CloseWindow(win.Address)
			}
			popover.Popdown()
			glib.TimeoutAdd(100, func() bool {
				a.Refresh()
				return false
			})
		})
		closeAllBtn.AddCSSClass("app-menu-danger")
		content.Append(closeAllBtn)
	}

	// New instance option
	sep3 := gtk.NewSeparator(gtk.OrientationHorizontal)
	content.Append(sep3)

	newBtn := a.createMenuItem("\u16A0", "New Instance", func() {
		a.launchApp(class)
		popover.Popdown()
	})
	content.Append(newBtn)

	popover.SetChild(content)
	popover.Popup()
}

// createMenuItem creates a menu item button
func (a *AppTracker) createMenuItem(runeChar, label string, onClick func()) *gtk.Button {
	btn := gtk.NewButton()
	btn.AddCSSClass("app-menu-item")

	box := gtk.NewBox(gtk.OrientationHorizontal, 8)

	runeLabel := gtk.NewLabel(runeChar)
	runeLabel.AddCSSClass("app-menu-rune")

	textLabel := gtk.NewLabel(label)
	textLabel.AddCSSClass("app-menu-label")

	box.Append(runeLabel)
	box.Append(textLabel)
	btn.SetChild(box)

	btn.ConnectClicked(func() {
		onClick()
	})

	return btn
}

// launchApp attempts to launch a new instance of the application
func (a *AppTracker) launchApp(class string) {
	// Try to find by desktop file
	apps := gio.AppInfoGetAll()
	for _, app := range apps {
		if app == nil {
			continue
		}
		name := strings.ToLower(app.Name())
		id := strings.ToLower(app.ID())
		if strings.Contains(name, class) || strings.Contains(id, class) {
			app.Launch(nil, nil)
			return
		}
	}

	// Fallback: try to execute the class name directly via Hyprland
	a.client.Dispatch(fmt.Sprintf("exec %s", class))
}

// updateFocusState updates the focused and minimized CSS classes on buttons
func (a *AppTracker) updateFocusState() {
	for class, btn := range a.buttons {
		if app, exists := a.apps[class]; exists {
			// Update focused state
			if app.Focused {
				btn.AddCSSClass("focused")
			} else {
				btn.RemoveCSSClass("focused")
			}

			// Update minimized state
			if app.AllMinimized {
				btn.AddCSSClass("minimized")
			} else {
				btn.RemoveCSSClass("minimized")
			}

			// Count minimized windows
			minimizedCount := 0
			for _, win := range app.Windows {
				if win.Minimized {
					minimizedCount++
				}
			}

			// Update tooltip
			var tooltip string
			if minimizedCount > 0 {
				tooltip = fmt.Sprintf("%s\n%d window(s) (%d minimized)\nClick: Focus/Restore | Right-click: Options",
					app.Title, len(app.Windows), minimizedCount)
			} else {
				tooltip = fmt.Sprintf("%s\n%d window(s)\nClick: Focus | Right-click: Options",
					app.Title, len(app.Windows))
			}
			btn.SetTooltipText(tooltip)
		}
	}
}
