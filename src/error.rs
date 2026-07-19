use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum G213Error {
    #[error("unsupported product '{product}'. This build supports: {supported}")]
    UnsupportedProduct { product: String, supported: String },

    #[error("invalid color '{0}'. Must be exactly 6 hex digits")]
    InvalidColor(String),

    #[error("invalid speed {0}. Must be between 500 and 65535 ms")]
    InvalidSpeed(u32),

    #[error("invalid segment {segment}. Must be between 1 and {max}")]
    InvalidSegment { segment: u8, max: u8 },

    #[error("configuration file {0} is missing PRODUCT= header")]
    MissingProductHeader(PathBuf),

    #[error("configuration file {0} has no commands")]
    EmptyConfig(PathBuf),

    #[error("USB device {0} not found")]
    DeviceNotFound(String),

    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, G213Error>;
