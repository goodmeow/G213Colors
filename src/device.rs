use std::time::Duration;

use rusb::{DeviceHandle, GlobalContext};

use crate::command::{commands_for_effect, Effect};
use crate::config::{read_config, save_user_config};
use crate::error::{G213Error, Result};
use crate::product::{spec_for, DeviceSpec, Product};

const USB_BM_REQUEST_TYPE: u8 = 0x21;
const USB_BM_REQUEST: u8 = 0x09;
const USB_W_INDEX: u16 = 0x0001;
const USB_INTERFACE: u8 = 1;
const USB_READ_ENDPOINT: u8 = 0x82;
const USB_READ_TIMEOUT: Duration = Duration::from_millis(1000);
const USB_WRITE_TIMEOUT: Duration = Duration::from_millis(1000);

pub fn detect(product: Product) -> bool {
    let spec = spec_for(product);
    let Ok(devices) = rusb::devices() else {
        return false;
    };

    devices.iter().any(|device| {
        device.device_descriptor().is_ok_and(|descriptor| {
            descriptor.vendor_id() == spec.vendor_id && descriptor.product_id() == spec.product_id
        })
    })
}

pub fn apply_effect(product: Product, effect: &Effect) -> Result<()> {
    let spec = spec_for(product);
    let commands = commands_for_effect(spec, effect)?;
    apply_commands(spec, &commands)?;
    save_user_config(product, &commands)?;
    Ok(())
}

pub fn apply_config_file(path: impl AsRef<std::path::Path>) -> Result<()> {
    let config = read_config(path)?;
    let spec = spec_for(config.product);
    apply_commands(spec, &config.commands)
}

pub fn apply_commands(spec: &DeviceSpec, commands: &[String]) -> Result<()> {
    let mut session = DeviceSession::connect(spec)?;
    for command in commands {
        let expects_receive = command_expects_receive(spec, command)?;
        session.send_hex(command)?;
        if expects_receive {
            session.receive_optional();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn command_expects_receive(spec: &DeviceSpec, command: &str) -> Result<bool> {
    let Some(marker) = spec.receive_after_command_marker else {
        return Ok(false);
    };

    let data = hex::decode(command)?;
    Ok(data
        .get(marker.byte_index)
        .is_some_and(|value| *value == marker.value))
}

struct DeviceSession<'a> {
    spec: &'a DeviceSpec,
    handle: DeviceHandle<GlobalContext>,
    kernel_driver_detached: bool,
}

impl<'a> DeviceSession<'a> {
    fn connect(spec: &'a DeviceSpec) -> Result<Self> {
        let handle = rusb::open_device_with_vid_pid(spec.vendor_id, spec.product_id)
            .ok_or_else(|| G213Error::DeviceNotFound(spec.product.to_string()))?;

        let mut kernel_driver_detached = false;
        match handle.kernel_driver_active(USB_INTERFACE) {
            Ok(true) => {
                handle.detach_kernel_driver(USB_INTERFACE)?;
                kernel_driver_detached = true;
            }
            Ok(false) => {}
            Err(rusb::Error::NotSupported) => {}
            Err(error) => return Err(error.into()),
        }

        Ok(Self {
            spec,
            handle,
            kernel_driver_detached,
        })
    }

    fn send_hex(&mut self, command: &str) -> Result<()> {
        let data = hex::decode(command)?;
        self.handle.write_control(
            USB_BM_REQUEST_TYPE,
            USB_BM_REQUEST,
            self.spec.w_value,
            USB_W_INDEX,
            &data,
            USB_WRITE_TIMEOUT,
        )?;
        Ok(())
    }

    fn receive_optional(&mut self) {
        let mut buffer = [0_u8; 64];
        let _ = self
            .handle
            .read_interrupt(USB_READ_ENDPOINT, &mut buffer, USB_READ_TIMEOUT);
    }
}

impl Drop for DeviceSession<'_> {
    fn drop(&mut self) {
        if self.kernel_driver_detached {
            let _ = self.handle.attach_kernel_driver(USB_INTERFACE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::G213_SPEC;

    #[test]
    fn g213_receives_only_after_color_commands() {
        assert!(
            command_expects_receive(&G213_SPEC, "11ff0c3a0001ffffff0200000000000000000000")
                .unwrap()
        );
        assert!(
            !command_expects_receive(&G213_SPEC, "11ff0c3a000200ff001388006400000000000000")
                .unwrap()
        );
        assert!(
            !command_expects_receive(&G213_SPEC, "11ff0c3a0003ffffff0000138864000000000000")
                .unwrap()
        );
    }
}
