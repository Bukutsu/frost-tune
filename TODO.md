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

- [x] Add AutoEQ import/export and normalization.
  - Files: `src/models.rs`, `src/autoeq.rs`, `src/ui/main_window.rs`
  - Improvements: Suffix-aware frequency parsing, line-level error reporting, native file pickers.

- [x] Add diagnostics pipeline and log export/copy workflow.
  - Files: `src/main.rs`, `src/ui/main_window.rs`, `src/diagnostics.rs`
  - Improvements: Errors-only filter, export to timestamped file.

- [x] Port reliability tools (`stress_push_pull`, timing benchmark) as Rust bins.
  - Note: Skipped per user decision.

- [x] Add tests for safety-critical and parser logic.
  - Files: `src/hardware/worker.rs`, `src/hardware/packet_builder.rs`, `src/hardware/dsp.rs`, `tests/*`

## Phase 2 - Robustness & UI Hardening

- [x] **Critical: Fix Rollback Logic**
  - Files: `src/hardware/worker.rs`
  - Task: Ensure that a `Err` during the verification read phase triggers an immediate rollback to the previous known-good snapshot.
  - Done when: Verified by a unit/integration test simulating a read failure during push.

- [x] **Safety: Align Value Clamps & Finite Validation**
  - Files: `src/autoeq.rs`, `src/models.rs`, `src/ui/main_window.rs`
  - Tasks:
    - Update `src/autoeq.rs` to clamp preamp to `±10.0` (matching `MAX_GLOBAL_GAIN`).
    - Implement `is_finite()` checks in `Filter::is_valid()` and `PEQData::is_valid()` to reject `NaN`/`inf`.
    - Centralize shared constants for min/max gain, Q, and frequency to prevent drift.

- [x] **Reliability: Improve HID Transport Feedback**
  - Files: `src/hardware/hid.rs`, `src/hardware/worker.rs`
  - Tasks:
    - Stop returning `Ok(0)` on global gain timeout; return a proper `TransportError::Timeout`.
    - Fix the "dead" retry heuristic (`has_no_filters` check) in the worker pull path.
    - Replace production `unwrap()` calls in worker serialization with safe error handling.

- [x] **UI: Modernize Hierarchy & Feedback (M3/Material You)**
  - Files: `src/ui/main_window.rs`, `src/ui/theme.rs`
  - Tasks:
    - Implement explicit loading/progress states (e.g., `ProgressBar` or `Spinner`) for Push/Pull/Connect operations.
    - Add operation-result "Toasts" or Banners with severity levels (Success/Error/Warning).
    - Refactor band editor rows for better typography hierarchy and larger touch targets.
    - Implement responsive layout: stack band controls into cards or columns when window width is narrow.

## Phase 3 - Refactoring & Multi-Device Evolution

- [x] **Architecture: Split MainWindow Monolith**
  - Files: `src/ui/main_window.rs`, `src/ui/state.rs`, `src/ui/messages.rs`
  - Task: Extract update logic into domain-specific modules (e.g., `ui/update/transport.rs`, `ui/update/editor.rs`, `ui/update/storage.rs`).
  - Task: Extract view components into a `ui/components/` directory.
  - Status: Partially done - created `ui/state.rs` and `ui/messages.rs` modules.

- [x] **Structure: Establish Library Boundary**
  - Files: `src/main.rs`, `src/lib.rs`
  - Task: Create `src/lib.rs` and move core modules (hardware, models, dsp) there to separate CLI/Core logic from the UI launcher.

- [x] **Multi-Device: Protocol Abstraction Layer**
  - Files: `src/hardware/protocol.rs`, `src/hardware/packet_builder.rs`
  - Task: Introduce a `DeviceProtocol` trait to decouple packet building from specific hardware offsets, allowing easier onboarding of non-TP35Pro DACs.

- [ ] **Testing: Transactional Integration Suite**
  - Files: `tests/worker_tests.rs`
  - Task: Add integration tests for the `UsbWorker` using a mock HID transport to verify:
    - Transactional write/verify/rollback flows.
    - Hotplug auto-reconnect debounce and failure modes.

## Open Decisions (ANSWERED)

- [x] Out-of-range behavior: **auto-clamp** (values above +10dB are clamped before push).
- [x] Auto-connect default: **off** (manual connect required on startup).
- [x] Parity scope for first milestone: **live hardware editing only** (no profile system in v1).
- [x] Profile storage location: N/A (not implemented in v1).
- [x] Multi-DAC near-term scope: **TP35Pro abstraction only** (ready for more, but only TP35Pro for now).
