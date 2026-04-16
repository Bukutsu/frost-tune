# Material 3 Compliance Execution TODO

## Objective

Align Frost-Tune with Material Design 3 principles while preserving pro-grade EQ precision and keeping implementation simple, stable, and testable in Rust + Iced.

## Phase 1: Foundation (Tokens + Theme-aware Structure)

- [x] Define and use semantic surface hierarchy with softened container borders (`src/ui/theme.rs`)
- [x] Reduce visual noise through subtle elevation and outline treatment (`src/ui/theme.rs`)
- [x] Formalize M3 token groups (color, state, spacing, typography) in a dedicated token module
- [ ] Add explicit component style roles (primary/secondary/tertiary/destructive) for all button families

## Phase 2: Adaptive Layout (Window Size Classes)

- [x] Keep graph in all layout buckets and prioritize core flow (`src/ui/main_window.rs`)
- [x] Increase graph size responsively for medium/expanded widths (`src/ui/main_window.rs`)
- [x] Keep diagnostics as secondary content below advanced controls (`src/ui/main_window.rs`)
- [x] Re-introduce expanded breakpoint with side-sheet diagnostics pattern (desktop only)
- [ ] Add compact breakpoint behavior for dense controls (narrow width ergonomics)

## Phase 3: High-Precision Audio UX

- [x] Convert advanced controls to headered table-like structure (`src/ui/main_window.rs`)
- [x] Remove duplicate per-row unit labels and redundant controls for Freq/Q (`src/ui/main_window.rs`)
- [x] Tighten gain/value grouping and row density (`src/ui/main_window.rs`)
- [ ] Add fine-tune interaction model (Shift modifier / step scaling)
- [ ] Add row-level state (clean/dirty/invalid) with inline validation hints

## Phase 4: Graph + Data Visualization Styling

- [x] Improve axis text legibility (size and contrast) (`src/ui/graph.rs`)
- [ ] Apply M3 chart styling contract (grid alpha tiers, curve emphasis tiers)
- [ ] Add optional peak/clipping visual indicators tied to gain/preamp limits
- [ ] Add accessible textual summary for graph state in diagnostics/export output

## Phase 5: Diagnostics Robustness + Containment

- [x] Remove duplicate diagnostics heading and improve empty state (`src/ui/main_window.rs`)
- [x] Make diagnostics area adaptive to event volume (`src/ui/main_window.rs`)
- [x] Demote diagnostics actions to tertiary style where appropriate (`src/ui/main_window.rs`)
- [x] Suppress immediate duplicate events at store level (`src/diagnostics.rs`)
- [ ] Add grouped duplicate count (e.g. repeated poll events as xN)
- [ ] Add severity chips (All/Warn/Error) and persistent preference

## Phase 6: Accessibility (WCAG + Keyboard + SR)

- [x] Improve low-contrast workflow subtitle readability (`src/ui/main_window.rs`)
- [ ] Run explicit WCAG contrast validation against all text roles
- [ ] Add deterministic keyboard traversal and visible focus ring conventions
- [ ] Add screen-reader oriented labels/summaries for graph-driven controls

## Validation Gates

- [x] `cargo build`
- [x] `cargo test`
- [x] Manual visual QA at 1024x700, 1366x768, 1920x1080
- [ ] Keyboard-only workflow QA (connect/read/import/edit/write)
- [ ] Contrast audit pass for caption/muted/chart labels

## Notes

- Completed items above were executed in this session as part of the UI usability and post-review tuning passes.
- Remaining items are the next implementation tranche for full M3 compliance and accessibility hardening.
