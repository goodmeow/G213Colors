PREFIX ?= /usr
BINDIR = $(PREFIX)/bin
SYSCONFDIR = /etc
SYSTEMDDIR = /etc/systemd/system
ICONDIR = /usr/share/icons/hicolor
APPDIR = /usr/share/applications
POLKITDIR = /usr/share/polkit-1/actions

UDEVDIR = /etc/udev/rules.d

.PHONY: install uninstall

install:
	install -Dm755 G213Colors.py $(BINDIR)/G213Colors.py
	install -Dm755 main.py $(BINDIR)/g213colors-gui
	install -Dm644 config_manager.py $(BINDIR)/config_manager.py
	install -Dm644 g213colors.service $(SYSTEMDDIR)/g213colors.service
	install -Dm644 icons/G213Colors-16.png $(ICONDIR)/16x16/apps/g213colors.png
	install -Dm644 icons/G213Colors-24.png $(ICONDIR)/24x24/apps/g213colors.png
	install -Dm644 icons/G213Colors-32.png $(ICONDIR)/32x32/apps/g213colors.png
	install -Dm644 icons/G213Colors-48.png $(ICONDIR)/48x48/apps/g213colors.png
	install -Dm644 icons/G213Colors-128.png $(ICONDIR)/128x128/apps/g213colors.png
	install -Dm644 icons/G213Colors-192.png $(ICONDIR)/192x192/apps/g213colors.png
	install -Dm644 G213Colors.desktop $(APPDIR)/g213colors.desktop
	install -Dm644 be.jeroened.pkexec.g213colors.policy $(POLKITDIR)/be.jeroened.pkexec.g213colors.policy
	-gtk-update-icon-cache -q $(ICONDIR)/ 2>/dev/null || true
	-systemctl daemon-reload 2>/dev/null || true

uninstall:
	-rm -f $(BINDIR)/G213Colors.py
	-rm -f $(BINDIR)/g213colors-gui
	-rm -f $(BINDIR)/config_manager.py
	-rm -f $(SYSCONFDIR)/G213Colors.conf
	-rm -f $(SYSTEMDDIR)/g213colors.service
	-rm -f $(UDEVDIR)/99-logitech-usb-permissions.rules
	-rm -f $(ICONDIR)/16x16/apps/g213colors.png
	-rm -f $(ICONDIR)/24x24/apps/g213colors.png
	-rm -f $(ICONDIR)/32x32/apps/g213colors.png
	-rm -f $(ICONDIR)/48x48/apps/g213colors.png
	-rm -f $(ICONDIR)/128x128/apps/g213colors.png
	-rm -f $(ICONDIR)/192x192/apps/g213colors.png
	-rm -f $(APPDIR)/g213colors.desktop
	-rm -f $(POLKITDIR)/be.jeroened.pkexec.g213colors.policy
	-gtk-update-icon-cache -q $(ICONDIR)/ 2>/dev/null || true
	-systemctl daemon-reload 2>/dev/null || true
	@echo "Uninstallation complete. Note: User config files in ~/.config/G213Colors/ were not removed."
