# Agent Instructions for Frost-Tune

## 🏗️ Project Architecture Map

This project is a native desktop application built with **Rust** and **Iced** GUI framework. It strictly adheres to the KISS principle.

- **`src/main.rs`**: Application entry point. Initializes Iced runtime and window.
- **`src/lib.rs`**: Core library exports and module definitions.
- **`src/ui/`**: Contains all Iced widget definitions. The UI runs on the main thread.
- **`src/hardware/`**: USB HID communication and DSP logic.
  - `usb_worker.rs`: Background thread for hidapi.
  - `protocol.rs`: Raw USB packet building.
  - `dsp.rs`: Freq/Q/Gain to Biquad math.
- **`src/models/`**: Data structures (Filter, PEQData, DeviceInfo).

## 🎨 Theme

Uses **Tokyo Night** theme by default (built into Iced).

## 🚀 Safety & Coding Rules

**MANDATORY**: Agents must strictly adhere to the following when modifying code:

### 1. The Unified Threading Model
Never call `hidapi.read()` or `hidapi.write()` directly from a UI widget. All HID calls must happen in the `hardware/usb_worker.rs` background thread.

### 2. Audio Safety Protocol
- **Max Gain**: Band gain capped at **+10dB**. Global preamp capped at **+10dB**.
- **Transactional Push**: Always follow `Write -> Read Back -> Verify` pattern.
- If read-back mismatches sent data, rollback immediately.

### 3. Multi-DAC Architecture
- Device detection based on `vendor_id` and `product_id`.
- Protocol logic in `hardware/protocol.rs` should be trait-based or enum-driven for multiple DACs.
- Start with TP35 Pro as `Device::TP35Pro`.

### 4. LLM-Friendly Rust
- Use explicit types everywhere.
- Keep UI state simple (no complex actor models).
- Document unsafe blocks.

### 5. Documentation Requirement for UI Changes
Before implementing any UI changes (Iced widgets, Application trait, Subscriptions, styling, or theming), agents MUST:
1. Search relevant Iced documentation (docs.rs, mintlify, GitHub examples)
2. Reference existing Iced examples in the codebase or official repo
3. Verify API compatibility with the project's Iced version (currently `0.14`)

**Rationale**: Iced API evolves frequently. Using outdated patterns from older tutorials will cause build failures or runtime bugs.

## 🛠️ Setup & Development

1. **Install Rust**: https://rustup.rs/
2. **Build**: `cargo build --release`
3. **Run**: `cargo run --release`
4. **Type check**: `cargo check`
5. **Test**: `cargo test`

## 📚 Reference Projects

- Original: `../tp35pro-eq` (Tauri + React + Rust)
- Hardware layer can be ported from `../tp35pro-eq/src-tauri/src/`

## 🌍 Cross-Platform Notes

- **Linux**: Requires `libhidapi-dev` (`sudo apt install libhidapi-dev`)
- **Windows**: Requires WebView2 (usually pre-installed on Win10/11)