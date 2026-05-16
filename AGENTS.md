# Frost-Tune — Developer Guidelines

## Project Overview

Frost-Tune is a native parametric EQ editor for USB DACs, built with Rust and the Iced GUI framework. It communicates with DACs over USB HID to adjust 10-band parametric EQ directly on hardware.

- **Version:** 0.8.5
- **Tech stack:** Rust 2021, Iced 0.14 (GUI), hidapi (HID I/O), tokio (async), serde/serde_json (serialization)
- **Target platforms:** Linux (primary), Windows
- **Status:** Actively maintained, CLI + GUI releases on Arch Linux AUR
- **GUI framework decision:** Iced. libcosmic was evaluated and rejected (Linux-only blocker; see `~/.claude/plans/` if revisiting).

## AI Coding Guidelines

**Role:** You are an expert Rust developer specializing in the Iced GUI framework and low-level USB HID interactions. Your primary focus is writing robust, safe, and highly maintainable code.

When assisting with this codebase, you must strictly adhere to the following core mindsets:

- **Maintainability First:** Write code that is easy to read, understand, and modify.
  - **No Placeholders:** Never leave `TODO`s, `FIXME`s, or incomplete implementation stubs. Provide complete, working code.
  - **Delete Dead Code:** Do not leave commented-out code (e.g., `// removed X`). Delete it cleanly.
  - **Reusability:** Always use existing UI helpers (e.g., `action_button`, `section_header` in `src/ui/views/mod.rs`) and state management methods (e.g., `push_undo()`). Do not reinvent them.
  - **Clarity over Cleverness:** Favor clear naming and modularity over obscure "clever" one-liners.

- **Scalability in Mind:** Consider how the application performs and adapts as it grows.
  - **Non-blocking UI:** NEVER perform blocking I/O (like HID communication or disk access) on the main UI thread. All I/O must route through the `WorkerState` IPC pipeline.
  - **State Segregation:** Strictly follow the decomposed state patterns. Never add fields to the top-level `EditorState`. Always categorize new state into the appropriate bucket (`data`, `session`, or `ui`).

- **Minimalism:** Strive for the simplest solution that works.
  - **No Unnecessary Dependencies:** Do not add new crates or dependencies to `Cargo.toml` unless explicitly requested by the user.
  - **Avoid Over-engineering:** Do not create unnecessary abstractions or deeply nested traits. Prefer simple, functional data flows (the Iced Elm architecture).

### Pre-Implementation Checklist
Before writing code or finalizing a task, ensure you have:
1. [ ] Checked `graphify-out/GRAPH_REPORT.md` (if it exists) to understand the codebase context.
2. [ ] Verified that any new UI component is a pure function with no mutable state or side effects.
3. [ ] Confirmed that any new state is placed in the correct `EditorState` sub-struct.
4. [ ] Ensured all errors are handled uniformly using `AppError` and the `thiserror` crate.
5. [ ] Verified that no formatting or linting regressions will be introduced (`cargo fmt`, `cargo clippy`).

---

## Claude Code Configuration

### graphify

This project has a knowledge graph at `graphify-out/` with god nodes, community structure, and cross-file relationships.

**Rules:**
- ALWAYS read `graphify-out/GRAPH_REPORT.md` before reading source files, running grep/glob, or answering codebase questions. The graph is the primary map of the codebase.
- IF `graphify-out/wiki/index.md` exists, navigate it instead of reading raw files.
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse EXTRACTED + INFERRED edges instead of scanning files.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

### Pull-request / commit conventions

- Use conventional-commit style prefixes (`feat:`, `fix:`, `refactor:`, `docs:`, `ci:`).
- Keep commit titles under 70 characters; put detail in the body.
- For multi-step refactors, prefer one cohesive commit per logically-coupled change rather than per-file splits.
- Co-author trailer for LLM-assisted commits is OK and expected.

---

## Architecture

```
frost-tune/
├── PKGBUILD                 # Arch Linux PKGBUILD (at repo root for easier installs)
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── autoeq.rs            # AutoEQ profile format
│   ├── diagnostics.rs       # Device diagnostics
│   ├── error.rs             # AppError + ErrorKind (thiserror)
│   ├── storage.rs           # Profile persistence
│   ├── hardware/            # HID / protocol layer
│   │   ├── dsp.rs           # Biquad filter computation
│   │   ├── elevated_transport.rs  # Linux privilege escalation
│   │   ├── helper_ipc.rs    # Helper process IPC
│   │   ├── helper_server.rs # Elevated helper server
│   │   ├── hid.rs           # HID transport
│   │   ├── operations.rs    # High-level hardware ops
│   │   ├── packet_builder.rs
│   │   ├── packet_format.rs # Packet constants, offsets, timing (single source of truth)
│   │   ├── pipeline.rs      # Read/write pipeline
│   │   ├── protocol.rs      # DeviceProtocol trait + TP35ProProtocol
│   │   └── worker/          # Background worker thread
│   │       ├── mod.rs       # WorkerState struct, UsbWorker
│   │       ├── backend.rs   # TransportBackend enum
│   │       ├── connection.rs
│   │       └── ops.rs       # Pull/push operations
│   ├── models/              # Domain types
│   │   ├── constants.rs     # EQ limits, ISO frequencies, band count
│   │   ├── device.rs        # Device definitions + registration
│   │   ├── filter.rs        # Filter model (with log-spaced default freqs)
│   │   └── ipc.rs           # IPC message types
│   └── ui/                  # Iced GUI
│       ├── graph.rs         # Frequency response canvas (EqGraph: Program)
│       ├── main_window.rs   # Window layout, subscription, bootstrap
│       ├── messages.rs      # Message enum (69 variants)
│       ├── state.rs         # MainWindow + EditorState (data/session/ui)
│       ├── theme.rs         # Tokyo Night styling, 15 style fns
│       ├── tokens.rs        # Design tokens (spacing, type, radii, icon font)
│       ├── update/          # Message handlers
│       │   ├── autoeq.rs
│       │   ├── connection.rs
│       │   ├── editor.rs
│       │   ├── hardware.rs
│       │   ├── mod.rs       # Dispatches Message → handler
│       │   └── profiles.rs
│       └── views/           # UI view components
│           ├── bands.rs
│           ├── confirm_dialog.rs
│           ├── diagnostics.rs
│           ├── graph_panel.rs
│           ├── header.rs
│           ├── mod.rs       # Shared button helpers
│           ├── preamp.rs
│           ├── status_banner.rs
│           └── tools_panel.rs
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
cargo clippy --all-targets       # Lint (target: 0 new warnings)
cargo test --all-targets         # Run all 70+ tests
cargo check --all-targets        # Fast build check
cargo run --release              # Start the app with optimizations

# Knowledge graph
graphify update .                # Refresh AST graph after code changes (AST-only, no API cost)

# Package for Arch Linux
makepkg -si
```

## Cutting a release

Releases are automated via `.github/workflows/release.yml`, which fires on any pushed tag matching `v*.*.*`. The workflow auto-syncs version files from the tag, builds `.deb` + `.rpm` artifacts on Linux/Windows runners, and publishes a GitHub Release.

**Steps for the LLM to follow when the user says "do a release" / "cut a release" / "release":**

1. Pick the new version. Patch bump (e.g. `0.8.4` → `0.8.5`) for fixes; minor (`0.8.x` → `0.9.0`) for features; major only on user request. Confirm with the user if unsure.
2. Update version in three places — keep them in lockstep:
   - `Cargo.toml`: `version = "X.Y.Z"`
   - `PKGBUILD`: `pkgver=X.Y.Z`
   - `Cargo.lock`: run `cargo check --quiet` after editing `Cargo.toml` so the lockfile picks up the new `frost-tune` version entry.
3. Commit: `chore: bump version to X.Y.Z` (stage only `Cargo.toml`, `Cargo.lock`, `PKGBUILD` — never staging dirs like `pkg/` or `*.tar.gz` artifacts).
4. Tag: `git tag vX.Y.Z` (annotated tag is fine but not required — the workflow only inspects the tag name).
5. Push both: `git push origin main && git push origin vX.Y.Z`.
6. The release workflow takes over. Verify with `gh run watch` or `gh release view vX.Y.Z`.

**Do not** create the GitHub release manually (`gh release create`) — the workflow does it. **Do not** push the tag before the bump commit; the workflow expects the tag's commit to already contain the bumped versions (it has a sync step that will commit a fix-up otherwise, but it's cleaner to bump first).

If the user just says "do release" with no version, default to a patch bump from the current `Cargo.toml` version and confirm before pushing the tag.

## Code Standards

- **Edition:** Rust 2021. No `unsafe` anywhere.
- **Comments:** None by default. Only when explaining *why* (hidden constraints, non-obvious invariants, workarounds). Never describe *what* the code does — names should.
- **Error handling:** Uniform `Result<T, AppError>` across ALL modules. `AppError` (`thiserror`) carries `kind: ErrorKind`, `message`, optional `context`. Defined in `error.rs`.
- **Async / threading:** Tokio runtime for background HID I/O; UI runs on main thread. HID I/O is always isolated on a worker thread (`std::thread` + `mpsc`) — never block the UI thread.
- **Writes:** Every EQ write follows push → read-back → verify → rollback.
- **Safety:** Band gain and global preamp capped at ±10 dB; bounds enforced via `Filter::clamp` and `PushPayload::clamp`.
- **Linting:** Zero clippy warnings in library code (excluding upstream `ashpd` dependency notice).
- **Formatting:** `cargo fmt --check` must pass. Run `cargo fmt --all` before commits.

## Maintainability & Scalability Principles

These rules exist to keep the codebase navigable as it grows. **Follow them by default**; deviate only with a clear reason.

### Adding state — pick the right bucket

`EditorState` is **deliberately decomposed** into three sub-structs. New fields go in the bucket that matches their lifetime, not at the top level:

| Sub-struct        | Holds                                                                                              | Examples                                                        |
|-------------------|----------------------------------------------------------------------------------------------------|-----------------------------------------------------------------|
| `data` (`EditorData`)       | The EQ itself — what gets saved to disk and pushed to hardware.                          | `filters`, `global_gain`                                        |
| `session` (`EditorSession`) | Transient per-session state — drafts, history, dialogs, banners. Never persisted.        | `input_buffer`, `undo_stack`/`redo_stack`, `pending_confirm`, `status_message`, `is_dirty`, `new_profile_name` |
| `ui` (`EditorUI`)           | UI cache + persistent preferences.                                                       | `profiles`, `selected_profile_name`, `profile_search`, `snap_to_iso_enabled`, `active_tools_tab` |

**Decision rule:** if it would survive a "reset session" but not a "factory reset," it's `ui`. If it would survive both, it's `data`. Otherwise it's `session`. **Never add a new field at the top of `EditorState`.**

### Adding a `Message` variant

1. Add the variant to `src/ui/messages.rs::Message`.
2. Route it in `src/ui/update/mod.rs` to the correct handler (`handle_connection`, `handle_hardware`, `handle_editor`, `handle_autoeq`, or `handle_profiles`). The router has no `_ =>` arm at the top level — every variant must be explicitly routed.
3. Implement the match arm in the handler.

### Adding a new view component

- New file under `src/ui/views/`, named after its visual responsibility.
- Pure function: `pub fn view_X(state: &MainWindow) -> Element<'_, Message>`.
- No mutable state, no side effects.
- Styling goes through `theme::*` functions; spacing/typography from `tokens::*`. Don't inline literal colors or pixel values.

### Adding a new device

1. Implement `DeviceProtocol` trait in `src/hardware/protocol.rs`.
2. Register the device in `src/models/device.rs`.
3. Follow the contributor guide comments at the bottom of `device.rs`.
4. Add protocol tests in `tests/protocol.rs` validating packet build/parse.

### Reuse existing helpers — don't reinvent

Before adding a new helper, check if one of these covers your case:

**State / domain:**
- `EditorState::push_undo()` — snapshots current `data`, pushes onto `undo_stack`, clears `redo_stack`, trims to `MAX_UNDO`. Use this; do not manipulate the stacks directly.

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

### Anti-patterns to avoid

- **Don't add fields directly to `EditorState`** — pick `data`, `session`, or `ui`.
- **Don't manipulate `undo_stack` / `redo_stack` directly** — call `editor_state.push_undo()`.
- **Don't run HID I/O on the UI thread** — go through the worker via the message protocol.
- **Don't add error handling for "can't happen" cases** — trust internal contracts. Only validate at system boundaries (user input, external APIs, hardware responses).
- **Don't redefine protocol constants** locally — they live in `packet_format.rs`.
- **Don't inline pixel values or colors** in views — use `tokens::*` and `theme::*`.
- **Don't add backwards-compatibility shims** — change the code and update callers.
- **Don't leave `// removed X` comments or unused `_var` placeholders** — delete cleanly.

## Testing & Quality

- **54 unit tests** (inline `#[cfg(test)]`) + **16 integration tests** = **70 total**.
- Run: `cargo test --all-targets`.
- **Protocol tests** (`tests/protocol.rs`) validate packet construction/parsing for TP35Pro.
- **Token consistency tests** (`tests/token_consistency.rs`) ensure UI design tokens match the design system (WCAG AA contrast enforced).
- **Worker/IPC tests** (`tests/worker_ipc.rs`) cover serialization roundtrips, version handshake, error handling.
- **State unit tests** (`src/ui/state.rs`) cover `EditorState::push_undo` invariants — model for testing future state methods.
- **Clippy:** Zero new warnings in library code. The `ashpd` dependency notice is upstream and not actionable.

When adding a new module method that touches `EditorState` shape, add a unit test alongside it — `EditorState::default()` is cheap to construct, so there's no excuse.

## Architecture Patterns

- **Iced Elm architecture:** State + Messages + Update + View pattern. View functions are pure; mutations live in `update/`.
- **DeviceProtocol trait:** Defines the HID packet protocol per device (build read/write packets, parse responses). One impl per device.
- **WorkerState pattern:** Background worker encapsulates mutable state behind a method (`run_iteration`), not loose function parameters.
- **AutoEQ format:** Profiles stored as plain text, compatible with the AutoEQ ecosystem.
- **Linux elevation:** `pkexec` re-runs the binary itself as a temporary helper; no system-wide install needed.
- **EditorState decomposition:** Domain (`data`) / session (`session`) / UI (`ui`) — see "Maintainability & Scalability Principles" above.
