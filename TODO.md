# Frost-Tune Parity TODO

Goal: bring Frost-Tune to feature parity with `../tp35pro-eq` while preserving project safety rules in `AGENTS.md`.

## Phase 0 - Foundations and Blockers

- [x] Build a real Iced app shell and wire UI actions to `UsbWorker` commands.
  - Files: `src/main.rs`, `src/ui/mod.rs`, `src/ui/main_window.rs`, `src/hardware/worker.rs`
  - Done when: Connect/Pull/Push are triggered from UI and reflected in UI state.

- [x] Introduce structured app state for device, editor, profiles, settings, and logs.
  - Files: `src/ui/main_window.rs`, `src/models.rs` (plus new `src/ui/*` state modules if needed)
  - Done when: state transitions are explicit and no HID call happens outside worker thread.

- [x] Harden transactional push semantics in worker and HID layer.
  - Files: `src/hardware/worker.rs`, `src/hardware/hid.rs`
  - Tasks:
    - Fix pull retry loop to truly retry.
    - Compare all critical fields on verify (`enabled/freq/gain/q/type/global_gain`).
    - Use backoff delays for verify retries.
    - Distinguish verify-failure vs rollback-failure errors.

- [x] Centralize safety validation before write.
  - Files: `src/models.rs`, `src/hardware/worker.rs`, `src/error.rs`
  - Rules:
    - Max band gain: `+10 dB`
    - Max global preamp: `+10 dB`
    - Validate filter count/range before HID write.

- [x] Add multi-DAC-ready protocol abstraction.
  - Files: `src/models.rs`, `src/hardware/protocol.rs`, `src/hardware/hid.rs`, `src/hardware/packet_builder.rs`, `src/hardware/worker.rs`
  - Done when: detection resolves to `Device::TP35Pro` and protocol routing is abstraction-based.

## Phase 1 - Core Parity

- [x] Implement connection lifecycle parity (connect, auto-connect, disconnect, hotplug).
  - Files: `src/hardware/worker.rs`, `src/ui/main_window.rs`, `src/models.rs`

- [x] Implement full PEQ editor workflow (pull -> edit -> push -> verify).
  - Files: `src/ui/main_window.rs`, `src/models.rs`, `src/hardware/worker.rs`
  - Done when: UI exposes 10 bands + preamp and reflects post-verify device state.

- [x] Add profile persistence CRUD and active-profile tracking.
  - Files: `src/ui/main_window.rs`, `src/models.rs`, `src/*` (new storage module)
  - Note: Not in v1 scope per user decision

- [ ] Add AutoEQ import/export and normalization.
  - Files: `src/models.rs`, `src/*` (new parser/serializer module), `src/ui/main_window.rs`

- [x] Add bypass and safe-flat behavior.
  - Files: `src/models.rs`, `src/ui/main_window.rs`, `src/hardware/worker.rs`
  - Note: User requested removal

## Phase 2 - Robustness and Polish

- [x] Replace stringly errors with a structured error taxonomy and user-facing mapping.
  - Files: `src/error.rs`, `src/hardware/worker.rs`, `src/ui/main_window.rs`

- [ ] Add diagnostics pipeline and log export/copy workflow.
  - Files: `src/main.rs`, `src/ui/main_window.rs`, `src/*` (new diagnostics module)

- [ ] Port reliability tools (`stress_push_pull`, timing benchmark) as Rust bins.
  - Files: `src/bin/*`, `src/hardware/hid.rs`, `src/hardware/worker.rs`

- [x] Add tests for safety-critical and parser logic.
  - Files: `src/hardware/worker.rs`, `src/hardware/packet_builder.rs`, `src/hardware/dsp.rs`, `tests/*`

## Open Decisions (ANSWERED)

- [x] Out-of-range behavior: **auto-clamp** (values above +10dB are clamped before push).
- [x] Auto-connect default: **off** (manual connect required on startup).
- [x] Parity scope for first milestone: **live hardware editing only** (no profile system in v1).
- [x] Profile storage location: N/A (not implemented in v1).
- [x] Multi-DAC near-term scope: **TP35Pro abstraction only** (ready for more, but only TP35Pro for now).
