# Frost-Tune

Native parametric EQ editor for USB DACs. Pushes state directly to hardware via HID.
Rust 2021 · Iced 0.14 · hidapi · tokio. Linux primary, Windows secondary.

`CLAUDE.md` is a symlink to this file — edit one place.

## Pre-Push (mirrors `.github/workflows/ci.yml`)

Run **all four**. Skipping wastes a CI round-trip and a force-push to fix.

```bash
cargo fmt --all                              # apply, don't just check
cargo clippy --all-targets -- -D warnings
cargo build --all-targets --locked
cargo test --all-targets --locked
```

- `--locked` matters: if you edited `Cargo.toml`, run `cargo check` to sync `Cargo.lock` and commit it.
- Do not bypass with `--no-verify` or skip clippy. Fix the root cause.
- Re-run after rebase/merge — drift creeps back in.

## Hard Rules (project invariants — not enforceable by lint)

- **HID I/O never on the UI thread.** Route through the worker (`src/hardware/worker/`) via the message protocol.
- **Every write is push → read-back → verify → rollback.** On divergence, restore the prior state.
- **Band gain & global preamp clamp to ±10 dB.** Enforced by `Filter::clamp` and `PushPayload::clamp` — don't bypass at the call site.
- **Protocol constants live in `src/hardware/packet_format.rs` only.** Never redefine offsets/timing locally.
- **Device responses are untrusted input.** Validate before applying to state. Same for AutoEQ profile files.
- **No `unsafe`. Anywhere.**
- **`pkexec` scope is HID access only.** The elevated helper drops privileges after opening the device. Don't widen the scope.

## State Layout — `EditorState`

Decomposed into three buckets by lifetime. **New fields go in the matching bucket, never at the top level.**

| Bucket | Lifetime | Examples |
|---|---|---|
| `data` (`EditorData`) | Persistent EQ state | `filters`, `global_gain` |
| `session` (`EditorSession`) | Lost on "reset session" | `input_buffer`, undo/redo stacks, `pending_confirm`, `status_message`, `is_dirty` |
| `ui` (`EditorUI`) | Survives session reset, not factory reset | `profiles`, `selected_profile_name`, `snap_to_iso_enabled`, `auto_pull_on_connect` |

Decision rule: survives both resets → `data`; survives session reset only → `ui`; lost on session reset → `session`.

When adding a method that touches `EditorState` shape, add a unit test next to it. `EditorState::default()` is cheap.

## Message Routing

1. Add the variant to `src/ui/messages.rs::Message`.
2. Route it in `src/ui/update/mod.rs` to one of `handle_connection`, `handle_hardware`, `handle_editor`, `handle_autoeq`, `handle_profiles`. **The dispatcher has no `_ =>` arm** — every variant is routed explicitly.
3. Implement the arm in the handler.

## Reuse Catalog (check before writing new code)

| Need | Use |
|---|---|
| Push EQ change with undo | `EditorState::push_undo()` — snapshots `data`, pushes onto `undo_stack`, clears redo, trims to `MAX_UNDO`. **Don't touch stacks directly.** |
| Profile load/save | `storage::load_all_profiles`, `storage::save_profile` |
| App preferences persistence | `storage::load_settings` / `storage::save_settings(Settings)` — call after any preference toggle |
| "Is hardware busy?" guard | `hardware.rs::is_hw_busy()` |
| Band freq/gain/Q draft input | `editor.rs::handle_band_text_input`, `cancel_band_draft_input` |
| Reload profiles task | `profiles.rs::reload_profiles_task()` |
| New worker mutable state | Add to `WorkerState` in `worker/mod.rs` — **not** a new `mpsc` channel |
| Buttons | `ui/views/mod.rs::{action_button, small_action_button, icon_button, toolbar_button, icon_action_button, section_header}` — not raw `button(...)` |
| Toolbar Read/Write/Disconnect | `header.rs::sync_toolbar_button()` |
| Tab strip | `tools_panel.rs::tab_button()` |
| Band row rendering | `bands.rs::render_band_row` → `render_freq_cell`/`render_gain_cell`/`render_q_cell` — keep this split when adding columns |

## Design System — Industrial Utilitarian

Non-obvious aesthetic constraints. Violate these and the app stops feeling like hardware control software.

- **No rounded corners on interactive elements.** `SHAPE_EXTRA_SMALL` and `SHAPE_SMALL` are `0.0` on purpose.
- **No borders on panels or tables.** Establish hierarchy via background contrast (`SURFACE_0` vs `SURFACE_1`).
- **Monospace alignment in data cells.** Structural typography only — no decorative type.
- **No inline pixel values or colors in views.** Use `tokens::*` for spacing/typography, `theme::*` for styling.
- **Motion is sparse.** Background color swap on hover/press is the entire feedback vocabulary. No micro-animations.

New view component → pure function `pub fn view_X(state: &MainWindow) -> Element<'_, Message>` under `src/ui/views/`. No mutable state, no side effects.

## Adding a Device

1. Implement `DeviceProtocol` in `src/hardware/protocol.rs`.
2. Register in `src/models/device.rs` (contributor notes at the bottom of that file).
3. Add packet build/parse tests in `tests/protocol.rs`.

## Cutting a Release

Tag push matching `v*.*.*` fires `.github/workflows/release.yml`. **Do not create the GitHub release manually.**

1. Pick version: patch for fixes, minor for features, major only on user request.
2. Bump three files: `Cargo.toml` (`version`), `Cargo.lock` (run `cargo check` to sync), `packaging/arch/PKGBUILD` (`pkgver`).
3. Commit `chore: bump version to X.Y.Z`.
4. `git tag vX.Y.Z && git push origin main && git push origin vX.Y.Z`.
5. Watch `gh run watch`. Don't push the tag before the bump commit.

If the user says "do release" with no version, default to patch bump and confirm.

## Local Tooling

- Commit style: Conventional Commits (`feat:`, `fix:`, `chore:`, scope when useful).
