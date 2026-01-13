package systray

import (
	"fmt"
	"os"
	"strings"
	"sync"

	"github.com/godbus/dbus/v5"
)

const (
	watcherService   = "org.kde.StatusNotifierWatcher"
	watcherPath      = "/StatusNotifierWatcher"
	watcherInterface = "org.kde.StatusNotifierWatcher"
	itemInterface    = "org.kde.StatusNotifierItem"
)

// Item represents a system tray item
type Item struct {
	Service     string // D-Bus service name
	Path        string // D-Bus object path
	IconName    string // Icon name
	Title       string // Item title
	ToolTip     string // Tooltip text
	Category    string // Item category
	IsAvailable bool   // Whether item is accessible
}

// Host manages the system tray connection
type Host struct {
	conn       *dbus.Conn
	items      map[string]*Item
	itemsMu    sync.RWMutex
	hostName   string
	registered bool
	onChange   func()
}

// NewHost creates a new system tray host
func NewHost(onChange func()) (*Host, error) {
	conn, err := dbus.ConnectSessionBus()
	if err != nil {
		return nil, fmt.Errorf("failed to connect to session bus: %w", err)
	}

	host := &Host{
		conn:     conn,
		items:    make(map[string]*Item),
		hostName: fmt.Sprintf("org.kde.StatusNotifierHost-%d-%d", os.Getpid(), 0),
		onChange: onChange,
	}

	return host, nil
}

// Start begins listening for tray items
func (h *Host) Start() error {
	// Request our host name on the bus
	reply, err := h.conn.RequestName(h.hostName, dbus.NameFlagDoNotQueue)
	if err != nil {
		return fmt.Errorf("failed to request name: %w", err)
	}
	if reply != dbus.RequestNameReplyPrimaryOwner {
		return fmt.Errorf("name already taken")
	}

	// Register as a StatusNotifierHost
	obj := h.conn.Object(watcherService, watcherPath)
	call := obj.Call(watcherInterface+".RegisterStatusNotifierHost", 0, h.hostName)
	if call.Err != nil {
		// Watcher might not be running - that's OK
		return nil
	}
	h.registered = true

	// Subscribe to item registration signals
	if err := h.conn.AddMatchSignal(
		dbus.WithMatchInterface(watcherInterface),
		dbus.WithMatchMember("StatusNotifierItemRegistered"),
	); err != nil {
		return fmt.Errorf("failed to add match: %w", err)
	}

	if err := h.conn.AddMatchSignal(
		dbus.WithMatchInterface(watcherInterface),
		dbus.WithMatchMember("StatusNotifierItemUnregistered"),
	); err != nil {
		return fmt.Errorf("failed to add match: %w", err)
	}

	// Get currently registered items
	h.refreshItems()

	// Start signal handler
	go h.handleSignals()

	return nil
}

// handleSignals processes D-Bus signals
func (h *Host) handleSignals() {
	signals := make(chan *dbus.Signal, 10)
	h.conn.Signal(signals)

	for sig := range signals {
		switch sig.Name {
		case watcherInterface + ".StatusNotifierItemRegistered":
			if len(sig.Body) > 0 {
				if service, ok := sig.Body[0].(string); ok {
					h.addItem(service)
				}
			}
		case watcherInterface + ".StatusNotifierItemUnregistered":
			if len(sig.Body) > 0 {
				if service, ok := sig.Body[0].(string); ok {
					h.removeItem(service)
				}
			}
		}
	}
}

// refreshItems gets the current list of registered items
func (h *Host) refreshItems() {
	obj := h.conn.Object(watcherService, watcherPath)
	variant, err := obj.GetProperty(watcherInterface + ".RegisteredStatusNotifierItems")
	if err != nil {
		return
	}

	items, ok := variant.Value().([]string)
	if !ok {
		return
	}

	for _, service := range items {
		h.addItem(service)
	}
}

// addItem adds a new tray item
func (h *Host) addItem(service string) {
	// Parse service string - can be "service" or "service/path"
	parts := strings.SplitN(service, "/", 2)
	serviceName := parts[0]
	path := "/StatusNotifierItem"
	if len(parts) > 1 {
		path = "/" + parts[1]
	}

	item := &Item{
		Service:     serviceName,
		Path:        path,
		IsAvailable: true,
	}

	// Get item properties
	h.fetchItemProperties(item)

	h.itemsMu.Lock()
	h.items[service] = item
	h.itemsMu.Unlock()

	if h.onChange != nil {
		h.onChange()
	}
}

// removeItem removes a tray item
func (h *Host) removeItem(service string) {
	h.itemsMu.Lock()
	delete(h.items, service)
	h.itemsMu.Unlock()

	if h.onChange != nil {
		h.onChange()
	}
}

// fetchItemProperties gets properties for an item
func (h *Host) fetchItemProperties(item *Item) {
	obj := h.conn.Object(item.Service, dbus.ObjectPath(item.Path))

	// Get IconName
	if variant, err := obj.GetProperty(itemInterface + ".IconName"); err == nil {
		if name, ok := variant.Value().(string); ok {
			item.IconName = name
		}
	}

	// Get Title
	if variant, err := obj.GetProperty(itemInterface + ".Title"); err == nil {
		if title, ok := variant.Value().(string); ok {
			item.Title = title
		}
	}

	// Get Category
	if variant, err := obj.GetProperty(itemInterface + ".Category"); err == nil {
		if cat, ok := variant.Value().(string); ok {
			item.Category = cat
		}
	}

	// Try Id if Title is empty
	if item.Title == "" {
		if variant, err := obj.GetProperty(itemInterface + ".Id"); err == nil {
			if id, ok := variant.Value().(string); ok {
				item.Title = id
			}
		}
	}
}

// GetItems returns a copy of current tray items
func (h *Host) GetItems() []*Item {
	h.itemsMu.RLock()
	defer h.itemsMu.RUnlock()

	items := make([]*Item, 0, len(h.items))
	for _, item := range h.items {
		items = append(items, item)
	}
	return items
}

// ItemCount returns the number of tray items
func (h *Host) ItemCount() int {
	h.itemsMu.RLock()
	defer h.itemsMu.RUnlock()
	return len(h.items)
}

// Stop closes the D-Bus connection
func (h *Host) Stop() {
	if h.conn != nil {
		h.conn.Close()
	}
}

// IsAvailable returns whether the system tray service is available
func (h *Host) IsAvailable() bool {
	return h.registered
}
