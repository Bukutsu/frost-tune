# Graph Report - frost-tune  (2026-05-18)

## Corpus Check
- 57 files · ~45,909 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 550 nodes · 802 edges · 44 communities (25 shown, 19 thin omitted)
- Extraction: 85% EXTRACTED · 15% INFERRED · 0% AMBIGUOUS · INFERRED: 121 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `6715b9d1`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Profile & Connection Management|Profile & Connection Management]]
- [[_COMMUNITY_Main Window UI|Main Window UI]]
- [[_COMMUNITY_Hardware Worker Server|Hardware Worker Server]]
- [[_COMMUNITY_HID Operations Pipeline|HID Operations Pipeline]]
- [[_COMMUNITY_Theme & UI Tokens|Theme & UI Tokens]]
- [[_COMMUNITY_Editor State Buffers|Editor State Buffers]]
- [[_COMMUNITY_DSP Math & Graph Drawing|DSP Math & Graph Drawing]]
- [[_COMMUNITY_Hardware Protocol|Hardware Protocol]]
- [[_COMMUNITY_AutoEQ Format Parsing|AutoEQ Format Parsing]]
- [[_COMMUNITY_Diagnostics System|Diagnostics System]]
- [[_COMMUNITY_Elevated Errors Transport|Elevated Errors Transport]]
- [[_COMMUNITY_Domain Rules Snapping|Domain Rules Snapping]]
- [[_COMMUNITY_Filter Model Input|Filter Model Input]]
- [[_COMMUNITY_IPC Payload Validation|IPC Payload Validation]]
- [[_COMMUNITY_Header & Status UI|Header & Status UI]]
- [[_COMMUNITY_Protocol Timing Specs|Protocol Timing Specs]]
- [[_COMMUNITY_Message Routing Types|Message Routing Types]]
- [[_COMMUNITY_Transport Backend|Transport Backend]]
- [[_COMMUNITY_Helper IPC Requests|Helper IPC Requests]]
- [[_COMMUNITY_Diagnostics UI View|Diagnostics UI View]]
- [[_COMMUNITY_Byte Conversion Utils|Byte Conversion Utils]]
- [[_COMMUNITY_Device Registry|Device Registry]]
- [[_COMMUNITY_Project README|Project README]]
- [[_COMMUNITY_App Entrypoint|App Entrypoint]]
- [[_COMMUNITY_Cache Warm Workflow|Cache Warm Workflow]]
- [[_COMMUNITY_CI Workflow|CI Workflow]]
- [[_COMMUNITY_Cargo Fmt Check|Cargo Fmt Check]]
- [[_COMMUNITY_Cargo Clippy Check|Cargo Clippy Check]]
- [[_COMMUNITY_Cargo Test Action|Cargo Test Action]]
- [[_COMMUNITY_Release Workflow|Release Workflow]]
- [[_COMMUNITY_Frost Tune Logo|Frost Tune Logo]]
- [[_COMMUNITY_App Screenshot|App Screenshot]]
- [[_COMMUNITY_UI Graph Panel|UI Graph Panel]]
- [[_COMMUNITY_UI Band Controls|UI Band Controls]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]

## God Nodes (most connected - your core abstractions)
1. `MainWindow` - 25 edges
2. `Frost-Tune — Developer Guidelines` - 18 edges
3. `parse_autoeq_text()` - 16 edges
4. `Frost-Tune — Agent Guidelines` - 16 edges
5. `Frost-Tune — Agent Guidelines` - 16 edges
6. `Frost-Tune — Agent Guidelines` - 16 edges
7. `TP35ProProtocol` - 15 edges
8. `enforce_disabled_button_contrast()` - 14 edges
9. `Frost-Tune` - 14 edges
10. `push_with_verify()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `test_disabled_button_contrast_wcag_aa()` --calls--> `theme()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `m3_tonal_button()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `pill_secondary_button()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `run_with_diagnostics()` --calls--> `parse_diagnostic_log_line()`  [INFERRED]
  src/ui/main_window.rs → src/diagnostics.rs
- `handle_editor()` --calls--> `parse_freq_string()`  [INFERRED]
  src/ui/update/editor.rs → src/ui/main_window.rs

## Communities (44 total, 19 thin omitted)

### Community 0 - "Profile & Connection Management"
Cohesion: 0.09
Nodes (38): peq_to_autoeq(), test_peq_to_autoeq_format(), main(), append_diagnostics_log(), delete_profile(), export_profile(), get_base_dir(), get_diagnostics_log_path() (+30 more)

### Community 1 - "Main Window UI"
Cohesion: 0.07
Nodes (11): layout_bucket_for_width(), LayoutBucket, MainWindow, parse_freq_string(), run(), run_with_diagnostics(), dialog_container(), view_confirm_dialog() (+3 more)

### Community 2 - "Hardware Worker Server"
Cohesion: 0.06
Nodes (29): CommandSpec, ElevatedTransport, spawn_via_pkexec(), validate_pkexec_target(), handle_connect(), pull_logic(), push_logic(), require_device() (+21 more)

### Community 3 - "HID Operations Pipeline"
Cohesion: 0.14
Nodes (23): assemble_filters(), delay_ms(), flush_hid_buffer(), get_next_nonce(), pull_peq_internal(), read_global_gain(), read_single_filter_with_nonce(), reset_nonce() (+15 more)

### Community 4 - "Theme & UI Tokens"
Cohesion: 0.07
Nodes (32): contrast_ratio(), linear_channel(), test_disabled_button_contrast_wcag_aa(), enforce_disabled_button_contrast(), gain_color(), gain_slider_style(), m3_filled_button(), m3_filled_button_error() (+24 more)

### Community 5 - "Editor State Buffers"
Cohesion: 0.1
Nodes (14): ConfirmAction, ConnectionStatus, DisconnectReason, DraftFilter, EditorData, EditorSession, EditorState, EditorUI (+6 more)

### Community 6 - "DSP Math & Graph Drawing"
Cohesion: 0.14
Nodes (20): calculate_total_response(), compute_iir_filter(), convert_to_byte_array(), get_biquad_coefficients(), get_magnitude_response(), get_magnitude_response_with_coeffs(), parse_filter_packet(), quantizer() (+12 more)

### Community 7 - "Hardware Protocol"
Cohesion: 0.11
Nodes (6): DeviceProtocol, test_tp35pro_build_filter_write_packet(), test_tp35pro_build_flash_eq_packet(), test_tp35pro_build_global_gain_write_packet(), test_tp35pro_build_temp_write_packet(), TP35ProProtocol

### Community 8 - "AutoEQ Format Parsing"
Cohesion: 0.24
Nodes (17): contains_token(), extract_fc_value(), extract_gain_value(), extract_number(), extract_number_after(), extract_q_value(), parse_autoeq_text(), parse_filter_line() (+9 more)

### Community 9 - "Diagnostics System"
Cohesion: 0.13
Nodes (6): DiagnosticEvent, DiagnosticsStore, format_diagnostics(), LogLevel, parse_diagnostic_log_line(), Source

### Community 10 - "Elevated Errors Transport"
Cohesion: 0.12
Nodes (25): Adding New Components, Anti-Patterns, Architecture, Architecture Patterns, Code Standards, Code Standards (Rust-Pro Guidelines), code:block1 (frost-tune/), code:bash (# Development workflow) (+17 more)

### Community 11 - "Domain Rules Snapping"
Cohesion: 0.09
Nodes (18): Filter, FilterType, PEQData, snap_freq_to_iso(), snap_q_to_iso(), test_band_gain_clamp_at_max(), test_band_gain_clamp_at_min(), test_band_gain_unchanged_when_in_bounds() (+10 more)

### Community 13 - "IPC Payload Validation"
Cohesion: 0.22
Nodes (7): ConnectionResult, OperationResult, PushPayload, handle_hardware(), is_hw_busy(), perform_pull(), recv_operation_result()

### Community 14 - "Header & Status UI"
Cohesion: 0.2
Nodes (6): sync_toolbar_button(), view_header(), icon_button(), small_action_button(), toolbar_button(), view_status_banner()

### Community 19 - "Message Routing Types"
Cohesion: 0.5
Nodes (3): Message, StatusMessage, StatusSeverity

### Community 30 - "Project README"
Cohesion: 0.06
Nodes (31): 1. Clone the Repository, 2. Build and Run, 3. (Linux Only) Setup USB Permissions, Acknowledgments, Arch Linux Manual Install, Architecture, Available Scripts, code:bash (git clone https://github.com/Bukutsu/frost-tune.git) (+23 more)

### Community 43 - "Community 43"
Cohesion: 0.1
Nodes (20): Adding New Components, Architecture, Architecture Patterns, Code Standards (Rust-Pro Guidelines), code:block1 (frost-tune/), code:bash (# Development workflow), Contribution Workflow, Cutting a Release (+12 more)

### Community 44 - "Community 44"
Cohesion: 0.1
Nodes (20): Adding New Components, Architecture, Architecture Patterns, Code Standards (Rust-Pro Guidelines), code:block1 (frost-tune/), code:bash (# Development workflow), Contribution Workflow, Cutting a Release (+12 more)

## Knowledge Gaps
- **95 isolated node(s):** `Profile`, `CommandSpec`, `DeviceProtocol`, `HelperRequest`, `HelperResponse` (+90 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `assemble_filters()` connect `HID Operations Pipeline` to `Domain Rules Snapping`?**
  _High betweenness centrality (0.142) - this node is a cross-community bridge._
- **Why does `handle_editor()` connect `Domain Rules Snapping` to `Profile & Connection Management`, `Main Window UI`?**
  _High betweenness centrality (0.088) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `parse_autoeq_text()` (e.g. with `.enabled()` and `load_all_profiles()`) actually correct?**
  _`parse_autoeq_text()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Profile`, `CommandSpec`, `DeviceProtocol` to the rest of the system?**
  _101 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Profile & Connection Management` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._
- **Should `Main Window UI` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._
- **Should `Hardware Worker Server` be split into smaller, more focused modules?**
  _Cohesion score 0.06 - nodes in this community are weakly interconnected._