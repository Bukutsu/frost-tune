// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Canonical biquad coefficient computation shared by USB packet building and graph rendering.

use crate::core::eq::{Filter, FilterType};
use std::f64::consts::TAU;

pub const DSP_SAMPLE_RATE: f64 = 96000.0;

/// Canonical biquad coefficient computation shared by USB packet building and graph rendering.
///
/// Returns `(b0, b1, b2, a0, a1, a2)` for the given filter parameters.
/// `FilterType::AllPass` (if ever added) would be an identity: `(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)`.
pub fn compute_biquad_coeffs(filter: &Filter) -> (f64, f64, f64, f64, f64, f64) {
    let freq = filter.freq as f64;
    let gain = filter.gain;
    let q = filter.q;
    let a_val = 10_f64.powf(gain / 40.0);
    let omega = (freq * TAU) / DSP_SAMPLE_RATE;
    let sin_w = omega.sin();
    let cos_w = omega.cos();

    match filter.filter_type {
        FilterType::Peak => {
            let alpha = sin_w / (2.0 * q);
            (
                1.0 + alpha * a_val,
                -2.0 * cos_w,
                1.0 - alpha * a_val,
                1.0 + alpha / a_val,
                -2.0 * cos_w,
                1.0 - alpha / a_val,
            )
        }
        FilterType::LowShelf => {
            let alpha = (sin_w / 2.0) * ((a_val + 1.0 / a_val) * (1.0 / q - 1.0) + 2.0).sqrt();
            let a_minus_1 = a_val - 1.0;
            let a_plus_1 = a_val + 1.0;
            let sqrt_a_alpha = 2.0 * a_val.sqrt() * alpha;
            (
                a_val * (a_plus_1 - a_minus_1 * cos_w + sqrt_a_alpha),
                2.0 * a_val * (a_minus_1 - a_plus_1 * cos_w),
                a_val * (a_plus_1 - a_minus_1 * cos_w - sqrt_a_alpha),
                a_plus_1 + a_minus_1 * cos_w + sqrt_a_alpha,
                -2.0 * (a_minus_1 + a_plus_1 * cos_w),
                a_plus_1 + a_minus_1 * cos_w - sqrt_a_alpha,
            )
        }
        FilterType::HighShelf => {
            let alpha = (sin_w / 2.0) * ((a_val + 1.0 / a_val) * (1.0 / q - 1.0) + 2.0).sqrt();
            let a_minus_1 = a_val - 1.0;
            let a_plus_1 = a_val + 1.0;
            let sqrt_a_alpha = 2.0 * a_val.sqrt() * alpha;
            (
                a_val * (a_plus_1 + a_minus_1 * cos_w + sqrt_a_alpha),
                -2.0 * a_val * (a_minus_1 + a_plus_1 * cos_w),
                a_val * (a_plus_1 + a_minus_1 * cos_w - sqrt_a_alpha),
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
    }
}
