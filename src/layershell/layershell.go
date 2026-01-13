// Package layershell provides Go bindings for gtk4-layer-shell.
// This wraps the C library to work with gotk4 GTK4 bindings.
package layershell

/*
#cgo pkg-config: gtk4-layer-shell-0
#include <gtk4-layer-shell.h>
*/
import "C"
import (
	"unsafe"

	"github.com/diamondburned/gotk4/pkg/gtk/v4"
)

// Edge represents a screen edge
type Edge int

const (
	EdgeLeft   Edge = C.GTK_LAYER_SHELL_EDGE_LEFT
	EdgeRight  Edge = C.GTK_LAYER_SHELL_EDGE_RIGHT
	EdgeTop    Edge = C.GTK_LAYER_SHELL_EDGE_TOP
	EdgeBottom Edge = C.GTK_LAYER_SHELL_EDGE_BOTTOM
)

// Layer represents a layer shell layer
type Layer int

const (
	LayerBackground Layer = C.GTK_LAYER_SHELL_LAYER_BACKGROUND
	LayerBottom     Layer = C.GTK_LAYER_SHELL_LAYER_BOTTOM
	LayerTop        Layer = C.GTK_LAYER_SHELL_LAYER_TOP
	LayerOverlay    Layer = C.GTK_LAYER_SHELL_LAYER_OVERLAY
)

// KeyboardMode represents keyboard interactivity mode
type KeyboardMode int

const (
	KeyboardModeNone      KeyboardMode = C.GTK_LAYER_SHELL_KEYBOARD_MODE_NONE
	KeyboardModeExclusive KeyboardMode = C.GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE
	KeyboardModeOnDemand  KeyboardMode = C.GTK_LAYER_SHELL_KEYBOARD_MODE_ON_DEMAND
)

// nativeWindow extracts the C GtkWindow pointer from a gotk4 Window.
// Uses Object.Native() which returns uintptr to break Go pointer chain for CGO.
func nativeWindow(win *gtk.Window) uintptr {
	return win.Object.Native()
}

// IsSupported returns true if the platform supports layer shell
func IsSupported() bool {
	return C.gtk_layer_is_supported() != 0
}

// InitForWindow sets up a window to be a layer surface
func InitForWindow(win *gtk.Window) {
	ptr := nativeWindow(win)
	C.gtk_layer_init_for_window((*C.GtkWindow)(unsafe.Pointer(ptr)))
}

// SetLayer sets the layer on which the surface appears
func SetLayer(win *gtk.Window, layer Layer) {
	ptr := nativeWindow(win)
	C.gtk_layer_set_layer((*C.GtkWindow)(unsafe.Pointer(ptr)), C.GtkLayerShellLayer(layer))
}

// SetAnchor sets whether the window should be anchored to an edge
func SetAnchor(win *gtk.Window, edge Edge, anchor bool) {
	ptr := nativeWindow(win)
	var a C.gboolean
	if anchor {
		a = 1
	}
	C.gtk_layer_set_anchor((*C.GtkWindow)(unsafe.Pointer(ptr)), C.GtkLayerShellEdge(edge), a)
}

// SetMargin sets the margin from an edge
func SetMargin(win *gtk.Window, edge Edge, margin int) {
	ptr := nativeWindow(win)
	C.gtk_layer_set_margin((*C.GtkWindow)(unsafe.Pointer(ptr)), C.GtkLayerShellEdge(edge), C.int(margin))
}

// SetExclusiveZone sets the exclusive zone size
func SetExclusiveZone(win *gtk.Window, zone int) {
	ptr := nativeWindow(win)
	C.gtk_layer_set_exclusive_zone((*C.GtkWindow)(unsafe.Pointer(ptr)), C.int(zone))
}

// AutoExclusiveZoneEnable enables automatic exclusive zone sizing
func AutoExclusiveZoneEnable(win *gtk.Window) {
	ptr := nativeWindow(win)
	C.gtk_layer_auto_exclusive_zone_enable((*C.GtkWindow)(unsafe.Pointer(ptr)))
}

// SetKeyboardMode sets the keyboard interactivity mode
func SetKeyboardMode(win *gtk.Window, mode KeyboardMode) {
	ptr := nativeWindow(win)
	C.gtk_layer_set_keyboard_mode((*C.GtkWindow)(unsafe.Pointer(ptr)), C.GtkLayerShellKeyboardMode(mode))
}

// SetNamespace sets the namespace for the layer surface
func SetNamespace(win *gtk.Window, namespace string) {
	ptr := nativeWindow(win)
	cns := C.CString(namespace)
	defer C.free(unsafe.Pointer(cns))
	C.gtk_layer_set_namespace((*C.GtkWindow)(unsafe.Pointer(ptr)), cns)
}
