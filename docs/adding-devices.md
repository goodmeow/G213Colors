# Adding Logitech G Devices

The Rust port starts with G213 support only, but the core is intentionally built
around a device registry. New devices should be added by contributing a new
`DeviceSpec` entry and tests, not by special-casing USB logic.

## Add a Device Spec

Edit `src/product.rs` and add a new constant:

```rust
pub const GXXX_SPEC: DeviceSpec = DeviceSpec {
    product: Product::Gxxx,
    display_name: "Logitech GXXX ...",
    vendor_id: 0x046d,
    product_id: 0x0000,
    w_value: 0x0000,
    color_command: "...{field}...{color}...",
    breathe_command: "...{color}{speed}...",
    cycle_command: "...{speed}...",
    zones: ZoneModel::Segmented { zones: 5 },
    receive_after_command_marker: Some(CommandMarker {
        byte_index: 5,
        value: 0x01,
    }),
};
```

Then add it to:

```rust
pub static DEVICE_SPECS: &[DeviceSpec] = &[
    G213_SPEC,
    GXXX_SPEC,
];
```

## Product Enum

Add a new variant in `Product` and update:

- `Product::key`
- `FromStr`

Config files use this key:

```text
PRODUCT=G213
<raw command>
```

## Capabilities

Do not assume all devices support the same controls. Add the smallest capability
surface needed for the device:

- static color
- breathe
- cycle
- segmented zones
- receive-after-command marker, when the device expects a read after specific commands

If a device has a different model, extend `ZoneModel` or `DeviceSpec` first and
keep the UI capability-driven.

## Tests

Add command-generation tests in `src/command.rs` before testing hardware:

- static command
- breathe command
- cycle command
- segment/zone command, when supported
- invalid segment handling

Run:

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

## Hardware Validation

Test one command at a time:

```bash
cargo run -- detect
cargo run -- set-static ff0000
cargo run -- set-cycle 5000
cargo run -- set-breathe 00ff00 5000
```

If the keyboard or mouse is actively being used, avoid loops and verify input
still works after each command. The USB session detaches the kernel driver while
sending and attempts to reattach it on drop.

## Installer

Add a udev rule for the new product ID in `makefile`:

```text
SUBSYSTEM=="usb", ATTR{idVendor}=="046d", ATTR{idProduct}=="0000", TAG+="uaccess"
```
