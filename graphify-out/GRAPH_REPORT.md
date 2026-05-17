# Graph Report - .  (2026-05-17)

## Corpus Check
- Corpus is ~44,495 words - fits in a single context window. You may not need a graph.

## Summary
- 434 nodes · 652 edges · 42 communities (24 shown, 18 thin omitted)
- Extraction: 83% EXTRACTED · 17% INFERRED · 0% AMBIGUOUS · INFERRED: 110 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

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
- [[_COMMUNITY_EQ Bands Table UI|EQ Bands Table UI]]
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

## God Nodes (most connected - your core abstractions)
1. `MainWindow` - 25 edges
2. `parse_autoeq_text()` - 16 edges
3. `TP35ProProtocol` - 15 edges
4. `push_with_verify()` - 11 edges
5. `handle_profiles()` - 11 edges
6. `InputBuffer` - 10 edges
7. `load_all_profiles()` - 9 edges
8. `save_profile()` - 9 edges
9. `delay_ms()` - 9 edges
10. `pull_peq_internal()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `test_disabled_button_contrast_wcag_aa()` --calls--> `theme()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `pill_secondary_button()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `run_with_diagnostics()` --calls--> `parse_diagnostic_log_line()`  [INFERRED]
  src/ui/main_window.rs → src/diagnostics.rs
- `handle_editor()` --calls--> `parse_freq_string()`  [INFERRED]
  src/ui/update/editor.rs → src/ui/main_window.rs
- `handle_autoeq()` --calls--> `format_diagnostics()`  [INFERRED]
  src/ui/update/autoeq.rs → src/diagnostics.rs

## Communities (42 total, 18 thin omitted)

### Community 0 - "Profile & Connection Management"
Cohesion: 0.09
Nodes (38): peq_to_autoeq(), test_peq_to_autoeq_format(), main(), append_diagnostics_log(), delete_profile(), export_profile(), get_base_dir(), get_diagnostics_log_path() (+30 more)

### Community 1 - "Main Window UI"
Cohesion: 0.07
Nodes (11): layout_bucket_for_width(), LayoutBucket, MainWindow, parse_freq_string(), run(), run_with_diagnostics(), dialog_container(), view_confirm_dialog() (+3 more)

### Community 2 - "Hardware Worker Server"
Cohesion: 0.08
Nodes (23): handle_connect(), pull_logic(), push_logic(), require_device(), run(), write_response(), device_info_from_hid(), find_device_info() (+15 more)

### Community 3 - "HID Operations Pipeline"
Cohesion: 0.14
Nodes (23): assemble_filters(), delay_ms(), flush_hid_buffer(), get_next_nonce(), pull_peq_internal(), read_global_gain(), read_single_filter_with_nonce(), reset_nonce() (+15 more)

### Community 4 - "Theme & UI Tokens"
Cohesion: 0.1
Nodes (15): contrast_ratio(), linear_channel(), test_disabled_button_contrast_wcag_aa(), enforce_disabled_button_contrast(), m3_filled_input(), m3_outlined_input(), pill_danger_button(), pill_outlined_danger_button() (+7 more)

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
Cohesion: 0.18
Nodes (6): CommandSpec, ElevatedTransport, spawn_via_pkexec(), validate_pkexec_target(), AppError, ErrorKind

### Community 11 - "Domain Rules Snapping"
Cohesion: 0.18
Nodes (9): test_band_gain_clamp_at_max(), test_band_gain_clamp_at_min(), test_band_gain_unchanged_when_in_bounds(), test_default_filter_has_correct_index(), test_push_payload_clamp_enables_all_bands(), test_push_payload_invalid_with_disabled_band(), test_push_payload_invalid_with_inf_q(), test_push_payload_invalid_with_nan_gain() (+1 more)

### Community 12 - "Filter Model Input"
Cohesion: 0.16
Nodes (9): Filter, FilterType, PEQData, snap_freq_to_iso(), snap_q_to_iso(), cancel_band_draft_input(), commit_band_field(), handle_band_text_input() (+1 more)

### Community 13 - "IPC Payload Validation"
Cohesion: 0.22
Nodes (7): ConnectionResult, OperationResult, PushPayload, handle_hardware(), is_hw_busy(), perform_pull(), recv_operation_result()

### Community 14 - "Header & Status UI"
Cohesion: 0.2
Nodes (6): sync_toolbar_button(), view_header(), icon_button(), small_action_button(), toolbar_button(), view_status_banner()

### Community 15 - "EQ Bands Table UI"
Cohesion: 0.33
Nodes (7): render_band_column(), render_band_row(), render_freq_cell(), render_gain_cell(), render_q_cell(), render_type_buttons(), view_bands()

### Community 19 - "Message Routing Types"
Cohesion: 0.5
Nodes (3): Message, StatusMessage, StatusSeverity

## Knowledge Gaps
- **34 isolated node(s):** `Profile`, `UiPreferences`, `CommandSpec`, `DeviceProtocol`, `HelperRequest` (+29 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **18 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `assemble_filters()` connect `HID Operations Pipeline` to `Domain Rules Snapping`?**
  _High betweenness centrality (0.202) - this node is a cross-community bridge._
- **Why does `MainWindow` connect `Main Window UI` to `Theme & UI Tokens`?**
  _High betweenness centrality (0.168) - this node is a cross-community bridge._
- **Why does `handle_editor()` connect `Filter Model Input` to `Profile & Connection Management`, `Main Window UI`, `Domain Rules Snapping`?**
  _High betweenness centrality (0.131) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `parse_autoeq_text()` (e.g. with `.enabled()` and `load_all_profiles()`) actually correct?**
  _`parse_autoeq_text()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **Are the 10 inferred relationships involving `push_with_verify()` (e.g. with `send_report()` and `delay_ms()`) actually correct?**
  _`push_with_verify()` has 10 INFERRED edges - model-reasoned connections that need verification._
- **Are the 7 inferred relationships involving `handle_profiles()` (e.g. with `update()` and `open_profiles_dir()`) actually correct?**
  _`handle_profiles()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Profile`, `UiPreferences`, `CommandSpec` to the rest of the system?**
  _40 weakly-connected nodes found - possible documentation gaps or missing edges._