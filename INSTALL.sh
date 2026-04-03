#!/bin/bash

set -e  # Exit on error

echo "G213/G203 Colors - Installation Script"
echo "---------------------------------------"
echo "This script needs to be run with sudo privileges."

# Ensure the script is run as root/sudo
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

echo ""
echo "Using package manager: $PKG_MANAGER"

if [ "$PKG_MANAGER" = "pacman" ]; then
    echo "Updating package lists..."
    pacman -Sy --noconfirm

    echo ""
    echo "Installing system libraries and Python core components via pacman..."
    pacman -S --noconfirm --needed \
        libusb \
        python \
        python-pip \
        python-pyusb \
        python-gobject \
        gtk3 \
        python-cairo \
        pango

    echo ""
    echo "Installing Python library 'randomcolor' via pip3..."
    PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
    PYTHON_MAJOR=$(echo "$PYTHON_VERSION" | cut -d. -f1)
    PYTHON_MINOR=$(echo "$PYTHON_VERSION" | cut -d. -f2)

    if [ "$PYTHON_MAJOR" -ge 3 ] && [ "$PYTHON_MINOR" -ge 11 ]; then
        python3 -m pip install randomcolor --break-system-packages 2>/dev/null || \
        python3 -m pip install randomcolor
    else
        python3 -m pip install randomcolor
    fi

elif [ "$PKG_MANAGER" = "apt" ]; then
    echo "Updating package lists..."
    apt-get update -y

    echo ""
    echo "Installing system libraries and Python core components via apt..."
    apt-get install -y \
        libusb-1.0-0 \
        python3-pip \
        python3-gi \
        python3-gi-cairo \
        gir1.2-gtk-3.0 \
        python3-cairo \
        python3-usb

    echo ""
    echo "Installing Python library 'randomcolor' via pip3..."
    PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
    PYTHON_MAJOR=$(echo "$PYTHON_VERSION" | cut -d. -f1)
    PYTHON_MINOR=$(echo "$PYTHON_VERSION" | cut -d. -f2)

    if [ "$PYTHON_MAJOR" -ge 3 ] && [ "$PYTHON_MINOR" -ge 11 ]; then
        python3 -m pip install randomcolor --break-system-packages 2>/dev/null || \
        python3 -m pip install randomcolor
    else
        python3 -m pip install randomcolor
    fi
fi

echo ""
echo "Creating udev rule for Logitech device permissions..."
UDEV_RULE_CONTENT=$(cat <<EOF
# Logitech G213 Keyboard
SUBSYSTEM=="usb", ATTR{idVendor}=="046d", ATTR{idProduct}=="c336", MODE="0666"

# Logitech G203 Mouse
SUBSYSTEM=="usb", ATTR{idVendor}=="046d", ATTR{idProduct}=="c084", MODE="0666"
EOF
)
echo "$UDEV_RULE_CONTENT" | tee /etc/udev/rules.d/99-logitech-usb-permissions.rules > /dev/null
echo "Reloading udev rules..."
udevadm control --reload-rules
udevadm trigger
echo "udev rules reloaded."

echo ""
echo "Creating a default system-wide configuration file for startup (-t option)..."
# Default configuration: G213 keyboard set to 'standard white' (ffb4aa)
DEFAULT_G213_COMMAND="11ff0c3a$(printf '%02x' 0)01ffb4aa0200000000000000000000"

DEFAULT_SYSTEM_CONF_CONTENT=$(cat <<EOF
PRODUCT=G213
${DEFAULT_G213_COMMAND}
EOF
)
echo "$DEFAULT_SYSTEM_CONF_CONTENT" | tee /etc/G213Colors.conf > /dev/null
echo "Default system configuration created at /etc/G213Colors.conf for G213 (standard white)."

echo ""
echo "Running make install..."
if [ -f makefile ]; then
    make install
else
    echo "WARNING: Makefile not found. Skipping 'make install'." >&2
    echo "Please ensure G213Colors.py, main.py (as g213colors-gui), service files, icons, etc., are copied to their correct system locations and systemd is reloaded if necessary." >&2
    exit 1
fi

echo ""
echo "Installation script finished."
echo "If your Logitech devices were already connected, you might need to unplug/replug them or reboot for all changes (especially udev rules) to fully take effect."
echo "The system service (g213colors.service) should now apply a default white color to G213 on startup."
echo "You can customize your colors using 'g213colors-gui' (without sudo), which saves to your user's local configuration."
