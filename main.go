package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/diamondburned/gotk4/pkg/gdk/v4"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/hyprland"
	"github.com/javanhut/crowbar/src/layershell"
	"github.com/javanhut/crowbar/src/status"
)

func main() {
	app := gtk.NewApplication("com.github.javanhut.crowbar", 0)

	var hyprClient *hyprland.Client
	var eventListener *hyprland.EventListener
	var bar *status.Bar

	app.ConnectStartup(func() {
		// Load CSS styling
		loadCSS()

		// Check layer shell support on startup
		if !layershell.IsSupported() {
			fmt.Fprintln(os.Stderr, "Warning: Layer shell not supported. Running as regular window.")
			fmt.Fprintln(os.Stderr, "Make sure you're running on Wayland with a compositor that supports wlr-layer-shell.")
		}

		// Try to connect to Hyprland IPC
		var err error
		hyprClient, err = hyprland.NewClient()
		if err != nil {
			fmt.Fprintln(os.Stderr, "Warning: Could not connect to Hyprland IPC:", err)
			fmt.Fprintln(os.Stderr, "Workspace and window features will be disabled.")
		}

		// Set up event listener for real-time updates
		if hyprClient != nil {
			eventListener, err = hyprland.NewEventListener()
			if err != nil {
				fmt.Fprintln(os.Stderr, "Warning: Could not set up event listener:", err)
			}
		}
	})

	app.ConnectActivate(func() {
		bar = status.NewBar(app, hyprClient)

		// Connect event listener if available
		if eventListener != nil {
			bar.SetupEvents(eventListener)
		}

		bar.Show()
	})

	app.ConnectShutdown(func() {
		// Clean up bar resources (battery polling, etc.)
		if bar != nil {
			bar.Stop()
		}
		// Clean up event listener
		if eventListener != nil {
			eventListener.Stop()
		}
	})

	if code := app.Run(os.Args); code > 0 {
		os.Exit(code)
	}
}

// loadCSS loads the application CSS stylesheet
func loadCSS() {
	provider := gtk.NewCSSProvider()

	// Try to find CSS file in various locations
	cssLocations := []string{
		"style.css",                                               // Current directory
		filepath.Join(getExecutableDir(), "style.css"),            // Executable directory
		filepath.Join(os.Getenv("HOME"), ".config/crowbar/style.css"), // User config
		"/usr/share/crowbar/style.css",                            // System-wide
	}

	var cssPath string
	for _, loc := range cssLocations {
		if _, err := os.Stat(loc); err == nil {
			cssPath = loc
			break
		}
	}

	if cssPath == "" {
		fmt.Fprintln(os.Stderr, "Warning: Could not find style.css")
		return
	}

	provider.LoadFromPath(cssPath)

	// Add provider to default display
	display := gdk.DisplayGetDefault()
	if display != nil {
		gtk.StyleContextAddProviderForDisplay(display, provider, gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
	}
}

// getExecutableDir returns the directory containing the executable
func getExecutableDir() string {
	exe, err := os.Executable()
	if err != nil {
		return ""
	}
	return filepath.Dir(exe)
}
