#!/bin/bash

set -e

REQUIRED_RUST_VERSION="1.85.0"

echo "G213 Colors - Installation Script"
echo "---------------------------------------"

if [ "$(id -u)" -ne 0 ]; then
  echo "This script must be run as root. Please use sudo." >&2
  exit 1
fi

# Detect package manager
if command -v pacman &> /dev/null; then
    PKG_MANAGER="pacman"
elif command -v apt-get &> /dev/null; then
    PKG_MANAGER="apt"
else
    echo "ERROR: No supported package manager found (pacman or apt-get required)." >&2
    exit 1
fi

echo "Using package manager: $PKG_MANAGER"
echo ""

# Install dependencies
if [ "$PKG_MANAGER" = "pacman" ]; then
    echo "Installing dependencies via pacman..."
    pacman -Sy --noconfirm
    pacman -S --noconfirm --needed \
        libusb \
        rust \
        cargo \
        pkgconf \
        libxkbcommon \
        wayland \
        fontconfig

elif [ "$PKG_MANAGER" = "apt" ]; then
    echo "Installing dependencies via apt..."
    apt-get update -y
    apt-get install -y \
        libusb-1.0-0 \
        libusb-1.0-0-dev \
        rustc \
        cargo \
        build-essential \
        pkg-config \
        libxkbcommon-dev \
        libwayland-dev \
        libfontconfig1-dev
fi

check_rust_version() {
    current_version="$(rustc --version | awk '{print $2}')"
    oldest_version="$(printf '%s\n%s\n' "$REQUIRED_RUST_VERSION" "$current_version" | sort -V | head -n1)"
    if [ "$oldest_version" != "$REQUIRED_RUST_VERSION" ]; then
        echo "ERROR: Rust $REQUIRED_RUST_VERSION or newer is required, found $current_version." >&2
        echo "Install a current Rust toolchain with rustup, then rerun this installer." >&2
        exit 1
    fi
}

check_rust_version

echo ""
echo "Running make install..."
make install

echo ""
echo "Note: USB device permissions now use logind session tracking (TAG+=uaccess)."
echo "This means only the currently logged-in user can access the device — more secure than world-readable."

echo ""
echo "Installation complete!"
echo "If your Logitech devices were already connected, you might need to unplug/replug them or reboot for udev rules to take effect."
echo "Launch 'G213 Colors' from your application menu or run 'g213colors' from terminal."
