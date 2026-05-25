# Adding a New Device to frost-tune

This guide walks you through every step needed to add support for a new USB DAC.
If something is unclear, look at `src/hardware/devices/tp35pro.rs` as the reference
implementation.

---

## Prerequisites: Capture the Wire Protocol

You need to know how the device talks over USB HID before you write any code.

**Tools:**
- **Linux:** `usbmon` + Wireshark with USB capture, or `hidraw` + a Python script with `hid` lib
- **Windows:** [USBPcap](https://desowin.org/usbpcap/) + Wireshark, or [API Monitor](http://www.rohitab.com/apimonitor)
- **Alternative:** Read the [devicePEQ source](https://github.com/jeromeof/devicePEQ) — it has reverse-engineered protocols for FiiO, Walkplay, KTMicro, Moondrop, and Qudelix devices

**What to capture:**
1. Open the official app, connect the device
2. Read EQ settings → observe the request/response packets
3. Change one filter, write EQ → observe the write packets
4. Note the report ID (first byte of every outgoing packet)
5. Note how filter index and any nonce/sequence number appear in responses

---

## How to Decode Unknown USB HID Packets

This section covers the reverse-engineering **methodology** — how to work from raw
hex dumps to a working protocol implementation. It matters more than any code
skeleton because every USB DAC uses a different wire format.

### The Golden Rule: Change One Parameter at a Time

In the official app, change **exactly one setting** between captures:

```
Capture A → set band 0 to 1000 Hz,  +5.0 dB gain, Q 1.0, Peak
Capture B → set band 0 to 2000 Hz,  +5.0 dB gain, Q 1.0, Peak   (change freq only)
Capture C → set band 0 to 1000 Hz, +10.0 dB gain, Q 1.0, Peak   (change gain only)
Capture D → set band 0 to 1000 Hz,  +5.0 dB gain, Q 5.0, Peak   (change Q only)
Capture E → set band 0 to 1000 Hz,  +5.0 dB gain, Q 1.0, LowShelf (change type only)
```

Compare `Capture A` to `Capture B` side-by-side. The bytes that changed are the
**frequency field**. Repeat for gain, Q, and filter type. Each field is almost always:

| Field | Typical size | Typical encoding |
|-------|-------------|-----------------|
| Frequency | 2 bytes (u16, little-endian) | Raw Hz (1000 = `[0xE8, 0x03]`) |
| Gain | 2 bytes (signed, little-endian) | `gain * 10` (`+5.0 dB` = `50` → `[0x32, 0x00]`) or `gain * 256` (TI/DSP chips) |
| Q | 2 bytes (u16, little-endian) | `q * 100` (`Q 1.0` = `100` → `[0x64, 0x00]`) or `q * 256` |
| Filter type | 1 byte | Varies per manufacturer (build a mapping table) |
| Band index | 1 byte | Usually byte 4 or byte 5 in the packet |
| Nonce | 1 byte | Usually byte 2 or byte 3; device echoes it back in responses |

### How to Determine the Fixed-Point Multiplier

Pick a known value, check the raw bytes, and compute backwards:

```
Gain = 5.0 dB → raw bytes = [0x32, 0x00] (50 as u16 LE)
Scale factor = 50 / 5.0 = 10   → multiplier is 10×

Gain = 5.0 dB → raw bytes = [0x00, 0x05] (1280 as u16 LE)
Scale factor = 1280 / 5.0 = 256 → multiplier is 256×
```

Common multipliers: **10×** (most Chinese DACs, FiiO), **100×** (some KTMicro chips),
**256×** (TI/ADI DSP chips, Walkplay family). Test with a simple value like +5.0 dB or
Q 1.0 and compute the ratio.

### How to Build a Filter Type Mapping Table

The `FilterType` enum uses its own u8 encoding (1=LowShelf, 2=Peak, 3=HighShelf,
4=HighPass, 5=LowPass). Your device will use **different** byte values. Do not call
`FilterType::from(device_byte)` — the byte domains are unrelated.

Instead, set each filter type in the official app and capture the write packet.
Read the filter-type byte from the packet and build a mapping function:

```
Official app: Peak     → capture shows byte 0x10 → FilterType::Peak
Official app: LowShelf → capture shows byte 0x20 → FilterType::LowShelf
...and so on.
```

Write a function like:
```rust
fn device_byte_to_filter_type(b: u8) -> FilterType {
    match b {
        0x10 => FilterType::Peak,
        0x20 => FilterType::LowShelf,
        0x30 => FilterType::HighShelf,
        0x40 => FilterType::HighPass,
        0x50 => FilterType::LowPass,
        _ => FilterType::Peak,
    }
}
```

### How to Find the Commit Sequence

Some devices write filters directly to flash (no commit step — `build_commit_packets`
returns `vec![]`). Others hold settings in volatile memory and need a "save" command.

To find the commit sequence:
1. In the official app, change one filter. Capture traffic while editing — you'll see
   write packets streaming.
2. Press **Save** / **Apply** / **OK** in the official app.
3. Capture traffic. You'll see 1-3 additional packets that did NOT appear during
   editing. These are your commit packets.
4. Note the delay between commit packets. The official app typically pauses 100-2000 ms
   for flash programming. This is your `write_timing().commit_step_ms`.

### How to Find the Init Sequence

Some devices need a "wake" ping before they'll accept read/write commands.
In a fresh capture:
1. Look at the first few packets the official app sends after opening the device.
2. They often look like `[0x80, 0x0C, 0x00]` or `[0x01, 0x00]` — short packets
   that appear before any EQ traffic.
3. These are your `build_init_packets()`.

### Known Protocol Families

If your device's packet structure resembles one of these, it likely shares the
same chip family. The offsets and multipliers will be identical or nearly so.

| Family | Chip vendor | Example devices | Key traits |
|--------|------------|----------------|-----------|
| **Walkplay** | Walkplay DSP | EPZ TP35 Pro, some Moondrop | Command byte at [1], nonce at [2], index at [4], 20-byte biquad block at [7..27], then freq/Q/gain/filter-type at [27..34]. **Host computes biquad coefficients.** gain*256, q*256 |
| **FiiO** | Savitech / XMOS | FiiO KA5, KA17, BTR series | Command byte at [0], index at [4], freq/gain/Q packed in 8-byte filter block. **Device takes (freq, gain, Q) directly.** gain*10, q*100 |
| **KTMicro** | KTMicro KT0200 series | Some budget dongles | Single fixed-size packet for all 10 bands. **Bulk write, no per-band reads.** gain*100 |
| **CB5100** | Comtrue CB5100 | Moondrop Dawn Pro | 5 bands. Use `CMD_READ_FILTER = 0x32`, filter type byte in a flags bitfield rather than a dedicated byte. gain*1 (integer), q*100 |

If your packet looks nothing like any of these, start from scratch with the
one-parameter-at-a-time method above.

---

## Step 1: Implement `DeviceProtocol`

Create a new file `src/hardware/devices/<vendor>/mod.rs`. Replace `<vendor>` with a
short lowercase identifier (e.g. `fiio`, `ktmicro`, `moondrop`).

```
src/hardware/devices/
  tp35pro.rs          ← existing reference implementation
  <vendor>/
    mod.rs            ← your DeviceProtocol + DeviceProfile
    constants.rs      ← (optional) wire constants if there are many
    dsp.rs            ← (optional) DSP math if host must compute biquad coefficients
```

**Minimal skeleton (`mod.rs`):**

```rust
use crate::core::device::protocol::DeviceProtocol;
use crate::core::device::profile::DeviceProfile;
use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};
use crate::core::eq::Filter;

// ── Wire constants ────────────────────────────────────────────────────────────

pub const REPORT_ID: u8 = 0xXX;  // first byte of every write; 0x00 if none

const CMD_READ_FILTER: u8  = 0xXX;
const CMD_WRITE_FILTER: u8 = 0xXX;
const CMD_READ_GAIN: u8    = 0xXX;
const CMD_WRITE_GAIN: u8   = 0xXX;

const READ:  u8 = 0xXX;
const WRITE: u8 = 0xXX;
const END:   u8 = 0x00;

// ── Protocol implementation ───────────────────────────────────────────────────

pub struct MyDeviceProtocol;

impl DeviceProtocol for MyDeviceProtocol {
    fn report_id(&self) -> u8 { REPORT_ID }

    fn build_init_packets(&self) -> Vec<Vec<u8>> {
        // Packets to wake the device at the start of every operation.
        // Return vec![] if the device needs no init sequence.
        vec![vec![READ, CMD_VERSION, END]]
    }

    fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8> {
        // Ask the device to send back the filter at `index`.
        // Include `nonce` so you can match the response.
        vec![READ, CMD_READ_FILTER, nonce, 0x00, index, END]
    }

    fn matches_filter_response(&self, data: &[u8], index: u8, nonce: u8) -> bool {
        // Return true only if `data` is the response to our filter read for `index`/`nonce`.
        // `data` has already had the report-ID prefix stripped.
        // MUST NOT panic on short packets.
        data.len() >= 10
            && data[0] == READ
            && data[1] == CMD_READ_FILTER
            && data[2] == nonce       // your nonce offset may differ
            && data[4] == index       // your index offset may differ
    }

    fn parse_filter_response(&self, data: &[u8]) -> Option<Filter> {
        // Extract filter parameters from a packet that passed `matches_filter_response`.
        // Return None if the packet is too short or malformed.
        if data.len() < 10 { return None; }
        Some(Filter {
            index:       data[4],
            enabled:     data[5] != 0,
            freq:        u16::from_le_bytes([data[6], data[7]]),
            gain:        (i16::from_le_bytes([data[8], data[9]]) as f64) / 10.0,
            q:           u16::from_le_bytes([data[10], data[11]]) as f64 / 100.0,
            filter_type: device_byte_to_filter_type(data[12]),
        })
    }

    fn build_filter_write_packet(&self, index: u8, filter: &Filter) -> Vec<u8> {
        let filter_type_byte: u8 = filter_type_to_device_byte(filter.filter_type);
        let gain_raw = (filter.gain * 10.0).round() as i16;
        let q_raw = (filter.q * 100.0).round() as u16;
        vec![
            WRITE, CMD_WRITE_FILTER,
            index,
            (filter.freq & 0xFF) as u8, (filter.freq >> 8) as u8,
            (gain_raw & 0xFF) as u8, (gain_raw >> 8) as u8,
            (q_raw & 0xFF) as u8, (q_raw >> 8) as u8,
            filter_type_byte,
            END,
        ]
    }

    fn build_global_gain_request(&self, nonce: u8) -> Vec<u8> {
        vec![READ, CMD_READ_GAIN, nonce, END]
    }

    fn matches_global_gain_response(&self, data: &[u8], _nonce: u8) -> bool {
        // Many devices don't include the nonce in the gain response — that's fine.
        data.len() >= 6 && data[0] == READ && data[1] == CMD_READ_GAIN
    }

    fn parse_global_gain_response(&self, data: &[u8]) -> Option<i8> {
        if data.len() > 4 { Some(data[4] as i8) } else { None }
    }

    fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8> {
        vec![WRITE, CMD_WRITE_GAIN, gain as u8, END]
    }

    fn build_commit_packets(&self) -> Vec<Vec<u8>> {
        // Return the ordered sequence of packets to persist EQ to flash.
        // Each Vec<u8> is one HID report payload.
        // Return vec![] if the device writes directly to flash (no commit step).
        vec![
            vec![WRITE, 0xXX, END],  // temp-write or equivalent
            vec![WRITE, 0xXX, END],  // flash-eq or equivalent
        ]
    }
}
```

Note: The TP35 Pro requires host-side biquad coefficient computation because its
Walkplay DSP chip accepts pre-computed coefficients instead of raw (freq, gain, Q).
If your device is the same, see `src/hardware/devices/tp35pro.rs` for the
`compute_iir_filter()` pattern. Most devices take (freq, gain, Q) directly.

For the full set of methods available on `DeviceProtocol`, see
`src/core/device/protocol.rs`.

### Filter type byte mapping

`FilterType`'s internal u8 encoding (1=LowShelf, 2=Peak, 3=HighShelf, 4=HighPass,
5=LowPass) is **app-internal** and will not match your device's wire format. You
must write two mapping functions — one for reads, one for writes:

```rust
fn device_byte_to_filter_type(b: u8) -> FilterType {
    match b {
        0x10 => FilterType::Peak,      // your device's wire byte for Peak
        0x20 => FilterType::LowShelf,  // your device's wire byte for LowShelf
        _    => FilterType::Peak,      // default for unknown bytes
    }
}

fn filter_type_to_device_byte(ft: FilterType) -> u8 {
    match ft {
        FilterType::Peak      => 0x10,
        FilterType::LowShelf  => 0x20,
        FilterType::HighShelf => 0x30,
        FilterType::HighPass  => 0x40,
        FilterType::LowPass   => 0x50,
    }
}
```

The TP35 Pro happens to use the same byte encoding as the app, which is why
its `parse_filter_packet` calls `FilterType::from()`. This is a coincidence,
not the norm. Most devices will need explicit mapping functions.

An unrecognised byte in `device_byte_to_filter_type` defaults to `Peak`; your
round-trip tests will catch this if you include a filter-type assertion.

---

## Step 2: Implement `DeviceProfile` and register

Add a `DeviceProfile` implementation in the same file:

```rust
pub struct MyDeviceProfile;

impl DeviceProfile for MyDeviceProfile {
    fn name(&self) -> &'static str {
        "Manufacturer Model Name"
    }

    fn vendor_id(&self) -> u16 {
        0x1234   // USB Vendor ID (from lsusb or USBPcap)
    }

    fn product_id(&self) -> u16 {
        0x5678   // USB Product ID
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 10,
            global_gain_range: (-10, 6),
            band_gain_range: (-10.0, 10.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: true,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(MyDeviceProtocol)
    }
}
```

**Finding the right `FilterTypeFlags`:** test which filter types the device accepts.
Sending an unsupported type will either silently misinterpret the value or be ignored.
When in doubt, start with `FilterTypeFlags::PEAK` only.

**`supports_per_band_enable`:** this flag controls whether the UI shows enable
toggles per band. When you set it `true`, your `build_filter_write_packet` **must**
also honour `filter.enabled`:

```rust
fn build_filter_write_packet(&self, index: u8, filter: &Filter) -> Vec<u8> {
    // If the device has an on-wire enable bit:
    let enable_byte: u8 = if filter.enabled { 0x01 } else { 0x00 };
    vec![WRITE, CMD_WRITE_FILTER, index, enable_byte, ...]

    // If the device has no enable bit but you want toggles to silence a band,
    // write gain = 0 when disabled (the UI will already grey the band out):
    let gain = if filter.enabled { filter.gain } else { 0.0 };
    // ... build packet using gain
}
```

When `supports_per_band_enable: false`, `PEQData::clamp_to_capabilities` zeroes
the gain of disabled bands before the payload reaches the protocol layer, so your
write packet does not need to handle `filter.enabled` at all (as in the TP35 Pro).

Then register in `src/hardware/registry.rs`:

```rust
pub const REGISTRY: &[&dyn DeviceProfile] = &[
    &crate::hardware::devices::tp35pro::TP35ProProfile,
    &crate::hardware::devices::myvendor::MyDeviceProfile,  // ← add this line
];
```

---

## Step 3: Register the module

Open `src/hardware/devices/mod.rs` and add:

```rust
pub mod myvendor;
```

---

## Step 4: Add a udev rule (Linux only)

Open `packaging/udev/99-frost-tune.rules` and add:

```
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", TAG+="uaccess"
```

Replace `1234` and `5678` with your device's VID and PID in lowercase hex (no `0x` prefix).

---

## Step 5: Write tests

At minimum, test the following in your `mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_filter_response_accepts_valid_packet() {
        let proto = MyDeviceProtocol;
        // Build a packet that your device would actually send
        let packet = /* construct a valid response */;
        assert!(proto.matches_filter_response(&packet, 0, 0x01));
    }

    #[test]
    fn matches_filter_response_rejects_wrong_index() {
        let proto = MyDeviceProtocol;
        let packet = /* valid response for index 0 */;
        assert!(!proto.matches_filter_response(&packet, 1, 0x01));
    }

    #[test]
    fn matches_filter_response_rejects_short_packet() {
        let proto = MyDeviceProtocol;
        assert!(!proto.matches_filter_response(&[0x00, 0x01], 0, 1));
    }

    #[test]
    fn build_commit_packets_nonempty() {
        let proto = MyDeviceProtocol;
        // Most devices need at least one commit packet to persist EQ
        assert!(!proto.build_commit_packets().is_empty());
    }

    #[test]
    fn filter_round_trip() {
        // Build a write packet, then parse a simulated response with the same values
        // and verify they match within the device's precision.
    }
}
```

Run with `cargo test --all-targets`.

---

## Step 6: Verify end-to-end

Run the full pre-push checklist (see `CLAUDE.md`):

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --all-targets --locked
cargo test --all-targets --locked
```

---

## Common pitfalls

| Problem | Likely cause |
|---------|-------------|
| `matches_filter_response` never fires | Wrong command byte or index offset — add logging to compare what the device actually sends |
| Filters read as all zeros after write | Missing or wrong commit packet sequence — capture the official app's commit traffic |
| Q or gain values slightly off | Check the fixed-point scale factor: devices use 10×, 100×, or 256× multipliers |
| Device disconnects after write | Some devices need `disconnectOnSave: true` equivalent — add a post-commit delay or re-init |
| Host-side biquad math needed | The TP35 Pro is unusual. Most devices take (freq, gain, Q) directly. Only implement `compute_iir_filter` if packet capture shows raw coefficient bytes |
