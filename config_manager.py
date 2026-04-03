"""
Configuration management module for G213Colors.
Handles loading, saving, and applying configuration files.
"""

import os
import re
import logging
from pathlib import Path
from typing import Optional
import G213Colors

logger = logging.getLogger(__name__)

# Application constants — derived from G213Colors to avoid duplication
APP_NAME = "G213 Colors"
SUPPORTED_PRODUCTS = list(G213Colors.LogitechDevice.PRODUCT_SPECS.keys())
USER_CONFIG_DIR = os.path.join(os.path.expanduser("~"), ".config", "G213Colors")
AUTOSTART_DIR = os.path.join(os.path.expanduser("~"), ".config", "autostart")

# --- Input Sanitization ---

def _sanitize_product_name(product_name: str) -> str:
    """
    Sanitize and validate a product name.

    Args:
        product_name: Raw product name input

    Returns:
        Validated product name

    Raises:
        ValueError: If product name is invalid or not supported
    """
    if not isinstance(product_name, str):
        raise ValueError(f"Product name must be a string, got {type(product_name).__name__}")
    if not re.match(r'^[A-Za-z0-9]+$', product_name):
        raise ValueError(f"Invalid product name: '{product_name}'. Only alphanumeric characters allowed.")
    if product_name not in SUPPORTED_PRODUCTS:
        raise ValueError(f"Unsupported product: '{product_name}'. Must be one of {SUPPORTED_PRODUCTS}")
    return product_name


def _safe_config_path(base_dir: str, filename: str) -> str:
    """
    Construct a safe config file path and validate it stays within base_dir.

    Args:
        base_dir: Base configuration directory
        filename: Config filename

    Returns:
        Safe absolute path

    Raises:
        ValueError: If path would escape base_dir
    """
    full_path = os.path.normpath(os.path.join(base_dir, filename))
    abs_base = os.path.normpath(os.path.abspath(base_dir))
    abs_full = os.path.normpath(os.path.abspath(full_path))

    if not abs_full.startswith(abs_base + os.sep) and abs_full != abs_base:
        raise ValueError(f"Path traversal detected: '{full_path}' is outside '{base_dir}'")

    return abs_full


def get_user_config_path(product_name: str) -> str:
    """
    Get the path to the user configuration file for a product.

    Args:
        product_name: Product name (validated)

    Returns:
        Safe absolute path to config file
    """
    product_name = _sanitize_product_name(product_name)
    return _safe_config_path(USER_CONFIG_DIR, f"{product_name}.conf")


def get_autostart_desktop_file_path(product_name: str) -> str:
    """
    Get the path to the autostart desktop file for a product.

    Args:
        product_name: Product name (validated)

    Returns:
        Safe absolute path to desktop file
    """
    product_name = _sanitize_product_name(product_name)
    return _safe_config_path(AUTOSTART_DIR, f"g213colors-autostart-{product_name}.desktop")


def ensure_config_dirs() -> bool:
    """
    Ensure configuration directories exist.
    
    Returns:
        True if directories exist or were created successfully
    """
    try:
        Path(USER_CONFIG_DIR).mkdir(parents=True, exist_ok=True)
        Path(AUTOSTART_DIR).mkdir(parents=True, exist_ok=True)
        return True
    except OSError as e:
        logger.error(f"Failed to create configuration directories: {e}")
        return False


def create_autostart_entry(product_name: str) -> bool:
    """
    Create an autostart desktop entry for a product.

    Args:
        product_name: Product name (e.g., "G213", "G203")

    Returns:
        True if entry created successfully
    """
    product_name = _sanitize_product_name(product_name)
    desktop_file_path = get_autostart_desktop_file_path(product_name)
    desktop_content = f"""[Desktop Entry]
Name=G213Colors Autostart ({product_name})
Comment=Apply saved G213Colors settings for {product_name} on login
Exec=/usr/bin/g213colors-gui --apply-user-config {product_name}
Icon=g213colors
Terminal=false
Type=Application
Categories=Utility;
X-GNOME-Autostart-enabled=true
"""
    try:
        ensure_config_dirs()
        with open(desktop_file_path, "w") as f:
            f.write(desktop_content)
        os.chmod(desktop_file_path, 0o775)
        logger.info(f"Created autostart entry for {product_name} at {desktop_file_path}")
        return True
    except IOError as e:
        logger.error(f"Failed to create autostart file for {product_name}: {e}")
        return False


def remove_autostart_entry(product_name: str) -> bool:
    """
    Remove an autostart desktop entry for a product.

    Args:
        product_name: Product name (e.g., "G213", "G203")

    Returns:
        True if entry removed successfully
    """
    product_name = _sanitize_product_name(product_name)
    desktop_file_path = get_autostart_desktop_file_path(product_name)
    if os.path.exists(desktop_file_path):
        try:
            os.remove(desktop_file_path)
            logger.info(f"Removed autostart entry for {product_name} from {desktop_file_path}")
            return True
        except OSError as e:
            logger.error(f"Failed to remove autostart file for {product_name}: {e}")
            return False
    return True


def is_autostart_enabled(product_name: str) -> bool:
    """Check if autostart is enabled for a product."""
    product_name = _sanitize_product_name(product_name)
    desktop_file_path = get_autostart_desktop_file_path(product_name)
    return os.path.exists(desktop_file_path)


def apply_system_default_config() -> bool:
    """
    Apply the system-wide default configuration.
    
    Returns:
        True if configuration applied successfully
    """
    logger.info("Applying system-wide default configuration")
    return G213Colors.LogitechDevice.apply_configuration_from_file(
        G213Colors.LogitechDevice.SYSTEM_DEFAULT_CONF_FILE
    )


def apply_user_config(product_name: str) -> bool:
    """
    Apply user-specific saved configuration for a product.

    Args:
        product_name: Product name (e.g., "G213", "G203")

    Returns:
        True if configuration applied successfully
    """
    product_name = _sanitize_product_name(product_name)
    logger.info(f"Applying user configuration for {product_name}")
    user_conf_path = get_user_config_path(product_name)

    if not os.path.exists(user_conf_path):
        logger.warning(f"User configuration file not found for {product_name} at {user_conf_path}")
        return False

    return G213Colors.LogitechDevice.apply_configuration_from_file(user_conf_path)
