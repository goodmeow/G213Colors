#!/bin/bash

set -e

echo "G213/G203 Colors - Installation Script"
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
        python \
        python-pip \
        python-pyusb \
        python-gobject \
        gtk3 \
        python-cairo \
        pango
    python3 -m pip install randomcolor --break-system-packages 2>/dev/null || \
    python3 -m pip install randomcolor

elif [ "$PKG_MANAGER" = "apt" ]; then
    echo "Installing dependencies via apt..."
    apt-get update -y
    apt-get install -y \
        libusb-1.0-0 \
        python3-pip \
        python3-gi \
        python3-gi-cairo \
        gir1.2-gtk-3.0 \
        python3-cairo \
        python3-usb
    python3 -m pip install randomcolor --break-system-packages 2>/dev/null || \
    python3 -m pip install randomcolor
fi

echo ""
echo "Running make install..."
make install

echo ""
echo "Note: USB device permissions now use logind session tracking (TAG+=uaccess)."
echo "This means only the currently logged-in user can access the device — more secure than world-readable."

echo ""
echo "Installation complete!"
echo "If your Logitech devices were already connected, you might need to unplug/replug them or reboot for udev rules to take effect."
echo "Launch 'G213 Colors' from your application menu or run 'g213colors-gui' from terminal."
