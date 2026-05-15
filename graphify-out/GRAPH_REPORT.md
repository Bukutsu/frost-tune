# Graph Report - frost-tune  (2026-05-15)

## Corpus Check
- 372 files · ~262,968 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1053 nodes · 2710 edges · 87 communities (64 shown, 23 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 119 edges (avg confidence: 0.81)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `e5e3d73c`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

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
- [[_COMMUNITY_Worker IPC Tests|Worker IPC Tests]]
- [[_COMMUNITY_Error Types|Error Types]]
- [[_COMMUNITY_UI Components|UI Components]]
- [[_COMMUNITY_Band Visualization|Band Visualization]]
- [[_COMMUNITY_Protocol Tests|Protocol Tests]]
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
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
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
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 72|Community 72]]
- [[_COMMUNITY_Community 73|Community 73]]
- [[_COMMUNITY_Community 74|Community 74]]
- [[_COMMUNITY_Community 75|Community 75]]
- [[_COMMUNITY_Community 76|Community 76]]
- [[_COMMUNITY_Community 77|Community 77]]
- [[_COMMUNITY_Community 78|Community 78]]
- [[_COMMUNITY_Community 79|Community 79]]
- [[_COMMUNITY_Community 80|Community 80]]
- [[_COMMUNITY_Community 81|Community 81]]
- [[_COMMUNITY_Community 82|Community 82]]
- [[_COMMUNITY_Community 83|Community 83]]
- [[_COMMUNITY_Community 84|Community 84]]
- [[_COMMUNITY_Community 85|Community 85]]
- [[_COMMUNITY_Community 86|Community 86]]

## God Nodes (most connected - your core abstractions)
1. `Message` - 79 edges
2. `Filter` - 48 edges
3. `PEQData` - 27 edges
4. `MainWindow` - 25 edges
5. `TP35ProProtocol` - 21 edges
6. `MainWindow` - 21 edges
7. `parse_autoeq_text()` - 19 edges
8. `handle_profiles()` - 18 edges
9. `push_with_verify()` - 17 edges
10. `handle_editor()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `test_disabled_button_contrast_wcag_aa()` --calls--> `theme()`  [INFERRED]
  tests/token_consistency.rs → src/frost-tune-0.8.3/src/ui/theme.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `pill_secondary_button()`  [INFERRED]
  tests/token_consistency.rs → src/frost-tune-0.8.3/src/ui/theme.rs
- `load_all_profiles()` --calls--> `parse_autoeq_text()`  [INFERRED]
  src/frost-tune-0.8.3/src/storage.rs → src/frost-tune-0.8.3/src/autoeq.rs
- `import_profile()` --calls--> `parse_autoeq_text()`  [INFERRED]
  src/frost-tune-0.8.3/src/storage.rs → src/frost-tune-0.8.3/src/autoeq.rs
- `test_import_profile()` --calls--> `parse_autoeq_text()`  [INFERRED]
  src/frost-tune-0.8.3/src/storage.rs → src/frost-tune-0.8.3/src/autoeq.rs

## Communities (87 total, 23 thin omitted)

### Community 0 - "Privilege Escalation"
Cohesion: 0.06
Nodes (41): Bands View, ConfirmAction, Confirm Dialog View, ConnectionResult, ConnectionStatus, DeviceInfo, Diagnostics View, DraftFilter (+33 more)

### Community 1 - "UI Architecture"
Cohesion: 0.08
Nodes (18): Filter, DeviceProtocol, test_tp35pro_build_filter_write_packet(), test_tp35pro_build_flash_eq_packet(), test_tp35pro_build_global_gain_write_packet(), test_tp35pro_build_temp_write_packet(), TP35ProProtocol, ConnectionResult (+10 more)

### Community 2 - "Dialogs & Confirmations"
Cohesion: 0.19
Nodes (33): OperationResult, PEQData, append_diagnostics_log(), delete_profile(), export_profile(), get_base_dir(), get_diagnostics_log_path(), get_profiles_dir() (+25 more)

### Community 3 - "UI Layout Engine"
Cohesion: 0.05
Nodes (47): Iced Elm architecture pattern for UI, icon_button view component, ICON_CLOSE token, m3_filled_input style, MainWindow state structure, Message::ClearStatusMessage variant, Message::DeleteProfilePressed variant, Message enum (+39 more)

### Community 4 - "Application Entry"
Cohesion: 0.18
Nodes (21): assemble_filters(), delay_ms(), detect_device(), device_info_from_hid(), find_device_info(), flush_hid_buffer(), get_next_nonce(), list_devices() (+13 more)

### Community 5 - "USB HID Operations"
Cohesion: 0.11
Nodes (7): layout_bucket_for_width(), LayoutBucket, MainWindow, parse_freq_string(), run(), run_with_diagnostics(), test_parse_freq_string()

### Community 6 - "Filter Data Models"
Cohesion: 0.16
Nodes (12): IterationResult, panic_message(), run_worker_iteration(), UsbCommand, UsbWorker, worker_status(), WorkerState, WorkerStatus (+4 more)

### Community 7 - "Design System"
Cohesion: 0.26
Nodes (18): Device, test_band_gain_clamp_at_max(), test_band_gain_clamp_at_min(), test_band_gain_unchanged_when_in_bounds(), test_default_filter_has_correct_index(), test_global_gain_clamp_max(), test_global_gain_clamp_min(), test_push_payload_clamp_enables_all_bands() (+10 more)

### Community 8 - "Packet Building"
Cohesion: 0.14
Nodes (6): CommandSpec, ElevatedTransport, spawn_via_pkexec(), validate_pkexec_target(), AppError, ErrorKind

### Community 9 - "DSP Algorithms"
Cohesion: 0.2
Nodes (11): Filter, FilterType, PEQData, snap_freq_to_iso(), snap_gain_step(), snap_q_to_iso(), cancel_band_draft_input(), commit_band_field() (+3 more)

### Community 10 - "UI State Management"
Cohesion: 0.19
Nodes (9): ConfirmAction, ConnectionStatus, DisconnectReason, DraftFilter, EditorState, InputBuffer, MainWindow, OperationLock (+1 more)

### Community 11 - "Device Protocol Tests"
Cohesion: 0.08
Nodes (28): Frost-Tune Agent Guidelines document, AutoEQ format for profile storage, build_filter_read_request method, build_filter_write_packet method, build_global_gain_request method, CMD_GLOBAL_GAIN constant, CMD_PEQ_VALUES constant, DeviceProtocol trait for device-specific protocols (+20 more)

### Community 12 - "Diagnostics System"
Cohesion: 0.45
Nodes (17): calculate_total_response(), compute_iir_filter(), convert_to_byte_array(), get_biquad_coefficients(), get_magnitude_response(), get_magnitude_response_with_coeffs(), parse_filter_packet(), quantizer() (+9 more)

### Community 13 - "Error & IPC Handling"
Cohesion: 0.46
Nodes (17): card_style(), checkbox_style(), enforce_disabled_button_contrast(), header_card_style(), m3_filled_input(), m3_input_pick_list(), m3_outlined_input(), pill_danger_button() (+9 more)

### Community 14 - "AutoEQ Parsing"
Cohesion: 0.24
Nodes (7): DiagnosticEvent, DiagnosticsStore, format_diagnostic_log_line(), format_diagnostics(), LogLevel, parse_diagnostic_log_line(), Source

### Community 15 - "IPC Models"
Cohesion: 0.54
Nodes (14): extract_fc_value(), extract_gain_value(), extract_number(), extract_number_after(), extract_q_value(), parse_autoeq_text(), parse_filter_line(), peq_to_autoeq() (+6 more)

### Community 16 - "Worker IPC Tests"
Cohesion: 0.24
Nodes (7): action_button(), icon_action_button(), icon_button(), section_header(), small_action_button(), toolbar_button(), view_status_banner()

### Community 17 - "Error Types"
Cohesion: 0.1
Nodes (17): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing (+9 more)

### Community 18 - "UI Components"
Cohesion: 0.11
Nodes (17): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing (+9 more)

### Community 19 - "Band Visualization"
Cohesion: 0.56
Nodes (11): compare_peq(), make_filter(), pull_peq_data(), rollback_and_verify(), rollback_state(), test_compare_peq_filter_type_mismatch(), test_compare_peq_freq_mismatch(), test_compare_peq_gain_mismatch() (+3 more)

### Community 20 - "Protocol Tests"
Cohesion: 0.11
Nodes (17): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing (+9 more)

### Community 21 - "Worker Backend"
Cohesion: 0.12
Nodes (16): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing, Frost-Tune (+8 more)

### Community 22 - "UI Messages"
Cohesion: 0.15
Nodes (17): parse_autoeq_text, ElevatedTransport::spawn, AppError, HelperRequest, helper_server::run, pull_peq_internal, send_report, main (+9 more)

### Community 23 - "Signal Processing"
Cohesion: 0.44
Nodes (9): render_band_column(), render_band_row(), render_freq_cell(), render_gain_cell(), render_header_row(), render_input_field(), render_q_cell(), render_type_buttons() (+1 more)

### Community 24 - "Helper IPC"
Cohesion: 0.53
Nodes (8): test_ipc_error_handling(), test_ipc_request_serialization(), test_ipc_response_serialization(), test_ipc_version_handshake(), test_ipc_version_mismatch_detection(), test_worker_connect_disconnect_cycle(), test_worker_new_and_status(), test_worker_status_structure()

### Community 25 - "Transport Layer"
Cohesion: 0.13
Nodes (13): Adding New Devices, Architecture, Architecture Patterns, Claude Code Configuration, Code Organization & Helpers, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow) (+5 more)

### Community 26 - "Button Tokens"
Cohesion: 0.13
Nodes (13): Adding New Devices, Architecture, Architecture Patterns, Claude Code Configuration, Code Organization & Helpers, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow) (+5 more)

### Community 27 - "Band Tokens"
Cohesion: 0.14
Nodes (13): Adding New Devices, Architecture, Architecture Patterns, Claude Code Configuration, Code Organization & Helpers, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow) (+5 more)

### Community 28 - "Device Models"
Cohesion: 0.14
Nodes (13): Adding New Devices, Architecture, Architecture Patterns, Claude Code Configuration, Code Organization & Helpers, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow) (+5 more)

### Community 29 - "Filter Utils"
Cohesion: 0.56
Nodes (6): contrast_ratio(), linear_channel(), test_band_density(), test_disabled_button_contrast_wcag_aa(), test_shape_semantics_tokens(), test_token_consistency()

### Community 30 - "Community 30"
Cohesion: 0.53
Nodes (6): handle_connect(), pull_logic(), push_logic(), require_device(), run(), write_response()

### Community 31 - "Community 31"
Cohesion: 0.15
Nodes (10): Adding New Devices, Architecture, Code Conventions, code:block1 (src/), code:bash (cargo fmt                          # Format code (run before), Frost-Tune — Agent Guidelines, Important Patterns, Key Commands (+2 more)

### Community 32 - "Community 32"
Cohesion: 0.15
Nodes (12): Adding New Devices, Architecture, Claude Code Configuration, Code Conventions, code:block1 (src/), code:bash (cargo fmt                          # Format code (run before), Frost-Tune — Agent Guidelines, graphify (+4 more)

### Community 33 - "Community 33"
Cohesion: 0.15
Nodes (12): Adding New Devices, Architecture, Claude Code Configuration, Code Conventions, code:block1 (src/), code:bash (cargo fmt                          # Format code (run before), Frost-Tune — Agent Guidelines, graphify (+4 more)

### Community 34 - "Community 34"
Cohesion: 0.42
Nodes (5): handle_connection(), maybe_check_profiles(), maybe_reconnect(), poll_worker_status(), timed_out_connection_result()

### Community 35 - "Community 35"
Cohesion: 0.51
Nodes (4): test_tp35pro_build_filter_read_request(), test_tp35pro_build_global_gain_request(), test_tp35pro_filter_packet_round_trip(), test_tp35pro_parse_filter_packet_invalid()

### Community 37 - "Community 37"
Cohesion: 0.51
Nodes (3): try_connect_elevated(), try_connect_local(), worker_connect()

### Community 38 - "Community 38"
Cohesion: 0.47
Nodes (3): Message, StatusMessage, StatusSeverity

### Community 40 - "Community 40"
Cohesion: 0.22
Nodes (9): AppError type, ErrorKind enum, HelperRequest enum, HelperResponse enum, HID I/O isolation on background threads, IPC_VERSION constant, worker_ipc tests module, UsbWorker type (+1 more)

### Community 44 - "Community 44"
Cohesion: 0.5
Nodes (4): compute_iir_filter, parse_filter_packet, DeviceProtocol, TP35ProProtocol

### Community 45 - "Community 45"
Cohesion: 0.67
Nodes (3): TransportBackend, worker_connect, ElevatedTransport

### Community 46 - "Community 46"
Cohesion: 0.67
Nodes (3): BAND_ROW_MIN_HEIGHT token, BAND_ROW_PADDING token, test_band_density function

### Community 47 - "Community 47"
Cohesion: 0.67
Nodes (3): BUTTON_PILL_RADIUS token, INPUT_RADIUS token, test_shape_semantics_tokens function

## Knowledge Gaps
- **124 isolated node(s):** `u8`, `ToolsTab`, `Supported devices`, `Hardware safety`, `Prerequisites` (+119 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Filter` connect `UI Architecture` to `Privilege Escalation`, `Dialogs & Confirmations`, `Application Entry`, `USB HID Operations`, `Design System`?**
  _High betweenness centrality (0.192) - this node is a cross-community bridge._
- **Why does `Message` connect `Privilege Escalation` to `Dialogs & Confirmations`, `Design System`, `Device Protocol Tests`, `Worker IPC Tests`, `Signal Processing`?**
  _High betweenness centrality (0.166) - this node is a cross-community bridge._
- **Why does `PEQData` connect `Dialogs & Confirmations` to `Privilege Escalation`, `UI Architecture`?**
  _High betweenness centrality (0.139) - this node is a cross-community bridge._
- **What connects `u8`, `ToolsTab`, `Supported devices` to the rest of the system?**
  _124 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Privilege Escalation` be split into smaller, more focused modules?**
  _Cohesion score 0.06 - nodes in this community are weakly interconnected._
- **Should `UI Architecture` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `UI Layout Engine` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._