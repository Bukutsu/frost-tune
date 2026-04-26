# Polkit Elevation Plan

## Objective

Replace Linux udev rules with a polkit-based privileged helper so users don't need to configure udev rules. Keep existing audio safety (write → read-back → verify → rollback), threading model (all HID ops in worker), and transactional behavior unchanged.

## Architecture

The main app (GUI) runs unprivileged. On Linux, a small privileged helper is launched via `pkexec` + polkit policy to access USB HID devices. The main app communicates with the helper over stdin/stdout using a simple JSON line protocol. On Windows and future platforms, HID access works directly without the helper.

**Hybrid mode**: try direct user-level HID first; only invoke polkit if permission is denied. Avoids unnecessary auth prompts on systems that already grant HID access.

## Component Map

```
frost-tune (main app, unprivileged)
    │
    ├─ UsbWorker ──► local HID backend ──► hidapi (direct, Linux)
    │                    or
    │               elevated helper backend ──► helper process (root via pkexec)
    │
    └─ UI (unaffected)

helper binary: frost-tune-hid-helper
    │
    └─ privileged HID ops (read/write only known DACs)
```

## Implementation Items

### 1. Add helper IPC types

File: `src/hardware/helper_ipc.rs` (new)

Request/response enums with serde serialization. Line-oriented JSON protocol over stdin/stdout.

```rust
// Request
enum HelperRequest {
    Connect,
    Disconnect,
    Status,
    PullPEQ { strict: bool },
    PushPEQ { filters: Vec<Filter>, global_gain: Option<i8> },
    Shutdown,
}

// Response
enum HelperResponse {
    Connected { device: Option<DeviceInfo> },
    Disconnected,
    Status { connected: bool, physically_present: bool, device: Option<DeviceInfo> },
    Pulled { data: Value },  // serde_json::Value, PEQData
    Pushed { data: Value },
    Error { message: String },
    Ok,
}
```

### 2. Add privileged helper binary

Entry point: `src/bin/frost-tune-hid-helper.rs` (new)

- Long-lived process, runs elevated
- Reads requests from stdin, writes responses to stdout, one JSON line per message
- Only opens HID devices with VID/PID from the known Device map (`Device::from_vid_pid`)
- Maps existing `pull_peq_internal`, `write_filters_and_gain`, `commit_changes` into helper requests
- Prevents arbitrary HID device access (security boundary)
- Exits cleanly on `Shutdown` request

### 3. Add elevated transport

File: `src/hardware/elevated_transport.rs` (new)

- Spawn helper via `Command::new("pkexec").arg(helper_path)`
- Maintain child stdin/stdout
- Round-trip: send request, read line response, return parsed response
- On pkexec failure/not found: surface clear error (policy not installed?)
- No automatic restart logic (let UsbWorker manage reconnection)

Helper discovery: try `/usr/libexec/frost-tune/frost-tune-hid-helper` first, then fallback to `CARGO_MANIFEST_DIR`.

### 4. Add UsbWorker backend abstraction

File: refactor `src/hardware/worker.rs`

Add a `TransportBackend` enum:

```rust
enum TransportBackend {
    Local,   // direct hidapi (all platforms)
    Elevated(Box<dyn ElevatedTransport>),  // polkit helper (Linux fallback)
}
```

Modify connection flow:

```
Connect
  ├─ Try Local.open()
  │    └─ On PermissionDenied → try Elevated
  │           └─ On success → use Elevated backend
  └─ On Local.open() success → use Local backend
```

Route pull/push/status/disconnect to active backend. Auto-reconnect uses last successful backend.

### 5. Update Cargo.toml

Add helper binary target for Linux:

```toml
[[bin]]
name = "frost-tune-hid-helper"
path = "src/bin/frost-tune-hid-helper.rs"
required-features = ["linux"]
```

Or use separate Cargo feature to gate helper code on Linux.

### 6. Add polkit policy file

File: `packaging/linux/org.frosttune.hid.policy` (new, installed to `/usr/share/polkit-1/actions/`)

```xml
<?xml version="1.0"?>
<!DOCTYPE policyconfig PUBLIC "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
  "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <vendor>Frost-Tune</vendor>
  <vendor_url>https://github.com/Bukutsu/frost-tune</vendor_url>
  <action id="org.frosttune.hid.access">
    <defaults>
      <allow_any>auth_admin_keep</allow_any>
      <allow_inactive>auth_admin_keep</allow_inactive>
      <allow_active>auth_admin_keep_always</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">1</annotate>
    <annotate key="org.freedesktop.policykit.exec.delay">30000</annotate>
  </action>
</policyconfig>
```

Key settings:
- `auth_admin_keep`: requires password but caches for 5 minutes
- `auth_admin_keep_always`: caches for entire active session (recommended for convenience)
- Enforce exact helper path (annotate with helper binary path)
- 30-second execution window (prevent relay attacks)

### 7. Improve permission-denied UX

File: `src/error.rs`

Add new error variant `PolkitAuthRequired` and message text:

```rust
ErrorKind::PolkitAuthRequired => "Authentication required to access USB DAC on Linux. Install the polkit policy from the Frost-Tune documentation.",
```

File: `src/ui/main_window.rs`

Surface `PolkitAuthRequired` in `WorkerConnected` handler with a dismissable info banner (not blocking).

### 8. Update README

Replace Linux USB permissions section:

```
### Linux USB Permissions (polkit)

On Linux, Frost-Tune uses polkit for privileged USB access. No udev rules needed.

#### Install polkit policy (first run only)

```bash
# Install the helper and polkit policy
sudo make install

# Or manually:
sudo mkdir -p /usr/libexec/frost-tune
sudo cp frost-tune-hid-helper /usr/libexec/frost-tune/
sudo cp packaging/linux/org.frosttune.hid.policy /usr/share/polkit-1/actions/
```

The first time you connect to a DAC, you'll be prompted to authenticate. Select "Authenticate" and enter your password. Access is cached for your session.

#### Troubleshooting

If you see "Authentication required" errors:

1. Verify the policy is installed: `ls /usr/share/polkit-1/actions/org.frosttune.hid.policy`
2. Verify the helper is installed: `ls /usr/libexec/frost-tune/frost-tune-hid-helper`
3. Re-run the install commands above

#### Legacy udev mode (optional)

If polkit is unavailable, you can still use udev rules:

```bash
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3302", ATTRS{idProduct}=="43e6", MODE="0666"' | sudo tee /etc/udev/rules.d/99-frosttune.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```
```

### 9. Add install targets

File: `Makefile` or `packaging/linux/` directory

```makefile
install:
	mkdir -p $(DESTDIR)/usr/libexec/frost-tune
	mkdir -p $(DESTDIR)/usr/share/polkit-1/actions
	install -m 755 frost-tune-hid-helper $(DESTDIR)/usr/libexec/frost-tune/
	install -m 644 packaging/linux/org.frosttune.hid.policy $(DESTDIR)/usr/share/polkit-1/actions/
```

## Safety & Security Guardrails

- Helper exposes only fixed DAC operations (no shell/command execution).
- Enforce existing payload clamp/validation before writes.
- Restrict device matching to known VID/PID map via `Device::from_vid_pid`.
- Keep rollback on verification mismatch exactly as today.
- Polkit policy enforces exact helper binary path (no path substitution attacks).
- 30-second execution window prevents helper relay.

## Cross-platform Behavior

| Platform | Direct HID | Polkit Helper |
|---------|-----------|--------------|
| Linux   | Try first  | Fallback     |
| Windows | Used      | Not used     |
| macOS   | TBD       | TBD          |

## Validation

- [ ] `cargo build` passes
- [ ] `cargo test` passes
- [ ] No udev rules required on Linux
- [ ] Connect triggers polkit auth prompt on first access
- [ ] Read/Write works after authentication
- [ ] Deny auth shows actionable error in UI
- [ ] Unplug/replug auto-detects and reconnects
- [ ] Rollback path still triggers on forced verify mismatch
- [ ] Helper binary correctly installed and invoked
- [ ] Polkit policy correctly installed and granting access