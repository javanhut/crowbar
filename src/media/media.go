package media

import (
	"os/exec"
	"strconv"
	"strings"
)

// PlaybackStatus represents the current playback state
type PlaybackStatus string

const (
	StatusPlaying PlaybackStatus = "Playing"
	StatusPaused  PlaybackStatus = "Paused"
	StatusStopped PlaybackStatus = "Stopped"
	StatusUnknown PlaybackStatus = "Unknown"
)

// MediaInfo contains information about the currently playing media
type MediaInfo struct {
	Available bool           // Whether a media player is available
	Status    PlaybackStatus // Current playback status
	Title     string         // Track title
	Artist    string         // Artist name
	Album     string         // Album name
	Player    string         // Player name (e.g., "spotify", "firefox")
	Position  int64          // Current position in microseconds
	Length    int64          // Track length in microseconds
}

// GetMediaInfo returns information about the currently playing media
func GetMediaInfo() *MediaInfo {
	info := &MediaInfo{
		Status: StatusUnknown,
	}

	// Check if playerctl is available
	_, err := exec.LookPath("playerctl")
	if err != nil {
		return info
	}

	// Get player name
	out, err := exec.Command("playerctl", "-l").Output()
	if err != nil || strings.TrimSpace(string(out)) == "" {
		return info
	}

	// Get first available player
	players := strings.Split(strings.TrimSpace(string(out)), "\n")
	if len(players) > 0 {
		info.Player = players[0]
		info.Available = true
	} else {
		return info
	}

	// Get playback status
	out, err = exec.Command("playerctl", "status").Output()
	if err == nil {
		status := strings.TrimSpace(string(out))
		switch status {
		case "Playing":
			info.Status = StatusPlaying
		case "Paused":
			info.Status = StatusPaused
		case "Stopped":
			info.Status = StatusStopped
		default:
			info.Status = StatusUnknown
		}
	}

	// Get metadata
	out, err = exec.Command("playerctl", "metadata", "--format", "{{title}}|||{{artist}}|||{{album}}").Output()
	if err == nil {
		parts := strings.Split(strings.TrimSpace(string(out)), "|||")
		if len(parts) >= 1 {
			info.Title = parts[0]
		}
		if len(parts) >= 2 {
			info.Artist = parts[1]
		}
		if len(parts) >= 3 {
			info.Album = parts[2]
		}
	}

	// Get position
	out, err = exec.Command("playerctl", "position").Output()
	if err == nil {
		if pos, err := strconv.ParseFloat(strings.TrimSpace(string(out)), 64); err == nil {
			info.Position = int64(pos * 1000000) // Convert to microseconds
		}
	}

	// Get length
	out, err = exec.Command("playerctl", "metadata", "mpris:length").Output()
	if err == nil {
		if length, err := strconv.ParseInt(strings.TrimSpace(string(out)), 10, 64); err == nil {
			info.Length = length
		}
	}

	return info
}

// PlayPause toggles play/pause
func PlayPause() error {
	return exec.Command("playerctl", "play-pause").Run()
}

// Play starts playback
func Play() error {
	return exec.Command("playerctl", "play").Run()
}

// Pause pauses playback
func Pause() error {
	return exec.Command("playerctl", "pause").Run()
}

// Next skips to next track
func Next() error {
	return exec.Command("playerctl", "next").Run()
}

// Previous goes to previous track
func Previous() error {
	return exec.Command("playerctl", "previous").Run()
}

// Stop stops playback
func Stop() error {
	return exec.Command("playerctl", "stop").Run()
}

// SetPosition sets the playback position (in seconds)
func SetPosition(seconds float64) error {
	return exec.Command("playerctl", "position", strconv.FormatFloat(seconds, 'f', 2, 64)).Run()
}

// FormatDuration formats microseconds to MM:SS
func FormatDuration(microseconds int64) string {
	seconds := microseconds / 1000000
	minutes := seconds / 60
	secs := seconds % 60
	return strconv.FormatInt(minutes, 10) + ":" + padZero(secs)
}

func padZero(n int64) string {
	if n < 10 {
		return "0" + strconv.FormatInt(n, 10)
	}
	return strconv.FormatInt(n, 10)
}

// GetStatusIcon returns an icon name for the playback status
func GetStatusIcon(status PlaybackStatus) string {
	switch status {
	case StatusPlaying:
		return "media-playback-start-symbolic"
	case StatusPaused:
		return "media-playback-pause-symbolic"
	default:
		return "media-playback-stop-symbolic"
	}
}

// TruncateString truncates a string to maxLen characters with ellipsis
func TruncateString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	if maxLen <= 3 {
		return s[:maxLen]
	}
	return s[:maxLen-3] + "..."
}
