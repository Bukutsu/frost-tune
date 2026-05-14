# Frost-Tune — Developer Guidelines

## Project Overview

Frost-Tune is a native parametric EQ editor for USB DACs, built with Rust and the Iced GUI framework. It communicates with DACs over USB HID to adjust 10-band parametric EQ directly on hardware.

**Version:** 0.8.2  
**Tech stack:** Rust 2021, Iced 0.14 (GUI), hidapi (HID I/O), tokio (async), serde/serde_json (serialization)  
**Target platforms:** Linux (primary), Windows  
**Status:** Actively maintained, CLI + GUI releases on Arch Linux AUR

## Architecture

```
frost-tune/
├── packaging/arch/      # Arch Linux PKGBUILD
│   └── PKGBUILD
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── autoeq.rs            # AutoEQ profile format
│   ├── diagnostics.rs       # Device diagnostics
│   ├── error.rs             # Error types (AppError + ErrorKind)
│   ├── storage.rs           # Profile persistence
│   ├── hardware/            # HID/protocol layer
│   │   ├── mod.rs
│   │   ├── dsp.rs           # Biquad filter computation
│   │   ├── elevated_transport.rs  # Linux privilege escalation
│   │   ├── helper_ipc.rs    # Helper process IPC
│   │   ├── helper_server.rs # Elevated helper server
│   │   ├── hid.rs           # HID transport
│   │   ├── operations.rs    # High-level hardware ops
│   │   ├── packet_builder.rs # Packet construction
│   │   ├── packet_format.rs # Packet constants, offsets, timing structs
│   │   ├── pipeline.rs      # Read/write pipeline
│   │   ├── protocol.rs      # DeviceProtocol trait + TP35ProProtocol
│   │   └── worker/          # Background worker thread
│   │       ├── mod.rs       # WorkerState struct, UsbWorker
│   │       ├── backend.rs   # TransportBackend enum
│   │       ├── connection.rs # Connection logic
│   │       └── ops.rs       # Pull/push operations
│   ├── models/              # Domain types
│   │   ├── mod.rs
│   │   ├── constants.rs     # EQ limits, band count, etc.
│   │   ├── device.rs        # Device definitions + registration
│   │   ├── filter.rs        # Filter model (freq, gain, Q, type)
│   │   └── ipc.rs           # IPC message types
│   └── ui/                  # Iced GUI
│       ├── mod.rs
│       ├── graph.rs         # Frequency response curve rendering
│       ├── main_window.rs   # Window layout
│       ├── messages.rs      # Message enum (68 variants)
│       ├── state.rs         # App state
│       ├── theme.rs         # Tokyo Night styling
│       ├── tokens.rs        # Design tokens
│       ├── update/          # Message handlers (5 sub-modules)
│       └── views/           # UI view components (9 files)
└── tests/                   # Integration tests
    ├── protocol.rs
    ├── token_consistency.rs
    └── worker_ipc.rs
```

## Essential Commands

```bash
# Development workflow
cargo fmt --all                  # Format code (required before commit)
cargo fmt --check                # Verify formatting
cargo clippy --all-targets       # Lint (target: 0 warnings)
cargo test --all-targets         # Run all 65+ tests
cargo check --all-targets        # Fast build check

# Run the application
cargo run --release              # Start the app with optimizations

# Package for Arch Linux
cd packaging/arch && makepkg -si
```

## Code Standards

- **Edition:** Rust 2021
- **Comments:** None by default. Only add if explaining *why* (hidden constraints, workarounds, non-obvious invariants)
- **Error handling:** Uniform `Result<T, AppError>` across ALL modules. `AppError` uses `thiserror` with `kind: ErrorKind`, `message`, and optional `context`. Defined in `error.rs`.
- **Async:** tokio runtime for background HID I/O; UI runs on main thread
- **Threading:** HID I/O always isolated on background threads (`std::thread` + `mpsc`), never on UI thread
- **Writes:** Every EQ write follows push → read-back → verify → rollback pattern
- **Safety:** Band gain and global preamp capped at ±10 dB; no unsafe code anywhere
- **Linting:** Zero clippy warnings in library code (excluding dependency warnings)

## Code Organization & Helpers

**Update handlers** (`src/ui/update/`):
- **`editor.rs`:** `handle_band_text_input()` consolidates freq/gain/Q input handlers via closure; `cancel_band_draft_input()` handles all three cancel variants
- **`connection.rs`:** `poll_worker_status()`, `maybe_reconnect()`, `maybe_check_profiles()` break up the 101-line `Tick` arm into focused, testable functions
- **`hardware.rs`:** `is_hw_busy()` replaces repeated 4-condition guard across PullPressed/PushPressed
- **`profiles.rs`:** `reload_profiles_task()` centralizes the 5 identical Task::perform calls

**Hardware layer** (`src/hardware/`):
- **`packet_format.rs`:** Single source of truth for packet constants, offsets, timing structs (breaks dsp/protocol circular dependency)
- **`worker/mod.rs`:** `WorkerState` struct bundles all mutable state; `run_iteration()` is a method (not 13 loose parameters)

**UI views** (`src/ui/views/`):
- **`bands.rs`:** `render_band_row` delegates to 4 focused sub-functions; keep rendering functions small
- **`header.rs`:** Toolbar buttons extracted to shared `sync_toolbar_button()` pattern

## Testing & Quality

- **49 unit tests** (inline `#[cfg(test)]`) + **16 integration tests** = **65 total**
- Run: `cargo test --all-targets`
- **Protocol tests** validate packet construction/parsing for TP35Pro
- **Token consistency tests** ensure UI design tokens match design system (WCAG AA contrast enforced)
- **Worker/IPC tests** cover serialization roundtrips, version handshake, error handling
- **Clippy:** Zero warnings in library code (v0.8.2+ passes all lints)

## Adding New Devices

1. Implement `DeviceProtocol` trait in `src/hardware/protocol.rs`
2. Register the device in `src/models/device.rs`
3. Follow the contributor guide comments at the bottom of `device.rs`

## Architecture Patterns

- **DeviceProtocol trait:** Defines the HID packet protocol per device (build read/write packets, parse responses)
- **Iced Elm architecture:** State + Messages + Update + View pattern in `ui/`
- **AutoEQ format:** Profiles stored as plain text, compatible with AutoEQ ecosystem
- **Linux elevation:** Uses `pkexec` with the binary itself as a temporary helper; no system-wide install needed
- **WorkerState pattern:** Background worker encapsulates mutable state, runs on dedicated thread

---

## Claude Code Configuration

### graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

**Rules:**
- ALWAYS read graphify-out/GRAPH_REPORT.md before reading any source files, running grep/glob searches, or answering codebase questions. The graph is your primary map of the codebase.
- IF graphify-out/wiki/index.md EXISTS, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
