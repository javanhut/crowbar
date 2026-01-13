package hyprland

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"strings"
)

// EventType represents a Hyprland event
type EventType string

const (
	EventWorkspace       EventType = "workspace"
	EventWorkspaceV2     EventType = "workspacev2"
	EventActiveWindow    EventType = "activewindow"
	EventActiveWindowV2  EventType = "activewindowv2"
	EventOpenWindow      EventType = "openwindow"
	EventCloseWindow     EventType = "closewindow"
	EventWindowTitle     EventType = "windowtitle"
	EventCreateWorkspace EventType = "createworkspace"
	EventDestroyWorkspace EventType = "destroyworkspace"
)

// Event represents a Hyprland event
type Event struct {
	Type EventType
	Data string
}

// EventHandler is called when an event is received
type EventHandler func(Event)

// EventListener listens for Hyprland events
type EventListener struct {
	conn     net.Conn
	handlers map[EventType][]EventHandler
	running  bool
}

// NewEventListener creates a new event listener
func NewEventListener() (*EventListener, error) {
	signature := os.Getenv("HYPRLAND_INSTANCE_SIGNATURE")
	if signature == "" {
		return nil, fmt.Errorf("HYPRLAND_INSTANCE_SIGNATURE not set")
	}

	socketPath := fmt.Sprintf("%s/hypr/%s/.socket2.sock",
		os.Getenv("XDG_RUNTIME_DIR"), signature)

	conn, err := net.Dial("unix", socketPath)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to event socket: %w", err)
	}

	return &EventListener{
		conn:     conn,
		handlers: make(map[EventType][]EventHandler),
		running:  false,
	}, nil
}

// On registers a handler for an event type
func (el *EventListener) On(eventType EventType, handler EventHandler) {
	el.handlers[eventType] = append(el.handlers[eventType], handler)
}

// Start begins listening for events in a goroutine
func (el *EventListener) Start() {
	if el.running {
		return
	}
	el.running = true

	go func() {
		reader := bufio.NewReader(el.conn)
		for el.running {
			line, err := reader.ReadString('\n')
			if err != nil {
				if el.running {
					// Connection error while still running
					el.running = false
				}
				return
			}

			line = strings.TrimSpace(line)
			parts := strings.SplitN(line, ">>", 2)
			if len(parts) != 2 {
				continue
			}

			event := Event{
				Type: EventType(parts[0]),
				Data: parts[1],
			}

			if handlers, ok := el.handlers[event.Type]; ok {
				for _, h := range handlers {
					h(event)
				}
			}
		}
	}()
}

// Stop stops listening for events
func (el *EventListener) Stop() {
	el.running = false
	if el.conn != nil {
		el.conn.Close()
	}
}
