// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Reusable IIR DSP math and quantization utility functions.

use crate::core::eq::FilterType;
use std::f64::consts::TAU;

pub const DSP_SAMPLE_RATE: f64 = 96000.0;
pub const QUANTIZER_SCALE: f64 = 1073741824.0;
pub const Q_FLOAT_TO_U16_DIVISOR: f64 = 256.0;
pub const GAIN_FLOAT_TO_U16_DIVISOR: f64 = 256.0;
pub const U16_WRAP_AROUND: i32 = 65536;
pub const GAIN_I16_THRESHOLD: i32 = 32767;
pub const BYTE_BIT_SHIFT: i32 = 8;

/// Quantizes filter parameters using a standard scale.
pub fn quantizer(d_arr: &[f64; 3], d_arr2: &[f64; 3]) -> [i32; 5] {
    let i_arr = [
        (d_arr[0] * QUANTIZER_SCALE).round() as i32,
        (d_arr[1] * QUANTIZER_SCALE).round() as i32,
        (d_arr[2] * QUANTIZER_SCALE).round() as i32,
    ];
    let i_arr2 = [
        (d_arr2[0] * QUANTIZER_SCALE).round() as i32,
        (d_arr2[1] * QUANTIZER_SCALE).round() as i32,
        (d_arr2[2] * QUANTIZER_SCALE).round() as i32,
    ];
    [
        i_arr2[0],
        i_arr2[1],
        i_arr2[2],
        i_arr[1].wrapping_neg(),
        i_arr[2].wrapping_neg(),
    ]
}

/// Computes IIR Biquad filter coefficients for Peak, LowShelf, HighShelf, HighPass, and LowPass.
pub fn compute_iir_filter(filter_type: FilterType, freq: f64, gain: f64, q: f64) -> [u8; 20] {
    let mut b_arr = [0u8; 20];
    let a = 10_f64.powf(gain / 40.0);
    let omega = (freq * TAU) / DSP_SAMPLE_RATE;
    let sin_w = omega.sin();
    let cos_w = omega.cos();

    let (b0, b1, b2, a0, a1, a2) = match filter_type {
        FilterType::Peak => {
            let alpha = sin_w / (2.0 * q);
            (
                1.0 + alpha * a,
                -2.0 * cos_w,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w,
                1.0 - alpha / a,
            )
        }
        FilterType::LowShelf => {
            let alpha = (sin_w / 2.0) * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).sqrt();
            let a_minus_1 = a - 1.0;
            let a_plus_1 = a + 1.0;
            let sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
            (
                a * (a_plus_1 - a_minus_1 * cos_w + sqrt_a_alpha),
                2.0 * a * (a_minus_1 - a_plus_1 * cos_w),
                a * (a_plus_1 - a_minus_1 * cos_w - sqrt_a_alpha),
                a_plus_1 + a_minus_1 * cos_w + sqrt_a_alpha,
                -2.0 * (a_minus_1 + a_plus_1 * cos_w),
                a_plus_1 + a_minus_1 * cos_w - sqrt_a_alpha,
            )
        }
        FilterType::HighShelf => {
            let alpha = (sin_w / 2.0) * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).sqrt();
            let a_minus_1 = a - 1.0;
            let a_plus_1 = a + 1.0;
            let sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
            (
                a * (a_plus_1 + a_minus_1 * cos_w + sqrt_a_alpha),
                -2.0 * a * (a_minus_1 + a_plus_1 * cos_w),
                a * (a_plus_1 - a_minus_1 * cos_w - sqrt_a_alpha),
                a_plus_1 - a_minus_1 * cos_w + sqrt_a_alpha,
                2.0 * (a_minus_1 - a_plus_1 * cos_w),
                a_plus_1 - a_minus_1 * cos_w - sqrt_a_alpha,
            )
        }
        FilterType::HighPass => {
            let alpha = sin_w / (2.0 * q);
            (
                (1.0 + cos_w) / 2.0,
                -(1.0 + cos_w),
                (1.0 + cos_w) / 2.0,
                1.0 + alpha,
                -2.0 * cos_w,
                1.0 - alpha,
            )
        }
        FilterType::LowPass => {
            let alpha = sin_w / (2.0 * q);
            (
                (1.0 - cos_w) / 2.0,
                1.0 - cos_w,
                (1.0 - cos_w) / 2.0,
                1.0 + alpha,
                -2.0 * cos_w,
                1.0 - alpha,
            )
        }
    };

    let quantizer_data = quantizer(&[1.0, a1 / a0, a2 / a0], &[b0 / a0, b1 / a0, b2 / a0]);

    for (i, &value) in quantizer_data.iter().enumerate() {
        b_arr[i * 4] = (value & 0xFF) as u8;
        b_arr[i * 4 + 1] = ((value >> BYTE_BIT_SHIFT) & 0xFF) as u8;
        b_arr[i * 4 + 2] = ((value >> (BYTE_BIT_SHIFT * 2)) & 0xFF) as u8;
        b_arr[i * 4 + 3] = ((value >> (BYTE_BIT_SHIFT * 3)) & 0xFF) as u8;
    }

    b_arr
}

/// Converts a 32-bit integer to a 2-byte array (little-endian).
pub fn convert_to_2byte_array(value: i32) -> [u8; 2] {
    [
        (value & 0xFF) as u8,
        ((value >> BYTE_BIT_SHIFT) & 0xFF) as u8,
    ]
}
