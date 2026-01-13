package status

import (
	"fmt"

	"github.com/diamondburned/gotk4/pkg/glib/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
	"github.com/javanhut/crowbar/src/media"
)

const (
	// MediaUpdateInterval is how often to refresh media status (in seconds)
	MediaUpdateInterval = 2
)

// Media represents the media player control widget
type Media struct {
	Box        *gtk.Box
	menuButton *gtk.MenuButton
	statusIcon *gtk.Image
	titleLabel *gtk.Label
	popover    *gtk.Popover
	// Popover controls
	trackTitle   *gtk.Label
	trackArtist  *gtk.Label
	playPauseBtn *gtk.Button
	playPauseIcon *gtk.Image
	progressBar  *gtk.ProgressBar
	positionLabel *gtk.Label
	sourceID     glib.SourceHandle
	available    bool
}

// NewMedia creates a new media player widget
func NewMedia() *Media {
	m := &Media{
		Box: gtk.NewBox(gtk.OrientationHorizontal, 0),
	}

	m.Box.AddCSSClass("media")

	// Create menu button for popover
	m.menuButton = gtk.NewMenuButton()
	m.menuButton.AddCSSClass("media-button")
	m.menuButton.SetHasFrame(false)

	// ᛚ Laguz - Flow, water, music
	rune := gtk.NewLabel("\u16DA")
	rune.AddCSSClass("module-rune")

	// Create button content
	btnContent := gtk.NewBox(gtk.OrientationHorizontal, 6)
	btnContent.Append(rune)

	m.statusIcon = gtk.NewImageFromIconName("media-playback-stop-symbolic")
	m.statusIcon.AddCSSClass("media-status-icon")
	btnContent.Append(m.statusIcon)

	m.titleLabel = gtk.NewLabel("No media")
	m.titleLabel.AddCSSClass("media-title")
	m.titleLabel.SetMaxWidthChars(20)
	m.titleLabel.SetEllipsize(3) // PANGO_ELLIPSIZE_END
	btnContent.Append(m.titleLabel)

	m.menuButton.SetChild(btnContent)

	// Create popover with controls
	m.popover = gtk.NewPopover()
	m.popover.AddCSSClass("media-popover")
	m.popover.SetAutohide(true)

	popoverContent := m.createPopoverContent()
	m.popover.SetChild(popoverContent)
	m.menuButton.SetPopover(m.popover)

	m.Box.Append(m.menuButton)

	// Start hidden - will show when media is playing
	m.Box.SetVisible(false)
	m.available = false

	// Initial refresh
	m.Refresh()

	// Start periodic updates
	m.startUpdates()

	return m
}

// createPopoverContent creates the media control popover content
func (m *Media) createPopoverContent() *gtk.Box {
	content := gtk.NewBox(gtk.OrientationVertical, 8)
	content.SetMarginTop(12)
	content.SetMarginBottom(12)
	content.SetMarginStart(12)
	content.SetMarginEnd(12)
	content.AddCSSClass("media-popover-content")

	// Header with rune
	header := gtk.NewBox(gtk.OrientationHorizontal, 8)
	headerRune := gtk.NewLabel("\u16DA")
	headerRune.AddCSSClass("slider-header-rune")
	headerLabel := gtk.NewLabel("Skaldic Songs")
	headerLabel.AddCSSClass("slider-header-label")
	header.Append(headerRune)
	header.Append(headerLabel)
	content.Append(header)

	// Track info section
	trackInfo := gtk.NewBox(gtk.OrientationVertical, 4)
	trackInfo.AddCSSClass("media-track-info")

	m.trackTitle = gtk.NewLabel("No track playing")
	m.trackTitle.AddCSSClass("media-track-title")
	m.trackTitle.SetHAlign(gtk.AlignStart)
	m.trackTitle.SetMaxWidthChars(30)
	m.trackTitle.SetEllipsize(3)
	trackInfo.Append(m.trackTitle)

	m.trackArtist = gtk.NewLabel("")
	m.trackArtist.AddCSSClass("media-track-artist")
	m.trackArtist.SetHAlign(gtk.AlignStart)
	m.trackArtist.SetMaxWidthChars(30)
	m.trackArtist.SetEllipsize(3)
	trackInfo.Append(m.trackArtist)

	content.Append(trackInfo)

	// Progress bar
	progressBox := gtk.NewBox(gtk.OrientationVertical, 4)

	m.progressBar = gtk.NewProgressBar()
	m.progressBar.AddCSSClass("media-progress")
	m.progressBar.SetShowText(false)
	progressBox.Append(m.progressBar)

	m.positionLabel = gtk.NewLabel("0:00 / 0:00")
	m.positionLabel.AddCSSClass("media-position")
	progressBox.Append(m.positionLabel)

	content.Append(progressBox)

	// Playback controls
	controls := gtk.NewBox(gtk.OrientationHorizontal, 8)
	controls.SetHAlign(gtk.AlignCenter)
	controls.AddCSSClass("media-controls")

	// Previous button
	prevBtn := m.createControlButton("\u16B1", "Previous", func() {
		media.Previous()
		glib.TimeoutAdd(200, func() bool {
			m.Refresh()
			return false
		})
	})
	prevBtn.AddCSSClass("media-prev")
	controls.Append(prevBtn)

	// Play/Pause button
	m.playPauseBtn = gtk.NewButton()
	m.playPauseBtn.AddCSSClass("media-play-pause")

	playPauseBox := gtk.NewBox(gtk.OrientationHorizontal, 0)
	m.playPauseIcon = gtk.NewImageFromIconName("media-playback-start-symbolic")
	m.playPauseIcon.AddCSSClass("media-control-icon")
	playPauseBox.Append(m.playPauseIcon)
	m.playPauseBtn.SetChild(playPauseBox)

	m.playPauseBtn.ConnectClicked(func() {
		media.PlayPause()
		glib.TimeoutAdd(200, func() bool {
			m.Refresh()
			return false
		})
	})
	controls.Append(m.playPauseBtn)

	// Next button
	nextBtn := m.createControlButton("\u16A0", "Next", func() {
		media.Next()
		glib.TimeoutAdd(200, func() bool {
			m.Refresh()
			return false
		})
	})
	nextBtn.AddCSSClass("media-next")
	controls.Append(nextBtn)

	content.Append(controls)

	return content
}

// createControlButton creates a media control button with a rune
func (m *Media) createControlButton(runeChar, tooltip string, onClick func()) *gtk.Button {
	btn := gtk.NewButton()
	btn.AddCSSClass("media-control-btn")
	btn.SetTooltipText(tooltip)

	runeLabel := gtk.NewLabel(runeChar)
	runeLabel.AddCSSClass("media-control-rune")
	btn.SetChild(runeLabel)

	btn.ConnectClicked(func() {
		onClick()
	})

	return btn
}

// startUpdates begins periodic media status updates
func (m *Media) startUpdates() {
	m.sourceID = glib.TimeoutAdd(MediaUpdateInterval*1000, func() bool {
		m.Refresh()
		return true
	})
}

// Stop stops the periodic updates
func (m *Media) Stop() {
	if m.sourceID > 0 {
		glib.SourceRemove(m.sourceID)
		m.sourceID = 0
	}
}

// Refresh updates the media display
func (m *Media) Refresh() {
	info := media.GetMediaInfo()

	// Only show when actively playing or paused (not stopped or unavailable)
	shouldShow := info.Available &&
		(info.Status == media.StatusPlaying || info.Status == media.StatusPaused) &&
		info.Title != ""

	if !shouldShow {
		if m.available {
			m.available = false
			m.Box.SetVisible(false)
		}
		return
	}

	if !m.available {
		m.available = true
		m.Box.SetVisible(true)
	}

	// Update status icon
	iconName := media.GetStatusIcon(info.Status)
	m.statusIcon.SetFromIconName(iconName)

	// Update title label in bar
	if info.Title != "" {
		m.titleLabel.SetText(media.TruncateString(info.Title, 20))
	} else {
		m.titleLabel.SetText("Unknown")
	}

	// Update CSS class based on status
	m.Box.RemoveCSSClass("playing")
	m.Box.RemoveCSSClass("paused")
	if info.Status == media.StatusPlaying {
		m.Box.AddCSSClass("playing")
	} else if info.Status == media.StatusPaused {
		m.Box.AddCSSClass("paused")
	}

	// Update popover content
	m.updatePopover(info)

	// Update tooltip
	tooltip := fmt.Sprintf("%s\n%s", info.Title, info.Artist)
	if info.Player != "" {
		tooltip += fmt.Sprintf("\nPlayer: %s", info.Player)
	}
	m.Box.SetTooltipText(tooltip)
}

// updatePopover updates the popover content
func (m *Media) updatePopover(info *media.MediaInfo) {
	// Update track info
	if info.Title != "" {
		m.trackTitle.SetText(info.Title)
	} else {
		m.trackTitle.SetText("Unknown Track")
	}

	if info.Artist != "" {
		m.trackArtist.SetText(info.Artist)
		m.trackArtist.SetVisible(true)
	} else {
		m.trackArtist.SetVisible(false)
	}

	// Update play/pause button icon
	if info.Status == media.StatusPlaying {
		m.playPauseIcon.SetFromIconName("media-playback-pause-symbolic")
	} else {
		m.playPauseIcon.SetFromIconName("media-playback-start-symbolic")
	}

	// Update progress bar
	if info.Length > 0 {
		progress := float64(info.Position) / float64(info.Length)
		m.progressBar.SetFraction(progress)
		m.positionLabel.SetText(fmt.Sprintf("%s / %s",
			media.FormatDuration(info.Position),
			media.FormatDuration(info.Length)))
	} else {
		m.progressBar.SetFraction(0)
		m.positionLabel.SetText("0:00 / 0:00")
	}
}
