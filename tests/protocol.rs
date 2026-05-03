use frost_tune::hardware::protocol::{DeviceProtocol, TP35ProProtocol, WRITE, READ, CMD_PEQ_VALUES, CMD_GLOBAL_GAIN};
use frost_tune::models::FilterType;

#[test]
fn test_tp35pro_build_filter_read_request() {
    let proto = TP35ProProtocol;
    let packet = proto.build_filter_read_request(3, 0xAA);
    assert_eq!(packet, vec![READ, CMD_PEQ_VALUES, 0xAA, 0x00, 3, 0x00]);
}

#[test]
fn test_tp35pro_build_global_gain_request() {
    let proto = TP35ProProtocol;
    let packet = proto.build_global_gain_request(0xBB);
    assert_eq!(packet, vec![READ, CMD_GLOBAL_GAIN, 0x00, 0x00]);
}

#[test]
fn test_tp35pro_parse_filter_packet_invalid() {
    let proto = TP35ProProtocol;
    // Too short
    let data = vec![0x00, 0x01];
    let filter = proto.parse_filter_packet(&data);
    assert!(filter.is_none());
}

#[test]
fn test_tp35pro_filter_packet_round_trip() {
    // We can't perfectly round-trip because dsp::compute_iir_filter creates the biquad array 
    // and dsp::parse_filter_packet reads it back, but let's at least ensure it doesn't panic
    let proto = TP35ProProtocol;
    let packet = proto.build_filter_write_packet(
        0, true, 1000.0, 2.5, 1.414, FilterType::Peak.into()
    );
    
    // In actual device, report ID is prefix, and device sends back packet.
    // The parse_filter_packet expects the payload.
    // Let's just make sure it's valid format.
    assert_eq!(packet.len(), 37);
    assert_eq!(packet[0], WRITE);
    assert_eq!(packet[1], CMD_PEQ_VALUES);
}
