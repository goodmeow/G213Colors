use std::fmt;
use std::str::FromStr;

use crate::error::{G213Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Product {
    G213,
}

impl Product {
    pub const fn key(self) -> &'static str {
        match self {
            Self::G213 => "G213",
        }
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

impl FromStr for Product {
    type Err = G213Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "G213" => Ok(Self::G213),
            other => Err(G213Error::UnsupportedProduct {
                product: other.to_string(),
                supported: supported_products().join(", "),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneModel {
    Segmented { zones: u8 },
}

#[derive(Debug, Clone, Copy)]
pub struct CommandMarker {
    pub byte_index: usize,
    pub value: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceSpec {
    pub product: Product,
    pub display_name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub w_value: u16,
    pub color_command: &'static str,
    pub breathe_command: &'static str,
    pub cycle_command: &'static str,
    pub zones: ZoneModel,
    pub receive_after_command_marker: Option<CommandMarker>,
}

impl DeviceSpec {
    pub const fn zone_count(self) -> u8 {
        match self.zones {
            ZoneModel::Segmented { zones } => zones,
        }
    }
}

pub const G213_SPEC: DeviceSpec = DeviceSpec {
    product: Product::G213,
    display_name: "Logitech G213 Prodigy Gaming Keyboard",
    vendor_id: 0x046d,
    product_id: 0xc336,
    w_value: 0x0211,
    color_command: "11ff0c3a{field}01{color}0200000000000000000000",
    breathe_command: "11ff0c3a0002{color}{speed}006400000000000000",
    cycle_command: "11ff0c3a0003ffffff0000{speed}64000000000000",
    zones: ZoneModel::Segmented { zones: 5 },
    receive_after_command_marker: Some(CommandMarker {
        byte_index: 5,
        value: 0x01,
    }),
};

pub static DEVICE_SPECS: &[DeviceSpec] = &[G213_SPEC];

pub fn supported_products() -> Vec<&'static str> {
    DEVICE_SPECS.iter().map(|spec| spec.product.key()).collect()
}

pub fn spec_for(product: Product) -> &'static DeviceSpec {
    DEVICE_SPECS
        .iter()
        .find(|spec| spec.product == product)
        .expect("registered product must have a device spec")
}

pub fn spec_by_key(product: &str) -> Result<&'static DeviceSpec> {
    let product = Product::from_str(product)?;
    Ok(spec_for(product))
}
