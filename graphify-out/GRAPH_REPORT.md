# Graph Report - .  (2026-05-14)

## Corpus Check
- Corpus is ~36,856 words - fits in a single context window. You may not need a graph.

## Summary
- 558 nodes · 818 edges · 50 communities (29 shown, 21 thin omitted)
- Extraction: 85% EXTRACTED · 15% INFERRED · 0% AMBIGUOUS · INFERRED: 119 edges (avg confidence: 0.81)
- Token cost: 8,524 input · 4,821 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Privilege Escalation|Privilege Escalation]]
- [[_COMMUNITY_UI Architecture|UI Architecture]]
- [[_COMMUNITY_Dialogs & Confirmations|Dialogs & Confirmations]]
- [[_COMMUNITY_UI Layout Engine|UI Layout Engine]]
- [[_COMMUNITY_Application Entry|Application Entry]]
- [[_COMMUNITY_USB HID Operations|USB HID Operations]]
- [[_COMMUNITY_Filter Data Models|Filter Data Models]]
- [[_COMMUNITY_Design System|Design System]]
- [[_COMMUNITY_Packet Building|Packet Building]]
- [[_COMMUNITY_DSP Algorithms|DSP Algorithms]]
- [[_COMMUNITY_UI State Management|UI State Management]]
- [[_COMMUNITY_Device Protocol Tests|Device Protocol Tests]]
- [[_COMMUNITY_Diagnostics System|Diagnostics System]]
- [[_COMMUNITY_Error & IPC Handling|Error & IPC Handling]]
- [[_COMMUNITY_AutoEQ Parsing|AutoEQ Parsing]]
- [[_COMMUNITY_IPC Models|IPC Models]]
- [[_COMMUNITY_Error Types|Error Types]]
- [[_COMMUNITY_UI Components|UI Components]]
- [[_COMMUNITY_Band Visualization|Band Visualization]]
- [[_COMMUNITY_Worker Backend|Worker Backend]]
- [[_COMMUNITY_UI Messages|UI Messages]]
- [[_COMMUNITY_Signal Processing|Signal Processing]]
- [[_COMMUNITY_Helper IPC|Helper IPC]]
- [[_COMMUNITY_Transport Layer|Transport Layer]]
- [[_COMMUNITY_Button Tokens|Button Tokens]]
- [[_COMMUNITY_Band Tokens|Band Tokens]]
- [[_COMMUNITY_Device Models|Device Models]]
- [[_COMMUNITY_Filter Utils|Filter Utils]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]

## God Nodes (most connected - your core abstractions)
1. `MainWindow` - 25 edges
2. `Message` - 25 edges
3. `TP35ProProtocol` - 15 edges
4. `MainWindow` - 15 edges
5. `parse_autoeq_text()` - 13 edges
6. `Filter` - 12 edges
7. `push_with_verify()` - 11 edges
8. `handle_profiles()` - 11 edges
9. `load_all_profiles()` - 10 edges
10. `InputBuffer` - 10 edges

## Surprising Connections (you probably didn't know these)
- `run_with_diagnostics()` --calls--> `parse_diagnostic_log_line()`  [INFERRED]
  src/ui/main_window.rs → src/diagnostics.rs
- `handle_editor()` --calls--> `parse_freq_string()`  [INFERRED]
  src/ui/update/editor.rs → src/ui/main_window.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `theme()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `pill_secondary_button()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `load_all_profiles()` --calls--> `parse_autoeq_text()`  [INFERRED]
  src/storage.rs → src/autoeq.rs

## Communities (50 total, 21 thin omitted)

### Community 0 - "Privilege Escalation"
Cohesion: 0.07
Nodes (26): CommandSpec, ElevatedTransport, spawn_via_pkexec(), validate_pkexec_target(), pull_logic(), push_logic(), run(), write_response() (+18 more)

### Community 1 - "UI Architecture"
Cohesion: 0.05
Nodes (47): Iced Elm architecture pattern for UI, icon_button view component, ICON_CLOSE token, m3_filled_input style, MainWindow state structure, Message::ClearStatusMessage variant, Message::DeleteProfilePressed variant, Message enum (+39 more)

### Community 2 - "Dialogs & Confirmations"
Cohesion: 0.09
Nodes (34): Bands View, ConfirmAction, Confirm Dialog View, ConnectionResult, ConnectionStatus, Device, DeviceInfo, Diagnostics View (+26 more)

### Community 3 - "UI Layout Engine"
Cohesion: 0.07
Nodes (12): layout_bucket_for_width(), LayoutBucket, MainWindow, parse_freq_string(), run(), run_with_diagnostics(), dialog_container(), view_confirm_dialog() (+4 more)

### Community 4 - "Application Entry"
Cohesion: 0.1
Nodes (33): peq_to_autoeq(), test_peq_to_autoeq_format(), main(), append_diagnostics_log(), delete_profile(), export_profile(), get_base_dir(), get_diagnostics_log_path() (+25 more)

### Community 5 - "USB HID Operations"
Cohesion: 0.12
Nodes (26): assemble_filters(), delay_ms(), flush_hid_buffer(), get_next_nonce(), pull_peq_internal(), read_global_gain(), read_single_filter_with_nonce(), ReadTiming (+18 more)

### Community 6 - "Filter Data Models"
Cohesion: 0.09
Nodes (16): Filter, FilterType, PEQData, snap_freq_to_iso(), snap_q_to_iso(), test_band_gain_clamp_at_max(), test_band_gain_clamp_at_min(), test_band_gain_unchanged_when_in_bounds() (+8 more)

### Community 7 - "Design System"
Cohesion: 0.1
Nodes (15): contrast_ratio(), linear_channel(), test_disabled_button_contrast_wcag_aa(), enforce_disabled_button_contrast(), m3_filled_input(), m3_outlined_input(), pill_danger_button(), pill_outlined_danger_button() (+7 more)

### Community 8 - "Packet Building"
Cohesion: 0.08
Nodes (28): Frost-Tune Agent Guidelines document, AutoEQ format for profile storage, build_filter_read_request method, build_filter_write_packet method, build_global_gain_request method, CMD_GLOBAL_GAIN constant, CMD_PEQ_VALUES constant, DeviceProtocol trait for device-specific protocols (+20 more)

### Community 9 - "DSP Algorithms"
Cohesion: 0.14
Nodes (20): calculate_total_response(), compute_iir_filter(), convert_to_byte_array(), get_biquad_coefficients(), get_magnitude_response(), get_magnitude_response_with_coeffs(), parse_filter_packet(), quantizer() (+12 more)

### Community 10 - "UI State Management"
Cohesion: 0.12
Nodes (8): ConfirmAction, ConnectionStatus, DisconnectReason, DraftFilter, EditorState, InputBuffer, MainWindow, OperationLock

### Community 11 - "Device Protocol Tests"
Cohesion: 0.11
Nodes (6): DeviceProtocol, test_tp35pro_build_filter_write_packet(), test_tp35pro_build_flash_eq_packet(), test_tp35pro_build_global_gain_write_packet(), test_tp35pro_build_temp_write_packet(), TP35ProProtocol

### Community 12 - "Diagnostics System"
Cohesion: 0.13
Nodes (6): DiagnosticEvent, DiagnosticsStore, format_diagnostics(), LogLevel, parse_diagnostic_log_line(), Source

### Community 13 - "Error & IPC Handling"
Cohesion: 0.15
Nodes (17): parse_autoeq_text, ElevatedTransport::spawn, AppError, HelperRequest, helper_server::run, pull_peq_internal, send_report, main (+9 more)

### Community 14 - "AutoEQ Parsing"
Cohesion: 0.31
Nodes (13): extract_fc_value(), extract_gain_value(), extract_number(), extract_number_after(), extract_q_value(), parse_autoeq_text(), parse_filter_line(), test_parse_autoeq_clamp_gain() (+5 more)

### Community 15 - "IPC Models"
Cohesion: 0.23
Nodes (6): ConnectionResult, OperationResult, PushPayload, handle_hardware(), perform_pull(), recv_operation_result()

### Community 17 - "Error Types"
Cohesion: 0.22
Nodes (9): AppError type, ErrorKind enum, HelperRequest enum, HelperResponse enum, HID I/O isolation on background threads, IPC_VERSION constant, worker_ipc tests module, UsbWorker type (+1 more)

### Community 18 - "UI Components"
Cohesion: 0.29
Nodes (3): icon_button(), small_action_button(), view_status_banner()

### Community 19 - "Band Visualization"
Cohesion: 0.47
Nodes (3): render_band_column(), render_band_row(), view_bands()

### Community 22 - "UI Messages"
Cohesion: 0.5
Nodes (3): Message, StatusMessage, StatusSeverity

### Community 23 - "Signal Processing"
Cohesion: 0.5
Nodes (4): compute_iir_filter, parse_filter_packet, DeviceProtocol, TP35ProProtocol

### Community 25 - "Transport Layer"
Cohesion: 0.67
Nodes (3): TransportBackend, worker_connect, ElevatedTransport

### Community 26 - "Button Tokens"
Cohesion: 0.67
Nodes (3): BUTTON_PILL_RADIUS token, INPUT_RADIUS token, test_shape_semantics_tokens function

### Community 27 - "Band Tokens"
Cohesion: 0.67
Nodes (3): BAND_ROW_MIN_HEIGHT token, BAND_ROW_PADDING token, test_band_density function

## Knowledge Gaps
- **42 isolated node(s):** `Profile`, `CommandSpec`, `HelperRequest`, `HelperResponse`, `DeviceProtocol` (+37 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **21 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Filter` connect `Dialogs & Confirmations` to `UI Layout Engine`, `USB HID Operations`, `Filter Data Models`, `DSP Algorithms`, `Device Protocol Tests`, `IPC Models`?**
  _High betweenness centrality (0.297) - this node is a cross-community bridge._
- **Why does `Message` connect `Dialogs & Confirmations` to `UI Layout Engine`, `Design System`, `Packet Building`, `UI Components`, `Band Visualization`?**
  _High betweenness centrality (0.210) - this node is a cross-community bridge._
- **Why does `PEQData` connect `Dialogs & Confirmations` to `Application Entry`?**
  _High betweenness centrality (0.145) - this node is a cross-community bridge._
- **What connects `Profile`, `CommandSpec`, `HelperRequest` to the rest of the system?**
  _42 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Privilege Escalation` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._
- **Should `UI Architecture` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._
- **Should `Dialogs & Confirmations` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._