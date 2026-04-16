# AutoEQ-First UI Refactor TODO

## Objective

Prioritize AutoEQ workflows, hide manual filter editing for advanced users, make dense sections fit standard laptop/full HD windows better, and remove awkward copy.

## Progress Tracker

- [x] 1. Create UI preference persistence (`advanced_filters_expanded`, `diagnostics_expanded`) in `src/storage.rs`
- [x] 2. Add new UI messages for toggle/load/save flow in `src/ui/messages.rs`
- [x] 3. Extend UI state with persisted visibility flags in `src/ui/state.rs`
- [x] 4. Wire startup loading + save-on-toggle logic in `src/ui/main_window.rs`
- [x] 5. Remove header subtitle text `COSMIC-inspired controls`
- [x] 6. Reorder layout so AutoEQ is prioritized above advanced filter editing
- [x] 7. Implement collapsible `Advanced filter controls` section (collapsed by default)
- [x] 8. Implement collapsible `Diagnostics` section (collapsed by default)
- [x] 9. Add responsive width-class behavior for compact/medium/large windows
- [x] 10. Verify build with `cargo build`
- [x] 11. Verify tests with `cargo test`

## Validation Criteria

- [x] First launch defaults to collapsed advanced filters and diagnostics
- [x] Toggle states persist across restarts
- [x] At 1366x768, primary actions remain usable without clipping critical controls
- [x] At 1920x1080, layout uses available space better

## Execution Notes

- `cargo build`: pass
- `cargo test`: pass (all suites)
- Added responsive width-bucket composition (`Narrow`/`Medium`/`Wide`) and coverage tests now pass
