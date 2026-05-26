// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! DSP utilities for frequency-response visualization.
//!
//! The canonical biquad coefficient computation lives in [`crate::core::eq::iir_math::compute_biquad_coeffs`].
//! This module wraps it for magnitude-response calculation used by the UI graph renderer.

use crate::core::eq::iir_math::compute_biquad_coeffs;
use crate::core::Filter;
use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct PrecomputedFreq {
    pub cos_w: f64,
    pub cos_2w: f64,
    pub sin_w: f64,
    pub sin_2w: f64,
}

impl PrecomputedFreq {
    pub fn new(f: f64, dsp_sample_rate: f64) -> Self {
        let w = (f * TAU) / dsp_sample_rate;
        Self {
            cos_w: w.cos(),
            cos_2w: (2.0 * w).cos(),
            sin_w: w.sin(),
            sin_2w: (2.0 * w).sin(),
        }
    }
}

/// Canonical biquad coefficients for a filter.
///
/// Delegates to [`crate::core::eq::iir_math::compute_biquad_coeffs`] — the single shared implementation.
pub fn get_biquad_coefficients(
    filter: &Filter,
    dsp_sample_rate: f64,
) -> (f64, f64, f64, f64, f64, f64) {
    compute_biquad_coeffs(filter, dsp_sample_rate)
}

pub fn get_magnitude_response_with_coeffs(
    b0: f64,
    b1: f64,
    b2: f64,
    a0: f64,
    a1: f64,
    a2: f64,
    f: f64,
    dsp_sample_rate: f64,
) -> f64 {
    let pf = PrecomputedFreq::new(f, dsp_sample_rate);
    get_magnitude_response_with_precomputed(b0, b1, b2, a0, a1, a2, &pf)
}

pub fn get_magnitude_response_with_precomputed(
    b0: f64,
    b1: f64,
    b2: f64,
    a0: f64,
    a1: f64,
    a2: f64,
    pf: &PrecomputedFreq,
) -> f64 {
    let num_real = b0 + b1 * pf.cos_w + b2 * pf.cos_2w;
    let num_imag = -(b1 * pf.sin_w + b2 * pf.sin_2w);
    let den_real = a0 + a1 * pf.cos_w + a2 * pf.cos_2w;
    let den_imag = -(a1 * pf.sin_w + a2 * pf.sin_2w);

    let num_mag_sq = num_real * num_real + num_imag * num_imag;
    let den_mag_sq = den_real * den_real + den_imag * den_imag;

    10.0 * (num_mag_sq / den_mag_sq).log10()
}

type BiquadCoeffs = (f64, f64, f64, f64, f64, f64);

pub fn get_magnitude_response(filter: &Filter, f: f64, dsp_sample_rate: f64) -> f64 {
    if filter.freq == 0 || !filter.enabled {
        return 0.0;
    }
    let (b0, b1, b2, a0, a1, a2) = get_biquad_coefficients(filter, dsp_sample_rate);
    get_magnitude_response_with_coeffs(b0, b1, b2, a0, a1, a2, f, dsp_sample_rate)
}

pub fn calculate_total_response(
    filters: &[Filter],
    global_gain: i8,
    freqs: &[PrecomputedFreq],
    dsp_sample_rate: f64,
) -> Vec<f64> {
    let precomputed_coeffs: Vec<Option<BiquadCoeffs>> = filters
        .iter()
        .map(|f| {
            if f.freq == 0 || !f.enabled {
                None
            } else {
                Some(get_biquad_coefficients(f, dsp_sample_rate))
            }
        })
        .collect();

    freqs
        .iter()
        .map(|pf| {
            let mut total_db = global_gain as f64;
            for (b0, b1, b2, a0, a1, a2) in precomputed_coeffs.iter().flatten() {
                total_db +=
                    get_magnitude_response_with_precomputed(*b0, *b1, *b2, *a0, *a1, *a2, pf);
            }
            total_db
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Filter, FilterType};

    #[test]
    fn test_peak_filter_magnitude_response() {
        let filter = Filter {
            index: 0,
            enabled: true,
            freq: 1000,
            gain: 6.0,
            q: 1.0,
            filter_type: FilterType::Peak,
        };
        let mag = get_magnitude_response(&filter, 1000.0, 96000.0);
        assert!((mag - 6.0).abs() < 0.1);

        let mag_far = get_magnitude_response(&filter, 100.0, 96000.0);
        assert!(mag_far.abs() < 0.5);
    }

    #[test]
    fn test_low_shelf_magnitude_response() {
        let filter = Filter {
            index: 0,
            enabled: true,
            freq: 100,
            gain: 6.0,
            q: 1.0,
            filter_type: FilterType::LowShelf,
        };
        let mag_low = get_magnitude_response(&filter, 20.0, 96000.0);
        assert!((mag_low - 6.0).abs() < 0.5);

        let mag_high = get_magnitude_response(&filter, 10000.0, 96000.0);
        assert!(mag_high.abs() < 0.1);
    }

    #[test]
    fn test_total_response_summation() {
        let filters = vec![
            Filter {
                index: 0,
                enabled: true,
                freq: 1000,
                gain: 6.0,
                q: 1.0,
                filter_type: FilterType::Peak,
            },
            Filter {
                index: 1,
                enabled: true,
                freq: 1000,
                gain: -2.0,
                q: 1.0,
                filter_type: FilterType::Peak,
            },
        ];
        let freqs = vec![PrecomputedFreq::new(1000.0, 96000.0)];
        let total = calculate_total_response(&filters, 0, &freqs, 96000.0);
        assert!((total[0] - 4.0).abs() < 0.1);

        let total_with_preamp = calculate_total_response(&filters, -3, &freqs, 96000.0);
        assert!((total_with_preamp[0] - 1.0).abs() < 0.1);
    }
}
