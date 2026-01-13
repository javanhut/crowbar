package status

import (
	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/hyprland"
	"github.com/javanhut/crowbar/src/layershell"
)

const (
	// BarHeight is the height of the status bar in pixels
	BarHeight = 32
)

// Bar represents the main status bar window
type Bar struct {
	Window       *gtk.Window
	box          *gtk.Box
	workspaces   *Workspaces
	windowTitle  *WindowTitle
	appTracker   *AppTracker
	media        *Media
	systray      *Systray
	connectivity *Connectivity
	audio        *Audio
	brightness   *Brightness
	power        *Power
	battery      *Battery
	clock        *Clock
	powerBtn     *PowerButton
	client       *hyprland.Client
}

// NewBar creates a new status bar instance
func NewBar(app *gtk.Application, client *hyprland.Client) *Bar {
	bar := &Bar{
		client: client,
	}

	// Create window - using gtk.Window for layer shell compatibility
	bar.Window = gtk.NewWindow()
	bar.Window.SetTitle("CrowBar")

	// Add window to application
	app.AddWindow(bar.Window)

	// Initialize as layer surface BEFORE showing
	// This must happen before the window is realized
	if layershell.IsSupported() {
		layershell.InitForWindow(bar.Window)

		// Set to top layer (above normal windows, below overlays)
		layershell.SetLayer(bar.Window, layershell.LayerTop)

		// Anchor to top edge, stretch across left to right
		layershell.SetAnchor(bar.Window, layershell.EdgeTop, true)
		layershell.SetAnchor(bar.Window, layershell.EdgeLeft, true)
		layershell.SetAnchor(bar.Window, layershell.EdgeRight, true)

		// Set exclusive zone so windows don't overlap the bar
		layershell.AutoExclusiveZoneEnable(bar.Window)

		// Set namespace for compositor identification
		layershell.SetNamespace(bar.Window, "crowbar")

		// No keyboard focus by default (status bar doesn't need it)
		layershell.SetKeyboardMode(bar.Window, layershell.KeyboardModeNone)
	}

	// Set height (width is determined by anchoring)
	bar.Window.SetDefaultSize(-1, BarHeight)

	// Create horizontal box for modules
	bar.box = gtk.NewBox(gtk.OrientationHorizontal, 4)
	bar.box.AddCSSClass("bar-container")
	bar.box.SetMarginStart(4)
	bar.box.SetMarginEnd(4)
	bar.box.SetMarginTop(2)
	bar.box.SetMarginBottom(2)
	bar.Window.SetChild(bar.box)

	// Add workspaces widget (left side)
	if client != nil {
		bar.workspaces = NewWorkspaces(client)
		bar.box.Append(bar.workspaces.Box)

		// Add separator
		sep := gtk.NewSeparator(gtk.OrientationVertical)
		bar.box.Append(sep)

		// Add window title widget (center, expanding)
		bar.windowTitle = NewWindowTitle(client)
		bar.box.Append(bar.windowTitle.Label)
	} else {
		// Fallback label if no Hyprland connection
		label := gtk.NewLabel("CrowBar - Not connected to Hyprland")
		label.SetHExpand(true)
		bar.box.Append(label)
	}

	// Add app tracker (after window title)
	if client != nil {
		bar.appTracker = NewAppTracker(client)
		bar.box.Append(bar.appTracker.Box)
	}

	// Add media player widget
	bar.media = NewMedia()
	bar.box.Append(bar.media.Box)

	// Add spacer to push right-side widgets to the right
	spacer := gtk.NewBox(gtk.OrientationHorizontal, 0)
	spacer.SetHExpand(true)
	bar.box.Append(spacer)

	// Add system tray (right side)
	bar.systray = NewSystray()
	bar.box.Append(bar.systray.Box)

	// Add connectivity widget (WiFi/Bluetooth)
	bar.connectivity = NewConnectivity()
	bar.box.Append(bar.connectivity.Box)

	// Add audio widget (right side)
	bar.audio = NewAudio()
	bar.box.Append(bar.audio.Box)

	// Add brightness widget (right side)
	bar.brightness = NewBrightness()
	bar.box.Append(bar.brightness.Box)

	// Add power widget (right side)
	bar.power = NewPower()
	bar.box.Append(bar.power.Box)

	// Add battery widget (right side)
	bar.battery = NewBattery()
	bar.box.Append(bar.battery.Box)

	// Add separator before clock
	sep2 := gtk.NewSeparator(gtk.OrientationVertical)
	bar.box.Append(sep2)

	// Add clock widget (right side)
	bar.clock = NewClock()
	bar.box.Append(bar.clock.Box)

	// Add power button (far right)
	bar.powerBtn = NewPowerButton()
	bar.box.Append(bar.powerBtn.Button)

	return bar
}

// SetupEvents configures event handlers for real-time updates
func (b *Bar) SetupEvents(listener *hyprland.EventListener) {
	// Setup audio events (works independently of Hyprland)
	if b.audio != nil {
		b.audio.SetupEvents()
	}

	if b.workspaces == nil || b.windowTitle == nil {
		return
	}

	// Workspace change events
	listener.On(hyprland.EventWorkspace, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			b.workspaces.Refresh()
		})
	})

	listener.On(hyprland.EventCreateWorkspace, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			b.workspaces.Refresh()
		})
	})

	listener.On(hyprland.EventDestroyWorkspace, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			b.workspaces.Refresh()
		})
	})

	// Window events
	listener.On(hyprland.EventActiveWindow, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			b.windowTitle.Refresh()
		})
	})

	listener.On(hyprland.EventWindowTitle, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			b.windowTitle.Refresh()
		})
	})

	listener.On(hyprland.EventCloseWindow, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			b.windowTitle.Refresh()
			if b.appTracker != nil {
				b.appTracker.Refresh()
			}
		})
	})

	// Window open event for app tracker
	listener.On(hyprland.EventOpenWindow, func(e hyprland.Event) {
		glib.IdleAdd(func() {
			if b.appTracker != nil {
				b.appTracker.Refresh()
			}
		})
	})

	listener.Start()
}

// Show displays the status bar window
func (b *Bar) Show() {
	b.Window.Show()
}

// Stop cleans up bar resources
func (b *Bar) Stop() {
	if b.appTracker != nil {
		b.appTracker.Stop()
	}
	if b.media != nil {
		b.media.Stop()
	}
	if b.systray != nil {
		b.systray.Stop()
	}
	if b.connectivity != nil {
		b.connectivity.Stop()
	}
	if b.audio != nil {
		b.audio.Stop()
	}
	if b.brightness != nil {
		b.brightness.Stop()
	}
	if b.power != nil {
		b.power.Stop()
	}
	if b.battery != nil {
		b.battery.Stop()
	}
	if b.clock != nil {
		b.clock.Stop()
	}
}
