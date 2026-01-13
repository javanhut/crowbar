package status

import (
	"fmt"
	"time"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
)

const (
	// ClockUpdateInterval is how often to refresh the clock (in seconds)
	ClockUpdateInterval = 1
)

// Clock represents the time/date widget
type Clock struct {
	Box      *gtk.Box
	timeLabel *gtk.Label
	dateLabel *gtk.Label
	sourceID  glib.SourceHandle
}

// NewClock creates a new clock widget
func NewClock() *Clock {
	c := &Clock{
		Box:       gtk.NewBox(gtk.OrientationHorizontal, 8),
		timeLabel: gtk.NewLabel(""),
		dateLabel: gtk.NewLabel(""),
	}

	c.Box.AddCSSClass("clock")
	c.timeLabel.AddCSSClass("clock-time")
	c.dateLabel.AddCSSClass("clock-date")

	// Elder Futhark runes for time:
	// ᛃ (Jera) - Year/harvest, cycles, seasons - perfect for time
	// ᛞ (Dagaz) - Day, transformation, balance of light/dark
	runeLeft := gtk.NewLabel("\u16C3") // ᛃ Jera - cycles of time
	runeLeft.AddCSSClass("clock-rune")

	runeRight := gtk.NewLabel("\u16DE") // ᛞ Dagaz - day
	runeRight.AddCSSClass("clock-rune")

	c.Box.Append(runeLeft)
	c.Box.Append(c.dateLabel)
	c.Box.Append(c.timeLabel)
	c.Box.Append(runeRight)

	// Initial update
	c.Refresh()

	// Start periodic updates
	c.startUpdates()

	return c
}

// startUpdates begins periodic clock updates
func (c *Clock) startUpdates() {
	c.sourceID = glib.TimeoutAdd(ClockUpdateInterval*1000, func() bool {
		c.Refresh()
		return true
	})
}

// Stop stops the periodic updates
func (c *Clock) Stop() {
	if c.sourceID > 0 {
		glib.SourceRemove(c.sourceID)
		c.sourceID = 0
	}
}

// Refresh updates the clock display
func (c *Clock) Refresh() {
	now := time.Now()

	// Time format: 24-hour
	timeStr := now.Format("15:04")
	c.timeLabel.SetText(timeStr)

	// Date format: Day, Month Date
	dateStr := now.Format("Mon, Jan 2")
	c.dateLabel.SetText(dateStr)

	// Tooltip with full date and week info
	_, week := now.ISOWeek()
	tooltip := now.Format("Monday, January 2, 2006") + "\n" +
		now.Format("Week ") + fmt.Sprintf("%d", week)
	c.Box.SetTooltipText(tooltip)
}
