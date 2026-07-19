# G213Colors

A Rust/Iced application to manage the illuminated key colors and effects on
Logitech G213 Prodigy Gaming Keyboards on Linux.

This project is based on the work of [JeroenED's G213Colors-gui](https://github.com/JeroenED/G213Colors-gui) and [SebiTimeWaster's G213Colors](https://github.com/SebiTimeWaster/G213Colors), updated and enhanced for modern Linux distributions.

## Features

* Control static colors, breathing effects, and cycle effects.
* Control individual G213 keyboard segments.
* Iced GUI for color selection and effect management.
* Settings are saved per user.
* System service to apply a default color scheme at system startup.
* Option to automatically apply the user's last saved G213 settings on desktop login.
* Device registry architecture for future Logitech G device contributions.

## Supported Devices

* Logitech G213 Prodigy Gaming Keyboard

The current Rust MVP intentionally supports G213 only. See
[`docs/adding-devices.md`](docs/adding-devices.md) for the extension path for
other Logitech G devices.

## Installation

The installation script automates the setup process, including installing dependencies, setting USB device permissions, and installing the application files.

1.  **Clone the repository:**
    ```
    git clone https://github.com/nickth76/G213Colors.git
    cd G213Colors
    ```

2.  **Run the installation script with sudo:**
    ```
    sudo ./INSTALL.sh
    ```
    This will install all dependencies (via pacman or apt), create udev rules, set up the systemd service, and install application files. After installation, you should find "G213 Colors" in your application menu.

    **Note:** If your Logitech devices were connected during installation, you might need to unplug and replug them, or reboot your computer, for the USB device permissions (udev rules) to take full effect.

### Manual Installation

If you prefer to install dependencies manually:

```bash
# Arch Linux
sudo pacman -S rust cargo pkgconf libusb libxkbcommon wayland fontconfig

# Debian/Ubuntu
sudo apt-get install rustc cargo build-essential pkg-config libusb-1.0-0 libusb-1.0-0-dev libxkbcommon-dev libwayland-dev libfontconfig1-dev

# Then run make install
sudo make install
```

## Usage

### GUI Mode (Recommended)
Launch from application menu or terminal:
```bash
g213colors
```
**No sudo required.** Settings are saved per-user in `~/.config/G213Colors/`.

**Features:**
- Scan for G213
- Choose effect: Static color, Cycle, Breathe, or Segments
- Click "Set G213" to apply and save
- Enable "Apply user settings on login" to auto-restore colors on desktop login

### CLI Mode
```bash
# Apply system-wide default config (requires sudo)
sudo g213colors -t

# Apply user config
g213colors --apply-user-config
g213colors --apply-user-config G213

# Developer hardware checks
g213colors detect
g213colors set-static ff0000
g213colors set-cycle 5000
g213colors set-breathe 00ff00 5000
g213colors set-segment 1 0000ff

# Show help
g213colors --help
```

### System Service
```bash
# Enable auto-start on boot
sudo systemctl enable g213colors.service

# Start immediately
sudo systemctl start g213colors.service

# Check status
systemctl status g213colors.service
```

## How it Works

There are two main ways color settings are applied:

### 1. GUI Application (User-Specific Settings & Login Autostart)

* Launch "G213 Colors" from your application menu. **It does not require `sudo` to run.**
* Configure your desired G213 colors and effects.
* When you apply settings by clicking "Set G213", they are saved to your user's personal configuration directory (`~/.config/G213Colors/G213.conf`). These settings are specific to your user account.

* **Applying Your Settings on Login:**
    * The GUI includes a checkbox: "Apply user settings on login".
    * If you check it, your last saved G213 configuration will be automatically applied when you log into your desktop session.
    * This works by creating a small startup file in your user's autostart directory (`~/.config/autostart/`).
    * This ensures your preferred colors are restored after the system's initial default (if any) is applied at boot.

### 2. System Startup Settings (System-wide Default via Service)

* The `INSTALL.sh` script sets up a systemd service (`g213colors.service`) that runs early during system startup.
* This service automatically applies a default color scheme to the **Logitech G213 keyboard**, setting it to a standard white color. This configuration is stored in `/etc/G213Colors.conf`.
**Order of Application:**
1.  On system boot, `g213colors.service` sets the G213 to the system default (e.g., white).
2.  When you log into your desktop, if you've enabled "Apply user settings on login" via the GUI, your saved G213 preference will be applied, overriding the system default for your session.

**Customizing System-Wide Startup Colors:**
If you wish to change the *system-wide default startup color* that the service applies (currently G213 white):
1.  You will need to edit the `/etc/G213Colors.conf` file with root privileges (e.g., `sudo nano /etc/G213Colors.conf`).
2.  The file format requires the first line to be `PRODUCT=<DEVICE_NAME>` (e.g., `PRODUCT=G213`) followed by the raw hex command string for the device on the next line.
    *(A future enhancement may allow setting this system default more easily via the GUI.)*

You can enable the system service to start on boot with:
```sudo systemctl enable g213colors.service```

You can also manually trigger the application of the system default settings by running:

```sudo /usr/bin/g213colors -t```

## Screenshots 

![Application in Apps menu](https://raw.githubusercontent.com/nickth76/G213Colors/refs/heads/master/screenshots/screenshot-3.png)
![Main GUI](https://raw.githubusercontent.com/nickth76/G213Colors/refs/heads/master/screenshots/screenshot-1.png)
![Color picker](https://raw.githubusercontent.com/nickth76/G213Colors/refs/heads/master/screenshots/screenshot-2.png)

## Limitations
The "Wave" color effect available in official Logitech software on other platforms is not replicated here. This effect is typically software-generated by rapidly updating colors, which would conflict with how this tool interacts with the device (by detaching the kernel driver for direct USB control, which can affect multimedia keys if done continuously). The effects provided (static, breathe, cycle) run directly on the device hardware.

## Uninstallation
To remove the application and all system-wide components:
```
sudo make uninstall
```

This will remove the application files, the system-wide configuration file (`/etc/G213Colors.conf`), the systemd service unit, the udev rule, desktop entry, and Polkit policy.

**Note on User Files:** The uninstallation command does not remove your personal configuration files (in `~/.config/G213Colors/`) or any autostart entries you created via the GUI (in `~/.config/autostart/`). You can remove these manually if desired, or simply uncheck the "Apply user settings on login" boxes in the GUI before uninstalling.
