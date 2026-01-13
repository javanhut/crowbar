package audio

import (
	"bufio"
	"fmt"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
)

// Info contains audio state information
type Info struct {
	Volume    int    // Volume percentage (0-100)
	Muted     bool   // Whether audio is muted
	SinkName  string // Name of the default sink
	Available bool   // Whether audio system is available
}

// volumeRegex matches volume percentage from pactl output
var volumeRegex = regexp.MustCompile(`(\d+)%`)

// GetInfo returns current audio state
func GetInfo() *Info {
	info := &Info{
		Available: true,
	}

	// Get volume
	vol, err := getVolume()
	if err != nil {
		info.Available = false
		return info
	}
	info.Volume = vol

	// Get mute state
	info.Muted = getMuted()

	// Get sink name
	info.SinkName = getSinkName()

	return info
}

// getVolume reads the current volume percentage
func getVolume() (int, error) {
	cmd := exec.Command("pactl", "get-sink-volume", "@DEFAULT_SINK@")
	output, err := cmd.Output()
	if err != nil {
		return 0, err
	}

	// Parse volume from output like:
	// Volume: front-left: 55707 /  85% / -4.23 dB, ...
	matches := volumeRegex.FindStringSubmatch(string(output))
	if len(matches) < 2 {
		return 0, fmt.Errorf("could not parse volume")
	}

	vol, err := strconv.Atoi(matches[1])
	if err != nil {
		return 0, err
	}

	return vol, nil
}

// getMuted returns whether the default sink is muted
func getMuted() bool {
	cmd := exec.Command("pactl", "get-sink-mute", "@DEFAULT_SINK@")
	output, err := cmd.Output()
	if err != nil {
		return false
	}

	return strings.Contains(string(output), "yes")
}

// getSinkName returns the name of the default sink
func getSinkName() string {
	cmd := exec.Command("pactl", "get-default-sink")
	output, err := cmd.Output()
	if err != nil {
		return ""
	}
	return strings.TrimSpace(string(output))
}

// SetVolume sets the volume to a percentage (0-100)
func SetVolume(percent int) error {
	if percent < 0 {
		percent = 0
	}
	if percent > 150 { // Allow some overdrive
		percent = 150
	}

	cmd := exec.Command("pactl", "set-sink-volume", "@DEFAULT_SINK@", fmt.Sprintf("%d%%", percent))
	return cmd.Run()
}

// ToggleMute toggles the mute state
func ToggleMute() error {
	cmd := exec.Command("pactl", "set-sink-mute", "@DEFAULT_SINK@", "toggle")
	return cmd.Run()
}

// SetMute sets the mute state
func SetMute(muted bool) error {
	val := "0"
	if muted {
		val = "1"
	}
	cmd := exec.Command("pactl", "set-sink-mute", "@DEFAULT_SINK@", val)
	return cmd.Run()
}

// VolumeUp increases volume by a percentage
func VolumeUp(percent int) error {
	cmd := exec.Command("pactl", "set-sink-volume", "@DEFAULT_SINK@", fmt.Sprintf("+%d%%", percent))
	return cmd.Run()
}

// VolumeDown decreases volume by a percentage
func VolumeDown(percent int) error {
	cmd := exec.Command("pactl", "set-sink-volume", "@DEFAULT_SINK@", fmt.Sprintf("-%d%%", percent))
	return cmd.Run()
}

// EventCallback is called when an audio event occurs
type EventCallback func()

// EventListener listens for audio events via pactl subscribe
type EventListener struct {
	cmd      *exec.Cmd
	callback EventCallback
	running  bool
}

// NewEventListener creates a new audio event listener
func NewEventListener(callback EventCallback) *EventListener {
	return &EventListener{
		callback: callback,
		running:  false,
	}
}

// Start begins listening for audio events
func (el *EventListener) Start() error {
	if el.running {
		return nil
	}

	el.cmd = exec.Command("pactl", "subscribe")
	stdout, err := el.cmd.StdoutPipe()
	if err != nil {
		return err
	}

	if err := el.cmd.Start(); err != nil {
		return err
	}

	el.running = true

	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() && el.running {
			line := scanner.Text()
			// Filter for sink events (volume/mute changes)
			if strings.Contains(line, "sink") || strings.Contains(line, "server") {
				if el.callback != nil {
					el.callback()
				}
			}
		}
		el.running = false
	}()

	return nil
}

// Stop stops listening for audio events
func (el *EventListener) Stop() {
	el.running = false
	if el.cmd != nil && el.cmd.Process != nil {
		el.cmd.Process.Kill()
	}
}

// GetVolumeIcon returns the appropriate icon name for the volume level
func GetVolumeIcon(volume int, muted bool) string {
	if muted || volume == 0 {
		return "audio-volume-muted-symbolic"
	}
	if volume < 33 {
		return "audio-volume-low-symbolic"
	}
	if volume < 66 {
		return "audio-volume-medium-symbolic"
	}
	return "audio-volume-high-symbolic"
}
