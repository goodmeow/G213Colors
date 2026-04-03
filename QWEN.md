# G213Colors Project Context

## Project Overview

**G213Colors** is a Linux application for controlling LED lighting effects on Logitech G213 Prodigy Gaming Keyboards and G203 Prodigy Gaming Mice. It provides both a GTK3 GUI for interactive color/effect selection and CLI capabilities for automated configuration.

### Key Features
- **Effects**: Static colors, breathing effects, and color cycling
- **Segment Control**: G213 supports individual keyboard segment color control (5 zones)
- **Per-User Settings**: Each user can save their own configuration
- **System Service**: Applies default colors at system boot via systemd
- **Login Autostart**: Automatically applies user preferences on desktop login
- **No Root Required**: GUI runs without sudo privileges (uses udev rules for permissions)

### Supported Devices
- Logitech G213 Prodigy Gaming Keyboard (USB ID: `046d:c336`)
- Logitech G203 Prodigy Gaming Mouse (USB ID: `046d:c084`)

## Architecture

The project is structured into three main Python modules:

### Core Modules

1. **G213Colors.py** - Core USB device control module
   - `LogitechDevice` class for USB communication
   - Device detection and connection management with retry mechanism
   - USB HID command formatting and transmission
   - Product specifications for G213 and G203

2. **config_manager.py** - Configuration management module
   - User config file handling (`~/.config/G213Colors/*.conf`)
   - Autostart entry management (`~/.config/autostart/`)
   - System-wide config application (`/etc/G213Colors.conf`)

3. **main.py** - GTK3 GUI application and CLI entry point
   - Command-line argument parsing (`-t` for system default, `-auc` for user config)
   - Multi-tab GUI with Stack/Switcher for different effects
   - Color picker integration
   - Per-product and "Set All" functionality

### Supporting Files

- **INSTALL.sh** - Automated installation script with package manager detection (pacman/apt)
- **makefile** - Build/install system using standard `install` commands
- **g213colors.service** - systemd service for boot-time color application
- **G213Colors.desktop** - Desktop entry for application menu integration
- **be.jeroened.pkexec.g213colors.policy** - Polkit policy for privileged operations
- **icons/** - Application icons in multiple sizes (16, 24, 32, 48, 128, 192px)

### Configuration File Format

Config files use a simple text format:
```
PRODUCT=<DEVICE_NAME>
<HEX_COMMAND_1>
<HEX_COMMAND_2>
...
```

Example:
```
PRODUCT=G213
11ff0c3a0001ffffff0200000000000000000000
```

## Technology Stack

- **Language**: Python 3
- **GUI Framework**: GTK3 via PyGObject (`gi.repository.Gtk`)
- **USB Communication**: `pyusb` library (direct USB HID control)
- **System Integration**: systemd, udev, Polkit
- **Package Management**: Supports both pacman (Arch) and apt (Debian/Ubuntu)

## Building and Running

### Installation

```bash
# Clone repository
git clone https://github.com/nickth76/G213Colors.git
cd G213Colors

# Run installer (requires sudo)
sudo ./INSTALL.sh
```

The installer will:
1. Detect package manager (pacman or apt)
2. Install dependencies (python-pyusb, gtk3, python-gobject, etc.)
3. Create udev rules for USB device permissions
4. Install application files to `/usr/bin/`
5. Install systemd service and desktop entry
6. Create default system config at `/etc/G213Colors.conf`

### Running the Application

**GUI Mode** (no sudo required):
```bash
g213colors-gui
```

**CLI Mode**:
```bash
# Apply system-wide default config
sudo g213colors-gui -t

# Apply user-specific config for a product
g213colors-gui --apply-user-config G213
g213colors-gui --apply-user-config G203
```

**Enable System Service** (auto-starts on boot):
```bash
sudo systemctl enable g213colors.service
sudo systemctl start g213colors.service
```

### Uninstallation

```bash
sudo make uninstall
```

Note: This does not remove user config files in `~/.config/G213Colors/` or autostart entries in `~/.config/autostart/`.

## Development Conventions

### Code Style
- **Type Hints**: All functions use Python type hints (`str`, `bool`, `Optional[T]`, etc.)
- **Docstrings**: All classes and functions have comprehensive docstrings with Args/Returns sections
- **Logging**: Uses Python's `logging` module with module-specific loggers
- **Error Handling**: Explicit exception handling with appropriate logging at all levels
- **Modern Python**: Uses `pathlib.Path` for path operations, f-strings for formatting

### Module Responsibilities
- **G213Colors.py**: Pure USB device control, no GUI dependencies
- **config_manager.py**: File I/O and configuration logic, reusable by both CLI and GUI
- **main.py**: GTK GUI and CLI argument handling

### USB Communication Pattern
1. Find device by vendor/product ID
2. Detach kernel driver (if active)
3. Send control transfer with hex command data
4. For G213: Read response data after color commands
5. Reattach kernel driver on disconnect

### Error Recovery
- Connection retries (up to 2 retries by default)
- Graceful fallback if device not found
- Clear error messages for permission issues
- Safe disconnect with kernel driver reattachment

## File Locations

### System-Wide (Installed)
- `/usr/bin/G213Colors.py` - Core device module
- `/usr/bin/g213colors-gui` - GUI application (main.py)
- `/usr/bin/config_manager.py` - Configuration module
- `/etc/G213Colors.conf` - System default configuration
- `/etc/systemd/system/g213colors.service` - Systemd service
- `/etc/udev/rules.d/99-logitech-usb-permissions.rules` - USB permissions
- `/usr/share/applications/g213colors.desktop` - Desktop entry
- `/usr/share/icons/hicolor/*/apps/g213colors.png` - Application icons
- `/usr/share/polkit-1/actions/be.jeroened.pkexec.g213colors.policy` - Polkit policy

### User-Specific (Created at Runtime)
- `~/.config/G213Colors/G213.conf` - User's G213 configuration
- `~/.config/G213Colors/G203.conf` - User's G203 configuration
- `~/.config/autostart/g213colors-autostart-*.desktop` - Login autostart entries

## Configuration Hierarchy

Settings are applied in this order:
1. **System Boot**: `g213colors.service` applies `/etc/G213Colors.conf` (default white for G213)
2. **User Login**: Autostart entries apply user's saved preferences from `~/.config/G213Colors/`

This allows system-wide defaults at boot time with per-user customization on login.

## Limitations

- **Wave Effect**: Not implemented (requires rapid software updates that conflict with kernel driver detachment)
- **Multimedia Keys**: May be affected while kernel driver is detached (temporary issue)
- **G203 Segments**: G203 doesn't support segment control; segment mode falls back to static color

## Testing

Manual testing approach:
```bash
# Test device detection
python3 -c "import G213Colors; print(G213Colors.LogitechDevice.detect_connected_devices())"

# Test connection
python3 -c "import G213Colors; d = G213Colors.LogitechDevice('G213'); print(d.connect()); d.disconnect()"

# Syntax check all modules
python3 -m py_compile G213Colors.py config_manager.py main.py
```

## Dependencies

### Runtime (Arch Linux package names)
- `python` - Python 3 interpreter
- `python-pyusb` - USB communication library
- `python-gobject` - GTK bindings for Python
- `gtk3` - GUI toolkit
- `python-cairo` - Cairo graphics (for GTK)
- `pango` - Text rendering (for GTK)

### Build/Install
- `pacman` or `apt-get` - Package manager
- `make` - For install/uninstall operations
- `systemd` - Service management
- `udev` - Device permissions

## License

MIT License - Original work Copyright (c) 2016 SebiTimeWaster
