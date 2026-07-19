use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::command::validate_command_for_spec;
use crate::error::{G213Error, Result};
use crate::product::{spec_by_key, Product};

pub const APP_DIR_NAME: &str = "G213Colors";
pub const SYSTEM_DEFAULT_CONF_FILE: &str = "/etc/G213Colors.conf";
const DEFAULT_INSTALLED_BIN_PATH: &str = "/usr/bin/g213colors";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceConfig {
    pub product: Product,
    pub commands: Vec<String>,
}

pub fn ensure_user_dirs() -> Result<()> {
    fs::create_dir_all(user_config_dir()?)?;
    fs::create_dir_all(autostart_dir()?)?;
    Ok(())
}

pub fn user_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "user config directory not found",
        )
    })?;
    Ok(config_dir.join(APP_DIR_NAME))
}

pub fn user_config_path(product: Product) -> Result<PathBuf> {
    Ok(user_config_dir()?.join(format!("{product}.conf")))
}

pub fn autostart_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "user config directory not found",
        )
    })?;
    Ok(config_dir.join("autostart"))
}

pub fn autostart_path(product: Product) -> Result<PathBuf> {
    Ok(autostart_dir()?.join(format!("g213colors-autostart-{product}.desktop")))
}

pub fn read_config(path: impl AsRef<Path>) -> Result<DeviceConfig> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)?;
    parse_config(&contents, path)
}

pub fn parse_config(contents: &str, path: &Path) -> Result<DeviceConfig> {
    let mut lines = contents.lines();
    let product_line = lines
        .next()
        .ok_or_else(|| G213Error::MissingProductHeader(path.to_path_buf()))?
        .trim();

    let Some(product_key) = product_line.strip_prefix("PRODUCT=") else {
        return Err(G213Error::MissingProductHeader(path.to_path_buf()));
    };

    let spec = spec_by_key(product_key.trim())?;
    let commands = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if commands.is_empty() {
        return Err(G213Error::EmptyConfig(path.to_path_buf()));
    }

    for command in &commands {
        validate_command_for_spec(spec, command)?;
    }

    Ok(DeviceConfig {
        product: spec.product,
        commands,
    })
}

pub fn save_config(product: Product, commands: &[String], path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    writeln!(file, "PRODUCT={product}")?;
    for command in commands {
        writeln!(file, "{command}")?;
    }
    Ok(())
}

pub fn save_user_config(product: Product, commands: &[String]) -> Result<()> {
    save_config(product, commands, user_config_path(product)?)
}

pub fn create_autostart_entry(product: Product) -> Result<()> {
    ensure_user_dirs()?;
    let path = autostart_path(product)?;
    let content = format!(
        "[Desktop Entry]\n\
         Name=G213Colors Autostart ({product})\n\
         Comment=Apply saved G213Colors settings for {product} on login\n\
         Exec={} --apply-user-config {product}\n\
         Icon=g213colors\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Utility;\n\
         X-GNOME-Autostart-enabled=true\n",
        installed_bin_path()
    );
    fs::write(&path, content)?;

    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;

    Ok(())
}

pub fn remove_autostart_entry(product: Product) -> Result<()> {
    let path = autostart_path(product)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn is_autostart_enabled(product: Product) -> bool {
    autostart_path(product).is_ok_and(|path| path.exists())
}

pub fn system_default_path() -> &'static Path {
    Path::new(SYSTEM_DEFAULT_CONF_FILE)
}

fn installed_bin_path() -> &'static str {
    option_env!("G213COLORS_BIN_PATH").unwrap_or(DEFAULT_INSTALLED_BIN_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_existing_config_format() {
        let config = parse_config(
            "PRODUCT=G213\n11ff0c3a0001ffffff0200000000000000000000\n",
            Path::new("test.conf"),
        )
        .unwrap();

        assert_eq!(config.product, Product::G213);
        assert_eq!(config.commands.len(), 1);
    }

    #[test]
    fn rejects_missing_product_header() {
        let err = parse_config(
            "11ff0c3a0001ffffff0200000000000000000000\n",
            Path::new("test.conf"),
        )
        .unwrap_err();

        assert!(matches!(err, G213Error::MissingProductHeader(_)));
    }

    #[test]
    fn rejects_unsupported_product() {
        let err = parse_config("PRODUCT=G999\nabc\n", Path::new("test.conf")).unwrap_err();
        assert!(matches!(err, G213Error::UnsupportedProduct { .. }));
    }
}
