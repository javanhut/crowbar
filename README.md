# CrowBar

A Nordic Aesir-themed status bar for Hyprland, forged in Go with GTK4.

## Features

- Workspace indicators with real-time updates
- Active window title display
- System tray (SNI/AppIndicator support)
- Audio volume control with slider popup
- Screen brightness control with slider popup
- CPU temperature and governor display
- Battery status with charging indicators
- Clock with date display
- Power menu (lock, logout, suspend, reboot, shutdown)
- Elder Futhark runes as visual indicators
- Nordic color theme inspired by the Nine Realms

## Dependencies

### Build Dependencies

- Go 1.21+
- GTK4 development libraries
- gtk4-layer-shell

### Arch Linux

```bash
sudo pacman -S go gtk4 gtk4-layer-shell
```

### Fedora

```bash
sudo dnf install golang gtk4-devel gtk4-layer-shell-devel
```

### Ubuntu/Debian

```bash
sudo apt install golang libgtk-4-dev libgtk4-layer-shell-dev
```

### Runtime Dependencies (Optional)

- `brightnessctl` - For brightness control without root permissions
- `pactl` (PulseAudio/PipeWire) - For audio control
- Hyprland - For workspace and window management features

## Installation

### Build from Source

```bash
git clone https://github.com/javanhut/crowbar.git
cd crowbar
make
```

### System-wide Installation

```bash
sudo make install
```

This installs:
- Binary to `/usr/local/bin/crowbar`
- Style to `/usr/local/share/crowbar/style.css`

### User Installation (no sudo)

```bash
make user-install
```

This installs:
- Binary to `~/.local/bin/crowbar`
- Style to `~/.local/share/crowbar/style.css`

Make sure `~/.local/bin` is in your PATH.

## Running on Hyprland

### Add to Hyprland Config

Add the following to your `~/.config/hypr/hyprland.conf`:

```bash
# Start CrowBar on launch
exec-once = crowbar
```

### Manual Start

You can also start CrowBar manually:

```bash
crowbar
```

### Replace Waybar

If you're currently using Waybar, comment out or remove the Waybar exec line:

```bash
# exec-once = waybar  # Disabled - using CrowBar instead
exec-once = crowbar
```

Then reload Hyprland:

```bash
hyprctl reload
```

Or log out and log back in.

## Configuration

### Custom Styling

Copy the default style to your config directory for customization:

```bash
mkdir -p ~/.config/crowbar
cp /usr/local/share/crowbar/style.css ~/.config/crowbar/style.css
```

CrowBar looks for `style.css` in the following locations (in order):
1. `./style.css` (current directory)
2. Executable directory
3. `~/.config/crowbar/style.css`
4. `/usr/local/share/crowbar/style.css`

### Color Scheme

The default theme uses colors from the Nine Realms:

| Realm | Color | Hex |
|-------|-------|-----|
| Ginnungagap (Void) | Deep black | `#0d0e14` |
| Bifrost (Bridge) | Blue | `#7aa2f7` |
| Bifrost (Bridge) | Cyan | `#7dcfff` |
| Muspelheim (Fire) | Orange | `#ff9e64` |
| Muspelheim (Fire) | Red | `#f7768e` |
| Yggdrasil (Tree) | Green | `#9ece6a` |
| Asgard (Gods) | Gold | `#e0af68` |

### Elder Futhark Runes

Each module displays a rune with symbolic meaning:

| Rune | Name | Module | Meaning |
|------|------|--------|---------|
| ᚠ | Fehu | Battery | Wealth, Energy |
| ᚢ | Uruz | CPU/Power | Strength, Power |
| ᚨ | Ansuz | Audio | Voice of Odin |
| ᛊ | Sowilo | Brightness | Sun, Light |
| ᛉ | Algiz | System Tray | Protection |
| ᛃ | Jera | Clock | Time, Cycles |
| ᛞ | Dagaz | Clock | Day |
| ᚦ | Thurisaz | Power Button | Protection |

## Uninstallation

### System-wide

```bash
sudo make uninstall
```

### User Installation

```bash
make user-uninstall
```

### Clean Build Artifacts

```bash
make clean
```

## Troubleshooting

### Bar not appearing as layer surface

Make sure you're running on Wayland with a compositor that supports `wlr-layer-shell` (Hyprland, Sway, etc.).

### Brightness control not working

Install `brightnessctl` and add your user to the `video` group:

```bash
sudo pacman -S brightnessctl  # Arch
sudo usermod -aG video $USER
```

Log out and back in for group changes to take effect.

### Audio control not working

Ensure PulseAudio or PipeWire is running and `pactl` is available:

```bash
pactl info
```

### Workspaces not showing

CrowBar requires Hyprland IPC. Make sure Hyprland is running and `HYPRLAND_INSTANCE_SIGNATURE` is set.

## License

MIT

## Credits

- [gotk4](https://github.com/diamondburned/gotk4) - GTK4 bindings for Go
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) - Layer shell protocol for GTK4
- [hyprland-go](https://github.com/thiagokokada/hyprland-go) - Hyprland IPC bindings
