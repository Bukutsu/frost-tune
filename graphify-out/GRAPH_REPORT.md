# Graph Report - frost-tune  (2026-05-17)

## Corpus Check
- 335 files · ~244,679 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1174 nodes · 2412 edges · 84 communities (69 shown, 15 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 99 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cba46bd9`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Main UI & Window Management|Main UI & Window Management]]
- [[_COMMUNITY_Profile Storage & State Updates|Profile Storage & State Updates]]
- [[_COMMUNITY_Background Worker & HID Connection|Background Worker & HID Connection]]
- [[_COMMUNITY_Hardware IO & Packet Logic|Hardware I/O & Packet Logic]]
- [[_COMMUNITY_Opencode Configuration|Opencode Configuration]]
- [[_COMMUNITY_UI Theme & Styling|UI Theme & Styling]]
- [[_COMMUNITY_UI State Model|UI State Model]]
- [[_COMMUNITY_DSP & EQ Graph Rendering|DSP & EQ Graph Rendering]]
- [[_COMMUNITY_AutoEQ Profile Parsing|AutoEQ Profile Parsing]]
- [[_COMMUNITY_Hardware Protocol Definition|Hardware Protocol Definition]]
- [[_COMMUNITY_Elevated Privileges & Error Handling|Elevated Privileges & Error Handling]]
- [[_COMMUNITY_Diagnostics & Logging|Diagnostics & Logging]]
- [[_COMMUNITY_EQ Filter Model & Editor Logic|EQ Filter Model & Editor Logic]]
- [[_COMMUNITY_Model Unit Tests|Model Unit Tests]]
- [[_COMMUNITY_Inter-Process Communication (IPC)|Inter-Process Communication (IPC)]]
- [[_COMMUNITY_Header & Status Bar UI|Header & Status Bar UI]]
- [[_COMMUNITY_AI Agent Instructions|AI Agent Instructions]]
- [[_COMMUNITY_EQ Band Editor UI|EQ Band Editor UI]]
- [[_COMMUNITY_Worker IPC Integration Tests|Worker IPC Integration Tests]]
- [[_COMMUNITY_Gemini CLI Settings|Gemini CLI Settings]]
- [[_COMMUNITY_Hardware Packet Timings|Hardware Packet Timings]]
- [[_COMMUNITY_Protocol Integration Tests|Protocol Integration Tests]]
- [[_COMMUNITY_CI Workflow|CI Workflow]]
- [[_COMMUNITY_Worker Connection Backend|Worker Connection Backend]]
- [[_COMMUNITY_UI Message Definitions|UI Message Definitions]]
- [[_COMMUNITY_Claude CLI Settings|Claude CLI Settings]]
- [[_COMMUNITY_Local Claude Settings|Local Claude Settings]]
- [[_COMMUNITY_Helper IPC Definitions|Helper IPC Definitions]]
- [[_COMMUNITY_UI Screenshot & Components|UI Screenshot & Components]]
- [[_COMMUNITY_UI Module Entry Point|UI Module Entry Point]]
- [[_COMMUNITY_Diagnostics View|Diagnostics View]]
- [[_COMMUNITY_Filter Type Enum|Filter Type Enum]]
- [[_COMMUNITY_Project README|Project README]]
- [[_COMMUNITY_Device Definitions|Device Definitions]]
- [[_COMMUNITY_Pre-amp (Global Gain) View|Pre-amp (Global Gain) View]]
- [[_COMMUNITY_Cache Warming Workflow|Cache Warming Workflow]]
- [[_COMMUNITY_Project Logo|Project Logo]]
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
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 54|Community 54]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Community 57|Community 57]]
- [[_COMMUNITY_Community 58|Community 58]]
- [[_COMMUNITY_Community 59|Community 59]]
- [[_COMMUNITY_Community 60|Community 60]]
- [[_COMMUNITY_Community 61|Community 61]]
- [[_COMMUNITY_Community 62|Community 62]]
- [[_COMMUNITY_Community 63|Community 63]]
- [[_COMMUNITY_Community 64|Community 64]]
- [[_COMMUNITY_Community 65|Community 65]]
- [[_COMMUNITY_Community 66|Community 66]]
- [[_COMMUNITY_Community 67|Community 67]]
- [[_COMMUNITY_Community 68|Community 68]]

## God Nodes (most connected - your core abstractions)
1. `MainWindow` - 25 edges
2. `parse_autoeq_text()` - 20 edges
3. `TP35ProProtocol` - 20 edges
4. `Frost-Tune — Developer Guidelines` - 18 edges
5. `Frost-Tune — Developer Guidelines` - 18 edges
6. `Frost-Tune — Developer Guidelines` - 18 edges
7. `push_with_verify()` - 16 edges
8. `handle_profiles()` - 16 edges
9. `InputBuffer` - 15 edges
10. `load_all_profiles()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `test_disabled_button_contrast_wcag_aa()` --calls--> `theme()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `test_disabled_button_contrast_wcag_aa()` --calls--> `pill_secondary_button()`  [INFERRED]
  tests/token_consistency.rs → src/ui/theme.rs
- `Automated Release Process` --rationale_for--> `Release Workflow`  [INFERRED]
  AGENTS.md → .github/workflows/release.yml
- `Release Workflow` --conceptually_related_to--> `Conventional Commits Standard`  [INFERRED]
  .github/workflows/release.yml → AGENTS.md
- `load_all_profiles()` --calls--> `parse_autoeq_text()`  [INFERRED]
  src/storage.rs → src/autoeq.rs

## Communities (84 total, 15 thin omitted)

### Community 0 - "Main UI & Window Management"
Cohesion: 0.12
Nodes (7): layout_bucket_for_width(), LayoutBucket, MainWindow, parse_freq_string(), run(), run_with_diagnostics(), test_parse_freq_string()

### Community 1 - "Profile Storage & State Updates"
Cohesion: 0.24
Nodes (30): append_diagnostics_log(), delete_profile(), export_profile(), get_base_dir(), get_diagnostics_log_path(), get_profiles_dir(), get_profiles_dir_mtime(), get_ui_preferences_path() (+22 more)

### Community 2 - "Background Worker & HID Connection"
Cohesion: 0.12
Nodes (15): try_connect_elevated(), try_connect_local(), worker_connect(), IterationResult, panic_message(), run_worker_iteration(), UsbCommand, UsbWorker (+7 more)

### Community 3 - "Hardware I/O & Packet Logic"
Cohesion: 0.18
Nodes (21): assemble_filters(), delay_ms(), detect_device(), device_info_from_hid(), find_device_info(), flush_hid_buffer(), get_next_nonce(), list_devices() (+13 more)

### Community 4 - "Opencode Configuration"
Cohesion: 0.07
Nodes (29): description, template, description, template, description, template, command, check (+21 more)

### Community 5 - "UI Theme & Styling"
Cohesion: 0.26
Nodes (19): card_style(), checkbox_style(), enforce_disabled_button_contrast(), header_card_style(), m3_filled_input(), m3_input_pick_list(), m3_outlined_input(), pill_danger_button() (+11 more)

### Community 6 - "UI State Model"
Cohesion: 0.16
Nodes (14): ConfirmAction, ConnectionStatus, DisconnectReason, DraftFilter, EditorData, EditorSession, EditorState, EditorUI (+6 more)

### Community 7 - "DSP & EQ Graph Rendering"
Cohesion: 0.43
Nodes (17): calculate_total_response(), compute_iir_filter(), convert_to_byte_array(), get_biquad_coefficients(), get_magnitude_response(), get_magnitude_response_with_coeffs(), parse_filter_packet(), quantizer() (+9 more)

### Community 8 - "AutoEQ Profile Parsing"
Cohesion: 0.26
Nodes (19): contains_token(), extract_fc_value(), extract_gain_value(), extract_number(), extract_number_after(), extract_q_value(), parse_autoeq_text(), parse_filter_line() (+11 more)

### Community 9 - "Hardware Protocol Definition"
Cohesion: 0.17
Nodes (6): DeviceProtocol, test_tp35pro_build_filter_write_packet(), test_tp35pro_build_flash_eq_packet(), test_tp35pro_build_global_gain_write_packet(), test_tp35pro_build_temp_write_packet(), TP35ProProtocol

### Community 10 - "Elevated Privileges & Error Handling"
Cohesion: 0.12
Nodes (12): CommandSpec, ElevatedTransport, spawn_via_pkexec(), validate_pkexec_target(), handle_connect(), pull_logic(), push_logic(), require_device() (+4 more)

### Community 11 - "Diagnostics & Logging"
Cohesion: 0.22
Nodes (7): DiagnosticEvent, DiagnosticsStore, format_diagnostic_log_line(), format_diagnostics(), LogLevel, parse_diagnostic_log_line(), Source

### Community 12 - "EQ Filter Model & Editor Logic"
Cohesion: 0.17
Nodes (11): Filter, FilterType, PEQData, snap_freq_to_iso(), snap_gain_step(), snap_q_to_iso(), cancel_band_draft_input(), commit_band_field() (+3 more)

### Community 13 - "Model Unit Tests"
Cohesion: 0.43
Nodes (15): test_band_gain_clamp_at_max(), test_band_gain_clamp_at_min(), test_band_gain_unchanged_when_in_bounds(), test_default_filter_has_correct_index(), test_global_gain_clamp_max(), test_global_gain_clamp_min(), test_push_payload_clamp_enables_all_bands(), test_push_payload_invalid_with_disabled_band() (+7 more)

### Community 14 - "Inter-Process Communication (IPC)"
Cohesion: 0.2
Nodes (7): ConnectionResult, OperationResult, PushPayload, handle_hardware(), is_hw_busy(), perform_pull(), recv_operation_result()

### Community 15 - "Header & Status Bar UI"
Cohesion: 0.16
Nodes (9): sync_toolbar_button(), view_header(), action_button(), icon_action_button(), icon_button(), section_header(), small_action_button(), toolbar_button() (+1 more)

### Community 16 - "AI Agent Instructions"
Cohesion: 0.2
Nodes (9): Conventional Commits Standard, Elm Architecture (Iced Pattern), Graphify Usage Policy, hidapi Library, Iced GUI Framework, Automated Release Process, Release Workflow, State Decomposition Pattern (data/session/ui) (+1 more)

### Community 17 - "EQ Band Editor UI"
Cohesion: 0.46
Nodes (9): render_band_column(), render_band_row(), render_freq_cell(), render_gain_cell(), render_header_row(), render_input_field(), render_q_cell(), render_type_buttons() (+1 more)

### Community 18 - "Worker IPC Integration Tests"
Cohesion: 0.53
Nodes (8): test_ipc_error_handling(), test_ipc_request_serialization(), test_ipc_response_serialization(), test_ipc_version_handshake(), test_ipc_version_mismatch_detection(), test_worker_connect_disconnect_cycle(), test_worker_new_and_status(), test_worker_status_structure()

### Community 19 - "Gemini CLI Settings"
Cohesion: 0.22
Nodes (8): host, model, experimental, gemmaModelRouter, classifier, enabled, hooks, BeforeTool

### Community 21 - "Protocol Integration Tests"
Cohesion: 0.53
Nodes (4): test_tp35pro_build_filter_read_request(), test_tp35pro_build_global_gain_request(), test_tp35pro_filter_packet_round_trip(), test_tp35pro_parse_filter_packet_invalid()

### Community 22 - "CI Workflow"
Cohesion: 0.5
Nodes (4): cargo clippy, cargo fmt --check, cargo test, CI Workflow

### Community 24 - "UI Message Definitions"
Cohesion: 0.5
Nodes (3): Message, StatusMessage, StatusSeverity

### Community 28 - "UI Screenshot & Components"
Cohesion: 0.67
Nodes (3): Frost-Tune UI Screenshot, UI: EQ Band Controls Panel, UI: Frequency Response Graph Panel

### Community 32 - "Project README"
Cohesion: 0.06
Nodes (31): 1. Clone the Repository, 2. Build and Run, 3. (Linux Only) Setup USB Permissions, Acknowledgments, Arch Linux Manual Install, Architecture, Available Scripts, code:bash (git clone https://github.com/Bukutsu/frost-tune.git) (+23 more)

### Community 40 - "Community 40"
Cohesion: 0.06
Nodes (35): description, template, description, template, description, template, command, check (+27 more)

### Community 41 - "Community 41"
Cohesion: 0.06
Nodes (35): description, template, description, template, description, template, command, check (+27 more)

### Community 42 - "Community 42"
Cohesion: 0.06
Nodes (30): 1. Clone the Repository, 2. Build and Run, 3. (Linux Only) Setup USB Permissions, Acknowledgments, Arch Linux Manual Install, Architecture, Available Scripts, code:bash (git clone https://github.com/Bukutsu/frost-tune.git) (+22 more)

### Community 43 - "Community 43"
Cohesion: 0.07
Nodes (29): description, template, description, template, description, template, command, check (+21 more)

### Community 44 - "Community 44"
Cohesion: 0.09
Nodes (22): Adding New Components, Anti-Patterns, Architecture, Architecture Patterns, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow), Contribution Workflow (+14 more)

### Community 45 - "Community 45"
Cohesion: 0.09
Nodes (22): Adding a `Message` variant, Adding a new device, Adding a new view component, Adding state — pick the right bucket, AI Coding Guidelines, Anti-patterns to avoid, Architecture, Architecture Patterns (+14 more)

### Community 46 - "Community 46"
Cohesion: 0.09
Nodes (22): Adding a `Message` variant, Adding a new device, Adding a new view component, Adding state — pick the right bucket, AI Coding Guidelines, Anti-patterns to avoid, Architecture, Architecture Patterns (+14 more)

### Community 47 - "Community 47"
Cohesion: 0.09
Nodes (22): Adding a `Message` variant, Adding a new device, Adding a new view component, Adding state — pick the right bucket, AI Coding Guidelines, Anti-patterns to avoid, Architecture, Architecture Patterns (+14 more)

### Community 48 - "Community 48"
Cohesion: 0.11
Nodes (17): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing (+9 more)

### Community 49 - "Community 49"
Cohesion: 0.11
Nodes (19): Anti-Patterns, Architecture, Architecture Patterns, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow), Contribution Workflow, Cutting a Release (+11 more)

### Community 50 - "Community 50"
Cohesion: 0.11
Nodes (19): Anti-Patterns, Architecture, Architecture Patterns, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow), Contribution Workflow, Cutting a Release (+11 more)

### Community 51 - "Community 51"
Cohesion: 0.11
Nodes (17): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing (+9 more)

### Community 52 - "Community 52"
Cohesion: 0.11
Nodes (17): Acknowledgments, Arch Linux, Build and run, code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (git clone https://github.com/Bukutsu/frost-tune.git), code:bash (echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", \), Contributing (+9 more)

### Community 53 - "Community 53"
Cohesion: 0.22
Nodes (6): handle_connection(), maybe_check_profiles(), maybe_reconnect(), poll_worker_status(), timed_out_connection_result(), update()

### Community 54 - "Community 54"
Cohesion: 0.54
Nodes (11): compare_peq(), make_filter(), pull_peq_data(), rollback_and_verify(), rollback_state(), test_compare_peq_filter_type_mismatch(), test_compare_peq_freq_mismatch(), test_compare_peq_gain_mismatch() (+3 more)

### Community 55 - "Community 55"
Cohesion: 0.14
Nodes (13): Adding New Devices, Architecture, Architecture Patterns, Claude Code Configuration, Code Organization & Helpers, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow) (+5 more)

### Community 56 - "Community 56"
Cohesion: 0.14
Nodes (13): Adding New Devices, Architecture, Architecture Patterns, Claude Code Configuration, Code Organization & Helpers, Code Standards, code:block1 (frost-tune/), code:bash (# Development workflow) (+5 more)

### Community 57 - "Community 57"
Cohesion: 0.15
Nodes (12): Adding New Devices, Architecture, Claude Code Configuration, Code Conventions, code:block1 (src/), code:bash (cargo fmt                          # Format code (run before), Frost-Tune — Agent Guidelines, graphify (+4 more)

### Community 58 - "Community 58"
Cohesion: 0.15
Nodes (12): Adding New Devices, Architecture, Claude Code Configuration, Code Conventions, code:block1 (src/), code:bash (cargo fmt                          # Format code (run before), Frost-Tune — Agent Guidelines, graphify (+4 more)

### Community 59 - "Community 59"
Cohesion: 0.17
Nodes (10): Adding New Devices, Architecture, Code Conventions, code:block1 (src/), code:bash (cargo fmt                          # Format code (run before), Frost-Tune — Agent Guidelines, Important Patterns, Key Commands (+2 more)

### Community 60 - "Community 60"
Cohesion: 0.58
Nodes (6): contrast_ratio(), linear_channel(), test_band_density(), test_disabled_button_contrast_wcag_aa(), test_shape_semantics_tokens(), test_token_consistency()

### Community 61 - "Community 61"
Cohesion: 0.4
Nodes (3): EqGraph, graph_label_layout(), GraphLabelLayout

### Community 62 - "Community 62"
Cohesion: 0.53
Nodes (4): dialog_container(), view_confirm_dialog(), view_exit_dialog(), view_name_input_dialog()

### Community 63 - "Community 63"
Cohesion: 0.5
Nodes (3): graph_container_style(), view_graph(), view_graph_fill()

### Community 65 - "Community 65"
Cohesion: 0.67
Nodes (3): Adding New Components, New device, New view component

### Community 66 - "Community 66"
Cohesion: 0.67
Nodes (3): Adding New Components, New device, New view component

## Knowledge Gaps
- **301 isolated node(s):** `$schema`, `instructions`, `command`, `extensions`, `template` (+296 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `MainWindow` connect `Main UI & Window Management` to `Community 62`?**
  _High betweenness centrality (0.034) - this node is a cross-community bridge._
- **Why does `handle_editor()` connect `EQ Filter Model & Editor Logic` to `Main UI & Window Management`, `Community 53`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **Why does `update()` connect `Community 53` to `AutoEQ Profile Parsing`, `Profile Storage & State Updates`, `EQ Filter Model & Editor Logic`, `Inter-Process Communication (IPC)`?**
  _High betweenness centrality (0.027) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `parse_autoeq_text()` (e.g. with `load_all_profiles()` and `import_profile()`) actually correct?**
  _`parse_autoeq_text()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **What connects `$schema`, `instructions`, `command` to the rest of the system?**
  _312 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Main UI & Window Management` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._
- **Should `Background Worker & HID Connection` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._