package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/connectivity"
)

const (
	// ConnectivityUpdateInterval is how often to refresh status (in seconds)
	ConnectivityUpdateInterval = 5
)

// Connectivity represents the WiFi/Bluetooth status widget
type Connectivity struct {
	Box        *gtk.Box
	menuButton *gtk.MenuButton
	wifiIcon   *gtk.Image
	btIcon     *gtk.Image
	popover    *gtk.Popover
	wifiSwitch *gtk.Switch
	btSwitch   *gtk.Switch
	wifiLabel  *gtk.Label
	btLabel    *gtk.Label
	sourceID   glib.SourceHandle
	updating   bool
}

// NewConnectivity creates a new connectivity widget
func NewConnectivity() *Connectivity {
	c := &Connectivity{
		Box: gtk.NewBox(gtk.OrientationHorizontal, 0),
	}

	c.Box.AddCSSClass("connectivity")

	// Create menu button for popover
	c.menuButton = gtk.NewMenuButton()
	c.menuButton.AddCSSClass("connectivity-button")
	c.menuButton.SetHasFrame(false)

	// ᚱ Raidho - Journey, communication, connection
	rune := gtk.NewLabel("\u16B1")
	rune.AddCSSClass("module-rune")

	// Create button content with icons
	btnContent := gtk.NewBox(gtk.OrientationHorizontal, 4)
	btnContent.Append(rune)

	c.wifiIcon = gtk.NewImageFromIconName("network-wireless-symbolic")
	c.wifiIcon.AddCSSClass("connectivity-icon")
	btnContent.Append(c.wifiIcon)

	c.btIcon = gtk.NewImageFromIconName("bluetooth-symbolic")
	c.btIcon.AddCSSClass("connectivity-icon")
	btnContent.Append(c.btIcon)

	c.menuButton.SetChild(btnContent)

	// Create popover with controls
	c.popover = gtk.NewPopover()
	c.popover.AddCSSClass("connectivity-popover")
	c.popover.SetAutohide(true)

	popoverContent := gtk.NewBox(gtk.OrientationVertical, 8)
	popoverContent.SetMarginTop(12)
	popoverContent.SetMarginBottom(12)
	popoverContent.SetMarginStart(12)
	popoverContent.SetMarginEnd(12)

	// Header
	header := gtk.NewBox(gtk.OrientationHorizontal, 8)
	headerRune := gtk.NewLabel("\u16B1")
	headerRune.AddCSSClass("slider-header-rune")
	headerLabel := gtk.NewLabel("Bifrost")
	headerLabel.AddCSSClass("slider-header-label")
	header.Append(headerRune)
	header.Append(headerLabel)
	popoverContent.Append(header)

	// WiFi section
	wifiSection := c.createWiFiSection()
	popoverContent.Append(wifiSection)

	// Separator
	sep := gtk.NewSeparator(gtk.OrientationHorizontal)
	sep.AddCSSClass("connectivity-separator")
	popoverContent.Append(sep)

	// Bluetooth section
	btSection := c.createBluetoothSection()
	popoverContent.Append(btSection)

	c.popover.SetChild(popoverContent)
	c.menuButton.SetPopover(c.popover)

	c.Box.Append(c.menuButton)

	// Initial refresh
	c.Refresh()

	// Start periodic updates
	c.startUpdates()

	return c
}

// createWiFiSection creates the WiFi control section
func (c *Connectivity) createWiFiSection() *gtk.Box {
	section := gtk.NewBox(gtk.OrientationVertical, 6)
	section.AddCSSClass("connectivity-section")

	// Row with icon, label, and switch
	row := gtk.NewBox(gtk.OrientationHorizontal, 8)

	// WiFi rune - ᚹ Wunjo (joy, connection)
	wifiRune := gtk.NewLabel("\u16B9")
	wifiRune.AddCSSClass("connectivity-rune")
	row.Append(wifiRune)

	// Icon
	icon := gtk.NewImageFromIconName("network-wireless-symbolic")
	icon.AddCSSClass("connectivity-section-icon")
	row.Append(icon)

	// Label
	labelBox := gtk.NewBox(gtk.OrientationVertical, 2)
	labelBox.SetHExpand(true)

	title := gtk.NewLabel("WiFi")
	title.AddCSSClass("connectivity-title")
	title.SetHAlign(gtk.AlignStart)
	labelBox.Append(title)

	c.wifiLabel = gtk.NewLabel("Disabled")
	c.wifiLabel.AddCSSClass("connectivity-status")
	c.wifiLabel.SetHAlign(gtk.AlignStart)
	labelBox.Append(c.wifiLabel)

	row.Append(labelBox)

	// Switch
	c.wifiSwitch = gtk.NewSwitch()
	c.wifiSwitch.AddCSSClass("connectivity-switch")
	c.wifiSwitch.SetVAlign(gtk.AlignCenter)
	c.wifiSwitch.ConnectStateSet(func(state bool) bool {
		if c.updating {
			return false
		}
		connectivity.SetWiFiEnabled(state)
		glib.TimeoutAdd(500, func() bool {
			c.Refresh()
			return false
		})
		return false
	})
	row.Append(c.wifiSwitch)

	section.Append(row)

	return section
}

// createBluetoothSection creates the Bluetooth control section
func (c *Connectivity) createBluetoothSection() *gtk.Box {
	section := gtk.NewBox(gtk.OrientationVertical, 6)
	section.AddCSSClass("connectivity-section")

	// Row with icon, label, and switch
	row := gtk.NewBox(gtk.OrientationHorizontal, 8)

	// Bluetooth rune - ᛒ Berkano (growth, connection)
	btRune := gtk.NewLabel("\u16D2")
	btRune.AddCSSClass("connectivity-rune")
	row.Append(btRune)

	// Icon
	icon := gtk.NewImageFromIconName("bluetooth-symbolic")
	icon.AddCSSClass("connectivity-section-icon")
	row.Append(icon)

	// Label
	labelBox := gtk.NewBox(gtk.OrientationVertical, 2)
	labelBox.SetHExpand(true)

	title := gtk.NewLabel("Bluetooth")
	title.AddCSSClass("connectivity-title")
	title.SetHAlign(gtk.AlignStart)
	labelBox.Append(title)

	c.btLabel = gtk.NewLabel("Disabled")
	c.btLabel.AddCSSClass("connectivity-status")
	c.btLabel.SetHAlign(gtk.AlignStart)
	labelBox.Append(c.btLabel)

	row.Append(labelBox)

	// Switch
	c.btSwitch = gtk.NewSwitch()
	c.btSwitch.AddCSSClass("connectivity-switch")
	c.btSwitch.SetVAlign(gtk.AlignCenter)
	c.btSwitch.ConnectStateSet(func(state bool) bool {
		if c.updating {
			return false
		}
		connectivity.SetBluetoothEnabled(state)
		glib.TimeoutAdd(500, func() bool {
			c.Refresh()
			return false
		})
		return false
	})
	row.Append(c.btSwitch)

	section.Append(row)

	return section
}

// startUpdates begins periodic status updates
func (c *Connectivity) startUpdates() {
	c.sourceID = glib.TimeoutAdd(ConnectivityUpdateInterval*1000, func() bool {
		c.Refresh()
		return true
	})
}

// Stop stops the periodic updates
func (c *Connectivity) Stop() {
	if c.sourceID > 0 {
		glib.SourceRemove(c.sourceID)
		c.sourceID = 0
	}
}

// Refresh updates the connectivity display
func (c *Connectivity) Refresh() {
	c.updating = true
	defer func() { c.updating = false }()

	// Update WiFi status
	wifiInfo := connectivity.GetWiFiInfo()
	c.updateWiFiDisplay(wifiInfo)

	// Update Bluetooth status
	btInfo := connectivity.GetBluetoothInfo()
	c.updateBluetoothDisplay(btInfo)

	// Update tooltip
	c.updateTooltip(wifiInfo, btInfo)
}

// updateWiFiDisplay updates WiFi UI elements
func (c *Connectivity) updateWiFiDisplay(info *connectivity.WiFiInfo) {
	// Update icon in button
	iconName := connectivity.GetWiFiIcon(info)
	c.wifiIcon.SetFromIconName(iconName)

	// Update switch state
	c.wifiSwitch.SetActive(info.Enabled)

	// Update status label
	if !info.Enabled {
		c.wifiLabel.SetText("Disabled")
		c.wifiLabel.RemoveCSSClass("connected")
	} else if info.Connected {
		c.wifiLabel.SetText(fmt.Sprintf("%s (%d%%)", info.SSID, info.Signal))
		c.wifiLabel.AddCSSClass("connected")
	} else {
		c.wifiLabel.SetText("Not connected")
		c.wifiLabel.RemoveCSSClass("connected")
	}

	// Update button CSS
	if info.Enabled {
		c.Box.RemoveCSSClass("wifi-disabled")
	} else {
		c.Box.AddCSSClass("wifi-disabled")
	}
}

// updateBluetoothDisplay updates Bluetooth UI elements
func (c *Connectivity) updateBluetoothDisplay(info *connectivity.BluetoothInfo) {
	// Update icon in button
	iconName := connectivity.GetBluetoothIcon(info)
	c.btIcon.SetFromIconName(iconName)

	// Update switch state
	if info.Available {
		c.btSwitch.SetSensitive(true)
		c.btSwitch.SetActive(info.Powered)
	} else {
		c.btSwitch.SetSensitive(false)
		c.btSwitch.SetActive(false)
	}

	// Update status label
	if !info.Available {
		c.btLabel.SetText("Not available")
		c.btLabel.RemoveCSSClass("connected")
	} else if !info.Powered {
		c.btLabel.SetText("Disabled")
		c.btLabel.RemoveCSSClass("connected")
	} else if info.Connected {
		c.btLabel.SetText(info.Device)
		c.btLabel.AddCSSClass("connected")
	} else {
		c.btLabel.SetText("Not connected")
		c.btLabel.RemoveCSSClass("connected")
	}

	// Update button CSS
	if info.Available && info.Powered {
		c.Box.RemoveCSSClass("bt-disabled")
	} else {
		c.Box.AddCSSClass("bt-disabled")
	}
}

// updateTooltip updates the tooltip text
func (c *Connectivity) updateTooltip(wifi *connectivity.WiFiInfo, bt *connectivity.BluetoothInfo) {
	var tooltip string

	// WiFi status
	if wifi.Enabled {
		if wifi.Connected {
			tooltip = fmt.Sprintf("WiFi: %s (%d%%)", wifi.SSID, wifi.Signal)
		} else {
			tooltip = "WiFi: Enabled (not connected)"
		}
	} else {
		tooltip = "WiFi: Disabled"
	}

	// Bluetooth status
	if bt.Available {
		if bt.Powered {
			if bt.Connected {
				tooltip += fmt.Sprintf("\nBluetooth: %s", bt.Device)
			} else {
				tooltip += "\nBluetooth: Enabled (not connected)"
			}
		} else {
			tooltip += "\nBluetooth: Disabled"
		}
	} else {
		tooltip += "\nBluetooth: Not available"
	}

	tooltip += "\nClick to configure"
	c.Box.SetTooltipText(tooltip)
}
