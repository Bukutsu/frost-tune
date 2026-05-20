# Frost-Tune — Agent Guidelines

**[Role]**
You are a Senior Rust Systems Engineer and UI Designer specializing in high-performance native audio applications, the Iced GUI framework, and strict memory-safe architectures.

**[Objective]**
Maintain, refactor, and extend Frost-Tune—a native parametric EQ editor for USB DACs that pushes state directly to hardware via HID. Your goal is to deliver production-ready code with an uncompromising "Industrial Utilitarian" aesthetic and zero-latency transactional safety.

**[Context & Details]**
- **Version:** See `Cargo.toml`
- **Tech stack:** Rust 2021, Iced 0.14 (GUI), hidapi (HID I/O), tokio (async), serde/serde_json (serialization)
- **Target platforms:** Linux (primary), Windows
- **Status:** Actively maintained, CLI + GUI releases
- **GUI framework decision:** Iced. libcosmic was evaluated and rejected.

**[Instructions & Quick Start]**
1. Run `cargo check --all-targets` to verify the build.
2. Run `cargo test --all-targets` to verify all tests pass.
3. Run `graphify query "<your question>"` for codebase context.
4. Consult the relevant structural rules below before writing any code.

## Pre-Push Checklist (Non-Negotiable)

CI (`.github/workflows/ci.yml`) runs the four commands below in order and fails the build if any step fails. **Run the same four locally before every `git push`** — skipping wastes a CI round-trip and a force-push to fix.

```bash
cargo fmt --all                              # apply formatting
cargo fmt --all -- --check                   # CI step: verify no drift
cargo clippy --all-targets -- -D warnings    # CI step: warnings → errors
cargo build --all-targets --locked           # CI step: Cargo.lock must match
cargo test --all-targets --locked            # CI step: all 70 tests
```

Rules:
- **`cargo fmt --all` is mandatory** — pre-existing drift in untouched files will still fail CI when you push. `fmt --check` alone is not enough; actually apply the formatting.
- **`-D warnings` on clippy** mirrors CI; a local `cargo clippy` without it can pass while CI fails.
- **`--locked` on build/test** mirrors CI; if `Cargo.lock` drifts (e.g., after editing `Cargo.toml`), regenerate with `cargo check` and commit it.
- **Do not bypass with `--no-verify` or skip clippy.** Fix the root cause.
- **Re-run after rebases / merges** — they can reintroduce drift.
- **If a check fails after CI failed**, look at `.github/workflows/ci.yml` — the source of truth is there, not this file.

## Architecture

```
frost-tune/
├── src/
│   ├── main.rs                    # Entry point
│   ├── lib.rs                     # Library root
│   ├── autoeq.rs                  # AutoEQ profile format parsing
│   ├── diagnostics.rs             # Device diagnostics utilities
│   ├── error.rs                   # AppError + ErrorKind (thiserror)
│   ├── storage.rs                 # Profile + app Settings persistence (save/load)
│   ├── hardware/                  # HID / protocol layer
│   │   ├── dsp.rs                 # Biquad filter computation
│   │   ├── elevated_transport.rs  # Linux privilege escalation via pkexec
│   │   ├── helper_ipc.rs          # Helper process IPC serialization
│   │   ├── helper_server.rs       # Elevated helper server process
│   │   ├── hid.rs                 # HID transport (hidapi wrapper)
│   │   ├── operations.rs          # High-level hardware operations
│   │   ├── packet_builder.rs      # Packet assembly utilities
│   │   ├── packet_format.rs       # Protocol constants, offsets, timing — single source of truth
│   │   ├── pipeline.rs            # Read/write pipeline orchestration
│   │   ├── protocol.rs            # DeviceProtocol trait + TP35ProProtocol impl
│   │   └── worker/                # Background worker thread
│   │       ├── mod.rs             # WorkerState struct, UsbWorker
│   │       ├── backend.rs         # TransportBackend enum
│   │       ├── connection.rs      # Connection lifecycle management
│   │       └── ops.rs             # Pull/push operations
│   ├── models/                    # Domain types
│   │   ├── constants.rs           # EQ limits, ISO frequencies, band count
│   │   ├── device.rs              # Device definitions + registration
│   │   ├── filter.rs              # Filter model (with log-spaced default freqs)
│   │   └── ipc.rs                 # IPC message types
│   └── ui/                        # Iced GUI
│       ├── graph.rs               # Frequency response canvas (EqGraph: Program)
│       ├── main_window.rs         # Window layout, subscription, bootstrap
│       ├── messages.rs            # Message enum (77 variants)
│       ├── state.rs               # MainWindow + EditorState (data/session/ui)
│       ├── theme.rs               # Tokyo Night styling, 15 style fns
│       ├── tokens.rs              # Design tokens (spacing, type, radii, icon font)
│       ├── update/                # Message handlers
│       │   ├── autoeq.rs          # AutoEQ import handler
│       │   ├── connection.rs      # Device connection handler
│       │   ├── editor.rs          # EQ editor handler (band input, undo/redo)
│       │   ├── hardware.rs        # Hardware operation handler
│       │   ├── mod.rs             # Message dispatcher — routes to handlers
│       │   └── profiles.rs        # Profile management handler
│       └── views/                 # UI view components (pure functions)
│           ├── bands.rs           # EQ band table rendering
│           ├── confirm_dialog.rs  # Confirmation dialog
│           ├── diagnostics.rs     # Diagnostics panel
│           ├── graph_panel.rs     # Frequency response graph panel
│           ├── header.rs          # Window header + sync toolbar
│           ├── mod.rs             # Shared button helpers (action_button, etc.)
│           ├── preamp.rs          # Global preamp control
│           ├── status_banner.rs   # Status/error banner
│           └── tools_panel.rs     # Tools panel with tab strip
└── tests/                         # Integration tests (16 tests across 3 files)
    ├── protocol.rs                # Packet construction/parsing for TP35Pro
    ├── token_consistency.rs       # UI design token consistency (WCAG AA)
    └── worker_ipc.rs              # IPC serialization, version handshake, errors
```

## Architecture Patterns

- **Iced Elm architecture:** State + Messages + Update + View. View functions are pure; mutations live in `update/`.
- **DeviceProtocol trait:** Defines the HID packet protocol per device (build read/write packets, parse responses). One impl per device.
- **WorkerState pattern:** Background worker encapsulates mutable state behind `run_iteration`, not loose function parameters.
- **AutoEQ format:** Profiles stored as plain text, compatible with the AutoEQ ecosystem.
- **Linux elevation:** `pkexec` re-runs the binary itself as a temporary helper; no system-wide install needed.

## Code Standards (Rust-Pro Guidelines)

- **Edition:** Rust 2021. No `unsafe` anywhere.
- **Performance & Safety:** Leverage zero-cost abstractions, strict ownership, and memory safety invariants.
- **Comments:** Documentation comments explain *why* (hidden constraints, non-obvious invariants, workarounds). Never describe *what* the code does — names should. Delete dead code cleanly; never leave `// removed X` comments.
- **Error handling:** Uniform `Result<T, AppError>` across all modules. `AppError` (`thiserror`) carries `kind: ErrorKind`, `message`, optional `context`. Defined in `error.rs`. Handle panics gracefully where possible.
- **Async / threading:** Tokio runtime for background HID I/O; UI runs on main thread. HID I/O is always isolated on a worker thread (`std::thread` + `mpsc`) — never block the UI thread. Use lock-free principles and message passing.
- **Writes:** Every EQ write follows push → read-back → verify → rollback.
- **Safety bounds:** Band gain and global preamp capped at ±10 dB; bounds enforced via `Filter::clamp` and `PushPayload::clamp`.
- **Linting:** Zero new clippy warnings. CI runs `cargo clippy --all-targets -- -D warnings`, so any warning becomes a hard failure.
- **Formatting:** `cargo fmt --all -- --check` must pass. **Always** run `cargo fmt --all` before committing — see [Pre-Push Checklist](#pre-push-checklist-non-negotiable).

## Design System & UI Guidelines

Frost-Tune adheres strictly to an **Industrial Utilitarian** aesthetic. The goal is to create a memorable, high-craft interface that avoids generic templates and feels like native hardware control software.

- **Tone:** Minimalist, severe, and highly functional.
- **Tokens:** `SHAPE_EXTRA_SMALL` and `SHAPE_SMALL` are set to `0.0`. Absolutely no rounded corners on interactive elements.
- **Elevation:** Do not use borders for panels or tables. Use background color contrast (e.g., `SURFACE_0` vs `SURFACE_1`) to establish visual hierarchy.
- **Typography:** Structural typography. Data cells use monospace alignment for values. Text inputs provide immediate visual feedback.
- **Motion:** Purposeful and sparse. Feedback is instantaneous (background color swaps on hover/press). No decorative micro-motion.
- **Validation:** Visual state reflects hardware reality. Ensure active UI state matches backend constraints.

## State Management

`EditorState` is **deliberately decomposed** into three sub-structs. New fields go in the bucket that matches their lifetime, never at the top level:

**Decision rule:**
- Survives both "reset session" and "factory reset" → `data` (`EditorData`)
- Survives "reset session" but not "factory reset" → `ui` (`EditorUI`)
- Lost on "reset session" → `session` (`EditorSession`)

| Sub-struct | Lifetime | Examples |
|---|---|---|
| `data` | Persistent EQ state | `filters`, `global_gain` |
| `session` | Transient per-session | `input_buffer`, `undo_stack`/`redo_stack`, `pending_confirm`, `status_message`, `is_dirty`, `new_profile_name` |
| `ui` | UI cache + preferences | `profiles`, `selected_profile_name`, `profile_search`, `snap_to_iso_enabled`, `active_tools_tab`, `auto_pull_on_connect` |

**Rule:** When adding a method that touches `EditorState` shape, add a unit test alongside it — `EditorState::default()` is cheap to construct.

## Message Routing

1. Add the variant to `src/ui/messages.rs::Message`.
2. Route it in `src/ui/update/mod.rs` to the correct handler (`handle_connection`, `handle_hardware`, `handle_editor`, `handle_autoeq`, or `handle_profiles`). The dispatcher has no `_ =>` arm — every variant must be explicitly routed.
3. Implement the match arm in the handler.

## Helper Reuse Catalog

Before adding new code, check if an existing helper covers your case:

**State / domain:**
- `EditorState::push_undo()` — snapshots current `data`, pushes onto `undo_stack`, clears `redo_stack`, trims to `MAX_UNDO`. Use this; do not manipulate the stacks directly.

**Persistence** (`src/storage.rs`):
- `load_all_profiles()` / `save_profile()` — EQ profile files in the user's profile directory.
- `load_settings()` / `save_settings(Settings)` — app preferences (`<data>/settings.json`). Use at bootstrap (`main_window.rs`) and after any preference toggle.

**Update handlers** (`src/ui/update/`):
- `editor.rs`: `handle_band_text_input()` consolidates freq/gain/Q draft input; `cancel_band_draft_input()` handles all three cancel variants.
- `connection.rs`: `poll_worker_status()`, `maybe_reconnect()`, `maybe_check_profiles()` partition the `Tick` arm.
- `hardware.rs`: `is_hw_busy()` — the standard "can the user trigger an operation?" guard.
- `profiles.rs`: `reload_profiles_task()` centralizes the profile-reload `Task::perform`.

**Hardware layer** (`src/hardware/`):
- `packet_format.rs` — the *only* place for protocol constants/offsets/timing structs. Do not redefine them locally.
- `worker/mod.rs`: `WorkerState` bundles all worker mutable state. New worker state goes here, not as a new `mpsc` channel.

**UI views** (`src/ui/views/`):
- `mod.rs` exports `action_button`, `small_action_button`, `icon_button`, `toolbar_button`, `icon_action_button`, `section_header` — use these instead of building `button(...)` raw.
- `header.rs::sync_toolbar_button()` — the toolbar Read/Write/Disconnect pattern.
- `bands.rs::render_band_row()` delegates to focused sub-functions (`render_freq_cell`, `render_gain_cell`, `render_q_cell`); keep that pattern when adding columns.
- `tools_panel.rs::tab_button()` — the tab-strip pattern.

## Adding New Components

### New view component
- New file under `src/ui/views/`, named after its visual responsibility.
- Pure function: `pub fn view_X(state: &MainWindow) -> Element<'_, Message>`.
- No mutable state, no side effects.
- Styling goes through `theme::*` functions; spacing/typography from `tokens::*`. Don't inline literal colors or pixel values.

### New device
1. Implement `DeviceProtocol` trait in `src/hardware/protocol.rs`.
2. Register the device in `src/models/device.rs`.
3. Follow the contributor guide comments at the bottom of `device.rs`.
4. Add protocol tests in `tests/protocol.rs` validating packet build/parse.

**[Constraints & Narrowing (Anti-Patterns)]**

| Anti-pattern | Instead (Strict Requirement) |
|---|---|
| Add fields to `EditorState` top level | Place in `data`, `session`, or `ui` |
| Manipulate `undo_stack` / `redo_stack` directly | Call `editor_state.push_undo()` |
| Run HID I/O on the UI thread | Route through worker via message protocol |
| Add error handling for "can't happen" cases | Trust internal contracts; validate only at system boundaries |
| Redefine protocol constants locally | Use `packet_format.rs` |
| Inline pixel values or colors in views | Use `tokens::*` and `theme::*` |
| Add backwards-compatibility shims | Change the code and update callers |
| Leave `// removed X` comments or unused `_var` | Delete cleanly |
| Leave `TODO` / `FIXME` / incomplete stubs | Provide complete, working code |

## Testing & Quality

- **70 tests total:** 54 unit tests (inline `#[cfg(test)]`) + 16 integration tests.
- Run: `cargo test --all-targets`.
- **Protocol tests** (`tests/protocol.rs`) validate packet construction/parsing for TP35Pro.
- **Token consistency tests** (`tests/token_consistency.rs`) ensure UI design tokens match the design system (WCAG AA contrast enforced).
- **Worker/IPC tests** (`tests/worker_ipc.rs`) cover serialization roundtrips, version handshake, error handling.
- **State unit tests** (`src/ui/state.rs`) cover `EditorState::push_undo` invariants — model for testing future state methods.

When adding a new module method that touches `EditorState` shape, add a unit test alongside it.

## Debugging & Troubleshooting

| Symptom | Approach |
|---|---|
| App crash or panic | Run with `RUST_BACKTRACE=1 cargo run` |
| HID device not detected | Check `dmesg | grep hid`, verify udev rules, ensure device is plugged in before launch |
| Permission denied on Linux | The app uses `pkexec` for elevated HID access — ensure polkit is running |
| IPC timeout or worker failure | Run with `RUST_LOG=debug cargo run` to trace worker lifecycle |
| Graph rendering issues | Check `src/ui/graph.rs` — verify filter values are within ±10 dB bounds |
| Profile load failure | Validate AutoEQ format in `src/autoeq.rs`; check file encoding (UTF-8 required) |

## Security Guidelines

- **No `unsafe` code** anywhere in the codebase.
- **HID trust boundary:** Device responses are untrusted input. Validate all hardware responses before applying to state.
- **Profile files:** Treat as untrusted input. Parse defensively; reject malformed AutoEQ files with clear errors.
- **Privilege escalation:** `pkexec` runs the binary itself as root only for HID access. The helper process drops privileges after opening the device. Do not expand the elevated scope.
- **No secrets in code:** Never commit API keys, credentials, or device-specific tokens.

## Contribution Workflow

- **Branch naming:** `feature/description`, `fix/description`, `chore/description`
- **Commit messages:** Conventional commits (`feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`, `style:`, `perf:`). Scope when useful: `feat(settings): ...`, `chore(packaging): ...`.
- **Before pushing:** Run the [Pre-Push Checklist](#pre-push-checklist-non-negotiable). All four steps must pass — they mirror `.github/workflows/ci.yml`.
- **Code review:** All PRs require review. Keep PRs focused on a single concern.
- **After modifying code:** Run `graphify update .` to keep the knowledge graph current (AST-only, no API cost).

## Essential Commands

```bash
# Day-to-day
cargo check --all-targets                    # fast build check
cargo run --release                          # launch the app
cargo fmt --all                              # apply formatting

# Pre-push (see Pre-Push Checklist for context — these mirror CI)
cargo fmt --all -- --check                   # CI: format check
cargo clippy --all-targets -- -D warnings    # CI: lint (warnings → errors)
cargo build --all-targets --locked           # CI: build with locked deps
cargo test --all-targets --locked            # CI: 70 tests

# Knowledge graph
graphify query "<question>"                  # query codebase context
graphify path "<A>" "<B>"                    # find relationships
graphify explain "<concept>"                 # explain a focused concept
graphify update .                            # refresh AST graph (AST-only, no API cost)
```

## Cutting a Release

Releases are automated via `.github/workflows/release.yml`, which fires on any pushed tag matching `v*.*.*`.

1. **Pick version:** Patch bump for fixes (`0.8.4` → `0.8.5`), minor for features (`0.8.x` → `0.9.0`), major only on user request. Confirm if unsure.
2. **Update versions** in three places:
   - `Cargo.toml`: `version = "X.Y.Z"`
   - `Cargo.lock`: run `cargo check --quiet` to sync
   - `packaging/arch/PKGBUILD`: `pkgver=X.Y.Z`
3. **Commit:** `chore: bump version to X.Y.Z` (stage `Cargo.toml`, `Cargo.lock`, `packaging/arch/PKGBUILD`)
4. **Tag:** `git tag vX.Y.Z`
5. **Push:** `git push origin main && git push origin vX.Y.Z`
6. **Verify:** `gh run watch` or `gh release view vX.Y.Z`

**Do not** create the GitHub release manually — the workflow does it. **Do not** push the tag before the bump commit.

If the user says "do release" with no version, default to a patch bump and confirm before pushing.

## Glossary

| Term | Definition |
|---|---|
| **AutoEQ** | Open-source headphone EQ profile format; plain text with frequency/gain pairs |
| **Biquad** | Second-order IIR filter; the DSP building block for parametric EQ bands |
| **HID** | Human Interface Device — USB class used for DAC communication |
| **IPC** | Inter-Process Communication; used between main app and elevated helper |
| **Push** | Write EQ state from app to device |
| **Pull** | Read EQ state from device to app |
| **Read-back** | Re-read device state after a push to verify write succeeded |
| **Rollback** | Restore previous state if read-back verification fails |
| **pkexec** | Polkit utility for privilege escalation on Linux |
| **WorkerState** | Background thread struct that owns all HID I/O state |
