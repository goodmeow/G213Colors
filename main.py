#!/usr/bin/env python3

"""
Main entry point for G213Colors GUI and CLI tool.
Handles command-line arguments and launches the GUI application.
"""

import sys
import os
import logging
import argparse
import G213Colors
import config_manager
from time import sleep
import gi

gi.require_version('Gtk', '3.0')
from gi.repository import Gtk

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("g213colors_gui")


def parse_arguments():
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(description="G213 Colors GUI and CLI tool.")
    parser.add_argument(
        "-t", "--apply-system-default",
        action="store_true",
        help="Apply system-wide default configuration from /etc/G213Colors.conf and exit."
    )
    parser.add_argument(
        "-auc", "--apply-user-config",
        metavar="PRODUCT_NAME",
        type=str,
        choices=config_manager.SUPPORTED_PRODUCTS,
        help=f"Apply user-specific saved configuration for the given PRODUCT_NAME and exit. Choices: {config_manager.SUPPORTED_PRODUCTS}"
    )
    return parser.parse_args()


def handle_cli_actions(args):
    """
    Handle CLI actions. Exits the program if a CLI action is processed.

    Args:
        args: Parsed command-line arguments
    """
    if args.apply_system_default:
        logger.info("Option '-t' / '--apply-system-default' detected. Applying system default saved settings.")
        success = config_manager.apply_system_default_config()
        if success:
            logger.info("System default settings applied successfully via -t.")
            sys.exit(0)
        else:
            logger.error("Failed to apply system default settings via -t.")
            sys.exit(1)

    if args.apply_user_config:
        product_to_load = args.apply_user_config
        logger.info(f"Option '--apply-user-config' detected for product: {product_to_load}")
        success = config_manager.apply_user_config(product_to_load)
        if success:
            logger.info(f"User settings for {product_to_load} applied successfully.")
            sys.exit(0)
        else:
            logger.error(f"Failed to apply user settings for {product_to_load}.")
            sys.exit(1)


# --- GUI Application Class ---
class Window(Gtk.Window):
    """Main GUI window for G213Colors application."""

    def __init__(self):
        """Initialize the main window and build the UI."""
        Gtk.Window.__init__(self, title=config_manager.APP_NAME)
        self.set_border_width(10)

        # Ensure config directories exist
        if not config_manager.ensure_config_dirs():
            logger.error("Could not create configuration directories. Some features may not work correctly.")

        vBoxMain = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.add(vBoxMain)

        # --- STACK for different effect types ---
        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.SLIDE_LEFT_RIGHT)
        self.stack.set_transition_duration(1000)

        # --- STATIC TAB ---
        vBoxStatic = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
        self.staticColorButton = Gtk.ColorButton()
        vBoxStatic.pack_start(self.staticColorButton, True, True, 0)
        self.stack.add_titled(vBoxStatic, "static", "Static")

        # --- CYCLE TAB ---
        vBoxCycle = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
        self.adjCycle = Gtk.Adjustment(value=5000, lower=500, upper=65535, step_increment=100, page_increment=1000, page_size=0)
        self.sbCycle = Gtk.SpinButton()
        self.sbCycle.set_adjustment(self.adjCycle)
        self.sbCycle.set_numeric(True)
        vBoxCycle.pack_start(Gtk.Label(label="Speed (500-65535ms):"), False, False, 0)
        vBoxCycle.pack_start(self.sbCycle, False, False, 0)
        self.stack.add_titled(vBoxCycle, "cycle", "Cycle")

        # --- BREATHE TAB ---
        vBoxBreathe = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.breatheColorButton = Gtk.ColorButton()
        self.adjBCycle = Gtk.Adjustment(value=5000, lower=500, upper=65535, step_increment=100, page_increment=1000, page_size=0)
        self.sbBCycle = Gtk.SpinButton()
        self.sbBCycle.set_adjustment(self.adjBCycle)
        self.sbBCycle.set_numeric(True)
        vBoxBreathe.pack_start(Gtk.Label(label="Color:"), False, False, 0)
        vBoxBreathe.pack_start(self.breatheColorButton, False, False, 0)
        vBoxBreathe.pack_start(Gtk.Label(label="Speed (500-65535ms):"), False, False, 0)
        vBoxBreathe.pack_start(self.sbBCycle, False, False, 0)
        self.stack.add_titled(vBoxBreathe, "breathe", "Breathe")

        # --- SEGMENTS TAB (G213 specific ideally) ---
        hBoxSegments = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=5)
        self.segmentColorBtns = [Gtk.ColorButton() for _ in range(5)]
        for i, btn in enumerate(self.segmentColorBtns):
            segment_label_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
            segment_label_box.pack_start(Gtk.Label(label=f"Seg {i+1}"), False, False, 0)
            segment_label_box.pack_start(btn, True, True, 0)
            hBoxSegments.pack_start(segment_label_box, True, True, 0)
        self.stack.add_titled(hBoxSegments, "segments", "Segments (G213)")

        # --- Stack Switcher and Stack addition to main VBox ---
        self.stack_switcher = Gtk.StackSwitcher()
        self.stack_switcher.set_stack(self.stack)
        vBoxMain.pack_start(self.stack_switcher, False, False, 0)
        vBoxMain.pack_start(self.stack, True, True, 0)

        # --- SET Buttons per product ---
        hBoxSetButtons = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=5)
        for product in config_manager.SUPPORTED_PRODUCTS:
            btn = Gtk.Button.new_with_label(f"Set {product}")
            btn.connect("clicked", self.on_button_clicked, product)
            hBoxSetButtons.pack_start(btn, True, True, 0)
        vBoxMain.pack_start(hBoxSetButtons, False, False, 5)

        # --- SET ALL Button ---
        btnSetAll = Gtk.Button.new_with_label("Set all Products")
        btnSetAll.connect("clicked", self.on_button_clicked, "all")
        vBoxMain.pack_start(btnSetAll, False, False, 0)

        # --- Autostart Checkboxes Section ---
        vBoxMain.pack_start(Gtk.Separator(orientation=Gtk.Orientation.HORIZONTAL, margin_top=10, margin_bottom=5), False, False, 0)
        autostart_label = Gtk.Label(label="<b>Apply user settings on login:</b>", use_markup=True, xalign=0)
        vBoxMain.pack_start(autostart_label, False, False, 0)

        hBoxAutostartChecks = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10, halign=Gtk.Align.CENTER)
        self.autostart_checkboxes = {}
        for product in config_manager.SUPPORTED_PRODUCTS:
            checkbox = Gtk.CheckButton(label=product)
            checkbox.connect("toggled", self.on_autostart_toggled, product)
            if config_manager.is_autostart_enabled(product):
                checkbox.set_active(True)
            self.autostart_checkboxes[product] = checkbox
            hBoxAutostartChecks.pack_start(checkbox, False, False, 0)
        vBoxMain.pack_start(hBoxAutostartChecks, False, False, 5)

        # --- Device Status Bar ---
        self.status_bar = Gtk.Statusbar()
        self.status_bar.set_margin_top(5)
        vBoxMain.pack_start(self.status_bar, False, False, 0)

        # --- Refresh/Scan Button ---
        hBoxActions = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=5, margin_top=5)
        self.btn_refresh = Gtk.Button.new_with_label("⟳ Scan Devices")
        self.btn_refresh.connect("clicked", self.on_scan_devices)
        hBoxActions.pack_start(self.btn_refresh, False, False, 0)
        vBoxMain.pack_start(hBoxActions, False, False, 0)

        # Initial device scan
        self.on_scan_devices(self.btn_refresh)


    def btnGetHex(self, btn: Gtk.ColorButton) -> str:
        """Convert Gtk.ColorButton RGBA to hex color string."""
        color = btn.get_rgba()
        red = int(color.red * 255)
        green = int(color.green * 255)
        blue = int(color.blue * 255)
        return f"{red:02x}{green:02x}{blue:02x}"

    def _show_error_dialog(self, primary_text: str, secondary_text: str = ""):
        """Show an error dialog to the user."""
        dialog = Gtk.MessageDialog(
            transient_for=self,
            flags=0,
            message_type=Gtk.MessageType.ERROR,
            buttons=Gtk.ButtonsType.OK,
            text=primary_text,
        )
        if secondary_text:
            dialog.format_secondary_text(secondary_text)
        dialog.run()
        dialog.destroy()

    def on_autostart_toggled(self, checkbox: Gtk.CheckButton, product_name: str):
        """Handle autostart checkbox toggle."""
        if checkbox.get_active():
            success = config_manager.create_autostart_entry(product_name)
            if not success:
                self._show_error_dialog(f"Error creating autostart for {product_name}", "Check logs for details.")
                checkbox.set_active(False)
        else:
            success = config_manager.remove_autostart_entry(product_name)
            if not success:
                self._show_error_dialog(f"Error removing autostart for {product_name}", "Check logs for details.")
                checkbox.set_active(True)

    def on_scan_devices(self, button):
        """Scan for connected Logitech devices and update status."""
        self.btn_refresh.set_sensitive(False)
        try:
            detected = G213Colors.LogitechDevice.detect_connected_devices()
            if detected:
                device_names = ", ".join(detected.keys())
                status_msg = f"Devices found: {device_names}"
                logger.info(f"Detected devices: {device_names}")
            else:
                status_msg = "No Logitech G213/G203 devices detected"
                logger.warning("No supported Logitech devices found")

            context_id = self.status_bar.get_context_id("device-status")
            self.status_bar.remove_all(context_id)
            self.status_bar.push(context_id, status_msg)

            # Update Set buttons sensitivity based on detection
            for product in config_manager.SUPPORTED_PRODUCTS:
                for child in self.get_children():
                    self._update_button_sensitivity(child, product, product in detected)

        except Exception as e:
            logger.error(f"Error scanning devices: {e}")
            context_id = self.status_bar.get_context_id("device-status")
            self.status_bar.push(context_id, f"Scan error: {e}")
        finally:
            self.btn_refresh.set_sensitive(True)

    def _update_button_sensitivity(self, widget, product, is_connected):
        """Recursively update button sensitivity based on device detection."""
        if isinstance(widget, Gtk.Button) and f"Set {product}" in widget.get_label():
            widget.set_sensitive(is_connected)
        elif hasattr(widget, 'get_children'):
            for child in widget.get_children():
                self._update_button_sensitivity(child, product, is_connected)

    def sendStatic(self, product: str):
        """Send static color command to the device."""
        logger.info(f"Initiating static color for {product}")
        controller = G213Colors.LogitechDevice(product)
        if not controller.connect():
            logger.error(f"Failed to connect to {product}. Aborting command.")
            self._show_error_dialog(f"Connection failed: {product}", "Could not connect to the device. Check USB connection and permissions (udev rules).")
            return

        color_hex = self.btnGetHex(self.staticColorButton)
        if controller.send_color_command(color_hex):
            logger.info(f"Static color command sent to {product}.")
            field_for_static = 0
            command_to_save = controller.spec["colorCommand"].format(f"{field_for_static:02x}", color_hex)
            user_conf_path = config_manager.get_user_config_path(product)
            controller.save_configuration(command_to_save, user_conf_path)
        else:
            logger.error(f"Failed to send static color command to {product}.")
            self._show_error_dialog(f"Command Failed: {product}", "Could not send static color command.")
        controller.disconnect()

    def sendBreathe(self, product: str):
        """Send breathe effect command to the device."""
        logger.info(f"Initiating breathe effect for {product}")
        controller = G213Colors.LogitechDevice(product)
        if not controller.connect():
            logger.error(f"Failed to connect to {product}. Aborting command.")
            self._show_error_dialog(f"Connection failed: {product}", "Could not connect to the device.")
            return

        color_hex = self.btnGetHex(self.breatheColorButton)
        speed = self.sbBCycle.get_value_as_int()
        if controller.send_breathe_command(color_hex, speed):
            logger.info(f"Breathe command sent to {product}.")
            command_to_save = controller.spec["breatheCommand"].format(color_hex, f"{speed:04x}")
            user_conf_path = config_manager.get_user_config_path(product)
            controller.save_configuration(command_to_save, user_conf_path)
        else:
            logger.error(f"Failed to send breathe command to {product}.")
            self._show_error_dialog(f"Command Failed: {product}", "Could not send breathe command.")
        controller.disconnect()

    def sendCycle(self, product: str):
        """Send cycle effect command to the device."""
        logger.info(f"Initiating cycle effect for {product}")
        controller = G213Colors.LogitechDevice(product)
        if not controller.connect():
            logger.error(f"Failed to connect to {product}. Aborting command.")
            self._show_error_dialog(f"Connection failed: {product}", "Could not connect to the device.")
            return

        speed = self.sbCycle.get_value_as_int()
        if controller.send_cycle_command(speed):
            logger.info(f"Cycle command sent to {product}.")
            command_to_save = controller.spec["cycleCommand"].format(f"{speed:04x}")
            user_conf_path = config_manager.get_user_config_path(product)
            controller.save_configuration(command_to_save, user_conf_path)
        else:
            logger.error(f"Failed to send cycle command to {product}.")
            self._show_error_dialog(f"Command Failed: {product}", "Could not send cycle command.")
        controller.disconnect()

    def sendSegments(self, product: str):
        """Send segment color commands to the device."""
        logger.info(f"Initiating segment colors for {product}")
        if product == "G203":
            logger.warning("Segment mode is not applicable to G203. Applying color from first segment to whole device.")
            self.staticColorButton.set_rgba(self.segmentColorBtns[0].get_rgba())
            self.sendStatic(product)
            return

        controller = G213Colors.LogitechDevice(product)
        if not controller.connect():
            logger.error(f"Failed to connect to {product}. Aborting command.")
            self._show_error_dialog(f"Connection failed: {product}", "Could not connect to the device.")
            return

        commands_to_save_list = []
        all_segments_sent_successfully = True
        for i in range(1, 6):
            segment_color_hex = self.btnGetHex(self.segmentColorBtns[i-1])
            logger.debug(f"Sending segment {i} color {segment_color_hex} for {product}")
            if not controller.send_color_command(segment_color_hex, i):
                logger.error(f"Failed to send color for segment {i} to {product}.")
                all_segments_sent_successfully = False
                break

            command_for_segment = controller.spec["colorCommand"].format(f"{i:02x}", segment_color_hex)
            commands_to_save_list.append(command_for_segment)
            sleep(0.01)

        if all_segments_sent_successfully:
            logger.info(f"All segment commands sent to {product}.")
            full_data_to_save = "\n".join(commands_to_save_list)
            user_conf_path = config_manager.get_user_config_path(product)
            controller.save_configuration(full_data_to_save, user_conf_path)
        else:
            logger.warning(f"Segment color setting partially failed for {product}. Configuration not saved for this attempt.")
            self._show_error_dialog(f"Segment Command Failed: {product}", "Could not send all segment color commands.")
        controller.disconnect()

    def sendManager(self, product_target: str):
        """Manage sending commands based on current stack (effect) selection."""
        if product_target == "all":
            logger.info("Applying current effect settings to all configured products.")
            for product in config_manager.SUPPORTED_PRODUCTS:
                self.sendManager(product)
        else:
            stack_name = self.stack.get_visible_child_name()
            logger.info(f"Managing '{stack_name}' settings for product: {product_target}")
            if stack_name == "static":
                self.sendStatic(product_target)
            elif stack_name == "cycle":
                self.sendCycle(product_target)
            elif stack_name == "breathe":
                self.sendBreathe(product_target)
            elif stack_name == "segments":
                self.sendSegments(product_target)

    def on_button_clicked(self, button: Gtk.Button, product: str):
        """Handle Set button click."""
        logger.debug(f"Set button clicked for product: {product}. Current effect tab: {self.stack.get_visible_child_name()}")
        self.sendManager(product)

# --- Main Execution Guard ---
if __name__ == "__main__":
    # Parse arguments and handle CLI actions
    args = parse_arguments()
    handle_cli_actions(args)
    
    # If we reach here, launch the GUI
    logger.info("No specific CLI action requested by args, launching G213Colors GUI.")
    try:
        win = Window()
        win.connect("delete-event", Gtk.main_quit)
        win.show_all()
        Gtk.main()
    except Exception as e:
        logger.critical(f"Critical error launching GUI: {e}", exc_info=True)
        print(f"CRITICAL GUI LAUNCH ERROR: {e}", file=sys.stderr)
        try:
            error_dialog = Gtk.MessageDialog(
                message_type=Gtk.MessageType.ERROR,
                buttons=Gtk.ButtonsType.OK,
                text="Fatal Error Launching G213Colors",
            )
            error_dialog.format_secondary_text(str(e))
            error_dialog.run()
            error_dialog.destroy()
        except Exception as ed_e:
            print(f"Could not display Gtk error dialog: {ed_e}", file=sys.stderr)
        sys.exit(1)
