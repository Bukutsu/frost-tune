# Robustness Hardening TODO

## Objective

Improve runtime reliability, transaction safety, diagnostics correctness, and CI robustness while preserving current architecture (UI thread + worker HID thread).

## Progress Tracker

### Quick wins (execute now)

- [x] 1. Surface rollback failure details in worker push path (`src/hardware/worker.rs`)
- [x] 2. Verify rollback by read-back compare after rollback (`src/hardware/worker.rs`)
- [x] 3. Centralize app version from Cargo metadata and remove hardcoded versions (`src/main.rs`, `src/ui/main_window.rs`)
- [x] 4. Strengthen CI gate to run all-target checks and tests (`.github/workflows/ci.yml`)
- [x] 5. Verify with `cargo build`
- [x] 6. Verify with `cargo test`

### Medium improvements (1-2 weeks)

- [ ] 7. Introduce typed hardware error kinds through worker/UI flow
- [ ] 8. Move storage paths to user app-data directories (Linux/Windows-safe)
- [ ] 9. Implement atomic writes for profiles/preferences/diagnostics
- [ ] 10. Formalize worker connection state machine and transition tests
- [ ] 11. Upgrade retry policy (exponential backoff + jitter + transient/permanent error classes)
- [ ] 12. Enrich diagnostics context (operation id, attempt, elapsed ms, VID/PID)

### Long-term hardening

- [ ] 13. Add transport abstraction + mock HID integration tests
- [ ] 14. Add fuzz/property tests for AutoEQ parser and packet parsing
- [ ] 15. Add release correctness checks (tag/version consistency, checksum validation)
- [ ] 16. Add security baseline checks (`cargo audit`, stricter clippy policy)

## Validation Criteria

- [x] Rollback failures are visible and actionable in UI/diagnostics
- [x] Rollback path verifies restored snapshot state
- [x] Diagnostics/version output matches `CARGO_PKG_VERSION`
- [x] CI blocks regressions with checks + tests

## Execution Notes

- Quick wins executed and validated locally.
- `cargo build`: pass
- `cargo test`: pass
