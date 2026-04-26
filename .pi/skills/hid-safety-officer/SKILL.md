---
name: hid-safety-officer
description: Enforces USB HID safety and transactional integrity.
---

# HID Safety Protocol

Use this skill when modifying `src/hardware/usb_worker.rs`, `src/hardware/protocol.rs`, or any code that interacts with the DAC hardware.

## Threading Model
1. **Non-Blocking UI**: Never call `hidapi.read()` or `hidapi.write()` directly from a UI widget or the main thread. All HID calls MUST occur within the background thread managed in `usb_worker.rs`.

## Data Integrity (Transactional Push)
1. **Write $\rightarrow$ Read Back $\rightarrow$ Verify**: Always follow this sequence:
   - Write the desired value to the device.
   - Read the value back from the device.
   - Verify the read value matches the written value.
2. **Immediate Rollback**: If a verification read fails or the device returns an error, immediately trigger a rollback to the last known good state and notify the user via the UI.

## Audio Safety
1. **Gain Caps**: Hard-cap all Band gain and Global preamp modifications at **+10dB**. 
2. **Clamping**: If a requested value exceeds +10dB, the agent must warn the user and clamp the value to +10.0.

## Hardware Architecture
- Ensure that protocol changes in `protocol.rs` are trait-based or enum-driven to support future DAC models beyond the TP35 Pro.
