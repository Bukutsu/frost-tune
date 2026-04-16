# UI Usability Refactor TODO

## Objective

Apply the UI review heuristics to improve information hierarchy, action clarity, readability, and advanced filter usability while preserving current functionality.

## Progress Tracker

- [x] 1. Rebalance layout so Diagnostics is secondary and graph gets more space
- [x] 2. Reduce visual clutter by softening/removing heavy section borders
- [x] 3. Implement button hierarchy (primary/secondary/tertiary)
- [x] 4. Demote Diagnostics "Clear" destructive emphasis unless useful
- [x] 5. Improve subtitle and graph label legibility/contrast
- [x] 6. Refactor advanced filters into table-like rows with static headers
- [x] 7. Tighten Gain control grouping (slider + value)
- [x] 8. Run `cargo build`
- [x] 9. Run `cargo test`

## Validation

- [x] Diagnostics no longer dominates initial viewport
- [x] Graph is larger and easier to read
- [x] Action hierarchy is visually clear
- [x] Reduced border noise across cards
- [x] Chart ticks and subtitle are more legible
- [x] Advanced filter rows scan faster with less repetition

## Post-Review Fixes

- [x] 10. Remove remaining duplicate controls in Advanced Filters (Freq/Q)
- [x] 11. Remove duplicate Diagnostics heading and improve empty-state behavior
- [x] 12. Prevent immediate duplicate diagnostics events for robustness
- [x] 13. Re-run `cargo build`
- [x] 14. Re-run `cargo test`

## Layout Tuning Pass

- [x] 15. Reduce empty horizontal space in advanced filter table
- [x] 16. Rebalance filter table column widths and row density
- [x] 17. Make diagnostics panel height adaptive to event count
- [x] 18. Reduce duplicate/noisy diagnostics status lines
- [x] 19. Re-run `cargo build`
- [x] 20. Re-run `cargo test`
