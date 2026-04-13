use crate::models::{Filter, FilterType};
use crate::hardware::protocol::{OFFSET_INDEX, OFFSET_FREQ_L, OFFSET_FREQ_H, OFFSET_Q_L, OFFSET_Q_H, OFFSET_GAIN_L, OFFSET_GAIN_H, OFFSET_FILTER_TYPE};
use std::f64::consts::TAU;

const DSP_SAMPLE_RATE: f64 = 96000.0;
const QUANTIZER_SCALE: f64 = 1073741824.0;
const Q_FLOAT_TO_U16_DIVISOR: f64 = 256.0;
const GAIN_FLOAT_TO_U16_DIVISOR: f64 = 256.0;
const U16_WRAP_AROUND: i32 = 65536;
const GAIN_I16_THRESHOLD: i32 = 32767;
const BYTE_BIT_SHIFT: i32 = 8;

pub fn quantizer(d_arr: &[f64], d_arr2: &[f64]) -> Vec<i32> {
    let i_arr: Vec<i32> = d_arr
        .iter()
        .map(|d| (d * QUANTIZER_SCALE).round() as i32)
        .collect();
    let i_arr2: Vec<i32> = d_arr2
        .iter()
        .map(|d| (d * QUANTIZER_SCALE).round() as i32)
        .collect();
    vec![i_arr2[0], i_arr2[1], i_arr2[2], -i_arr[1], -i_arr[2]]
}

pub fn compute_iir_filter(freq: f64, gain: f64, q: f64) -> Vec<u8> {
    let mut b_arr = vec![0u8; 20];
    let sqrt = (10_f64.powf(gain / 20.0)).sqrt();
    let omega = (freq * TAU) / DSP_SAMPLE_RATE;
    let sin_omega_over_2q = omega.sin() / (2.0 * q);
    let omega_correction = sin_omega_over_2q * sqrt;
    let denom = (sin_omega_over_2q / sqrt) + 1.0;

    let quantizer_data = quantizer(
        &[
            1.0,
            (omega.cos() * -2.0) / denom,
            (1.0 - sin_omega_over_2q / sqrt) / denom,
        ][..],
        &[
            (omega_correction + 1.0) / denom,
            omega.cos() * -2.0 / denom,
            (1.0 - omega_correction) / denom,
        ][..],
    );

    for (i, &value) in quantizer_data.iter().enumerate() {
        b_arr[i * 4] = (value & 0xFF) as u8;
        b_arr[i * 4 + 1] = ((value >> BYTE_BIT_SHIFT) & 0xFF) as u8;
        b_arr[i * 4 + 2] = ((value >> BYTE_BIT_SHIFT * 2) & 0xFF) as u8;
        b_arr[i * 4 + 3] = ((value >> BYTE_BIT_SHIFT * 3) & 0xFF) as u8;
    }

    b_arr
}

pub fn convert_to_byte_array(value: i32, length: usize) -> Vec<u8> {
    let mut arr = Vec::with_capacity(length);
    for i in 0..length {
        arr.push(((value >> (BYTE_BIT_SHIFT * i as i32)) & 0xFF) as u8);
    }
    arr
}

pub fn parse_filter_packet(packet: &[u8]) -> Option<Filter> {
    if packet.len() < 34 {
        return None;
    }

    let filter_index = packet[OFFSET_INDEX];
    let freq = (packet[OFFSET_FREQ_L] as u16) | ((packet[OFFSET_FREQ_H] as u16) << BYTE_BIT_SHIFT);
    let q_raw = (packet[OFFSET_Q_L] as u16) | ((packet[OFFSET_Q_H] as u16) << BYTE_BIT_SHIFT);
    let gain_raw = (packet[OFFSET_GAIN_L] as u16) | ((packet[OFFSET_GAIN_H] as u16) << BYTE_BIT_SHIFT);

    let gain_from_device = if gain_raw > GAIN_I16_THRESHOLD as u16 {
        (gain_raw as i32 - U16_WRAP_AROUND) as i16
    } else {
        gain_raw as i16
    };

    let q = ((q_raw as f64) / Q_FLOAT_TO_U16_DIVISOR * 100.0).round() / 100.0;
    let gain = ((gain_from_device as f64) / GAIN_FLOAT_TO_U16_DIVISOR * 100.0).round() / 100.0;
    let filter_type = FilterType::from(packet[OFFSET_FILTER_TYPE]);
    let enabled = !(freq == 0 && gain_from_device == 0);

    Some(Filter {
        index: filter_index,
        enabled,
        freq,
        gain,
        q,
        filter_type,
    })
}
