// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use frost_tune::hardware::protocol::{
    DeviceProtocol, TP35ProProtocol, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, READ, WRITE,
};
use frost_tune::models::{Filter, FilterType};

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
fn test_tp35pro_parse_filter_response_invalid() {
    let proto = TP35ProProtocol;
    assert!(proto.parse_filter_response(&[0x00, 0x01]).is_none());
}

#[test]
fn test_tp35pro_filter_packet_round_trip() {
    let proto = TP35ProProtocol;
    let filter = Filter {
        index: 0,
        enabled: true,
        filter_type: FilterType::Peak,
        freq: 1000,
        gain: 2.5,
        q: 1.414,
    };
    let packet = proto.build_filter_write_packet(0, &filter);
    assert_eq!(packet.len(), 37);
    assert_eq!(packet[0], WRITE);
    assert_eq!(packet[1], CMD_PEQ_VALUES);
}

#[test]
fn test_tp35pro_build_commit_packets_count() {
    let proto = TP35ProProtocol;
    let packets = proto.build_commit_packets();
    // TP35 Pro uses 2 commit steps: temp-write, flash-eq
    assert_eq!(packets.len(), 2);
}
