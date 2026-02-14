# CrowBar Makefile
# Nordic Aesir Status Bar for Hyprland (Rust Edition)

BINARY_NAME := crowbar
VERSION := 0.1.0

# Installation directories
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share/$(BINARY_NAME)

.PHONY: all build clean install uninstall user-install user-uninstall run help debug test

all: build

## build: Compile the binary (release mode)
build:
	cargo build --release

## debug: Compile the binary (debug mode)
debug:
	cargo build

## clean: Remove build artifacts
clean:
	cargo clean

## test: Run tests
test:
	cargo test

## install: Install crowbar to system (requires sudo)
install: build
	@echo "Installing $(BINARY_NAME) to $(BINDIR)..."
	install -Dm755 target/release/$(BINARY_NAME) $(DESTDIR)$(BINDIR)/$(BINARY_NAME)
	@echo "Installing style.css to $(DATADIR)..."
	install -Dm644 style.css $(DESTDIR)$(DATADIR)/style.css
	@echo "Installing config.toml.example to $(DATADIR)..."
	install -Dm644 config.toml.example $(DESTDIR)$(DATADIR)/config.toml.example
	@echo ""
	@echo "Installation complete!"
	@echo "  Binary: $(DESTDIR)$(BINDIR)/$(BINARY_NAME)"
	@echo "  Style:  $(DESTDIR)$(DATADIR)/style.css"
	@echo "  Config: $(DESTDIR)$(DATADIR)/config.toml.example"
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
	install -Dm755 target/release/$(BINARY_NAME) $(HOME)/.local/bin/$(BINARY_NAME)
	@echo "Installing style.css to ~/.local/share/$(BINARY_NAME)..."
	install -Dm644 style.css $(HOME)/.local/share/$(BINARY_NAME)/style.css
	@echo "Installing config.toml.example to ~/.local/share/$(BINARY_NAME)..."
	install -Dm644 config.toml.example $(HOME)/.local/share/$(BINARY_NAME)/config.toml.example
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

## run: Build and run locally (debug mode)
run: debug
	./target/debug/$(BINARY_NAME)

## help: Show this help message
help:
	@echo "CrowBar - Nordic Aesir Status Bar for Hyprland (Rust Edition)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@sed -n 's/^## //p' $(MAKEFILE_LIST) | column -t -s ':' | sed 's/^/  /'
	@echo ""
	@echo "Examples:"
	@echo "  make                  # Build release binary"
	@echo "  make debug            # Build debug binary"
	@echo "  make test             # Run tests"
	@echo "  sudo make install     # Install system-wide"
	@echo "  make user-install     # Install to ~/.local/bin (no sudo)"
