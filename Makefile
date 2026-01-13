# CrowBar Makefile
# Nordic Aesir Status Bar for Hyprland

BINARY_NAME := crowbar
VERSION := 0.1.0

# Installation directories
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share/$(BINARY_NAME)
SYSCONFDIR ?= /etc

# Build settings
GO := go
GOFLAGS := -ldflags="-s -w -X main.Version=$(VERSION)"

# Source files
SRC := $(shell find . -name '*.go' -type f)

.PHONY: all build clean install uninstall help

all: build

## build: Compile the binary
build: $(BINARY_NAME)

$(BINARY_NAME): $(SRC)
	$(GO) build $(GOFLAGS) -o $(BINARY_NAME)

## clean: Remove build artifacts
clean:
	rm -f $(BINARY_NAME)
	$(GO) clean

## install: Install crowbar to system (requires sudo)
install: build
	@echo "Installing $(BINARY_NAME) to $(BINDIR)..."
	install -Dm755 $(BINARY_NAME) $(DESTDIR)$(BINDIR)/$(BINARY_NAME)
	@echo "Installing style.css to $(DATADIR)..."
	install -Dm644 style.css $(DESTDIR)$(DATADIR)/style.css
	@echo ""
	@echo "Installation complete!"
	@echo "  Binary: $(DESTDIR)$(BINDIR)/$(BINARY_NAME)"
	@echo "  Style:  $(DESTDIR)$(DATADIR)/style.css"
	@echo ""
	@echo "You can also copy style.css to ~/.config/crowbar/style.css for customization"

## uninstall: Remove crowbar from system (requires sudo)
uninstall:
	@echo "Removing $(BINARY_NAME) from $(BINDIR)..."
	rm -f $(DESTDIR)$(BINDIR)/$(BINARY_NAME)
	@echo "Removing $(DATADIR)..."
	rm -rf $(DESTDIR)$(DATADIR)
	@echo ""
	@echo "Uninstall complete!"
	@echo "Note: User config at ~/.config/crowbar/ was not removed"

## user-install: Install to user's local bin (~/.local/bin)
user-install: build
	@echo "Installing $(BINARY_NAME) to ~/.local/bin..."
	install -Dm755 $(BINARY_NAME) $(HOME)/.local/bin/$(BINARY_NAME)
	@echo "Installing style.css to ~/.local/share/$(BINARY_NAME)..."
	install -Dm644 style.css $(HOME)/.local/share/$(BINARY_NAME)/style.css
	@echo ""
	@echo "User installation complete!"
	@echo "Make sure ~/.local/bin is in your PATH"

## user-uninstall: Remove from user's local bin
user-uninstall:
	@echo "Removing $(BINARY_NAME) from ~/.local/bin..."
	rm -f $(HOME)/.local/bin/$(BINARY_NAME)
	@echo "Removing ~/.local/share/$(BINARY_NAME)..."
	rm -rf $(HOME)/.local/share/$(BINARY_NAME)
	@echo ""
	@echo "User uninstall complete!"

## run: Build and run locally
run: build
	./$(BINARY_NAME)

## help: Show this help message
help:
	@echo "CrowBar - Nordic Aesir Status Bar for Hyprland"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@sed -n 's/^## //p' $(MAKEFILE_LIST) | column -t -s ':' | sed 's/^/  /'
	@echo ""
	@echo "Examples:"
	@echo "  make                  # Build the binary"
	@echo "  sudo make install     # Install system-wide"
	@echo "  sudo make uninstall   # Remove system-wide installation"
	@echo "  make user-install     # Install to ~/.local/bin (no sudo)"
	@echo "  make user-uninstall   # Remove from ~/.local/bin"
