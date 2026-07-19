# Rust/Iced G213 MVP TODO

## Goal

Convert the project to a Rust + Iced application while keeping the first working
target focused on Logitech G213. Keep the design open for future Logitech G
device contributions through a device registry and capability model.

## Phase 1 - Rust Core

- [x] Scaffold Rust crate and remove the old implementation after Rust parity.
- [x] Add a G213-only `DeviceSpec` registry entry.
- [x] Implement command generation for static, breathe, cycle, and 5 segments.
- [x] Implement config parsing/saving with the existing `PRODUCT=G213` format.
- [x] Implement USB send/apply flow with `rusb`.
- [x] Add unit tests for validation, command generation, and config parsing.

## Phase 2 - CLI Parity

- [x] Support `-t` / `--apply-system-default`.
- [x] Support `--apply-user-config` and `--apply-user-config G213`.
- [x] Add developer commands for detect/static/breathe/cycle/segment tests.
- [x] Keep unsupported products explicit and contributor-friendly.

## Phase 3 - Iced GUI

- [x] Build G213-only GUI with Static, Cycle, Breathe, and Segments views.
- [x] Add Scan G213 and Set G213 actions.
- [x] Add G213 login autostart toggle.
- [x] Run USB work as tasks so the UI remains responsive.
- [x] Surface status and errors clearly.

## Phase 4 - Install/Docs

- [x] Update desktop entry and service to call the Rust binary.
- [x] Update install/uninstall targets for the Rust binary.
- [x] Keep udev rule for G213 `046d:c336`.
- [x] Add contributor docs for adding more Logitech G devices.

## Hardware Safety

- [x] Test detect first.
- [ ] Test one USB lighting command at a time.
- [ ] Verify the kernel driver is reattached after each command.
- [ ] Avoid command loops while the G213 is being used as the active keyboard.
