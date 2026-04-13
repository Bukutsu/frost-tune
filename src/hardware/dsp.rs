use crate::hardware::protocol::{
    OFFSET_FILTER_TYPE, OFFSET_FREQ_H, OFFSET_FREQ_L, OFFSET_GAIN_H, OFFSET_GAIN_L, OFFSET_INDEX,
    OFFSET_Q_H, OFFSET_Q_L,
};
use crate::models::{Filter, FilterType};
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
    vec![
        i_arr2[0],
        i_arr2[1],
        i_arr2[2],
        i_arr[1].wrapping_neg(),
        i_arr[2].wrapping_neg(),
    ]
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
    let gain_raw =
        (packet[OFFSET_GAIN_L] as u16) | ((packet[OFFSET_GAIN_H] as u16) << BYTE_BIT_SHIFT);

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

pub fn get_biquad_coefficients(filter: &Filter) -> (f64, f64, f64, f64, f64, f64) {
    let freq = filter.freq as f64;
    let gain = filter.gain;
    let q = filter.q;
    let a = 10_f64.powf(gain / 40.0);
    let omega = (freq * TAU) / DSP_SAMPLE_RATE;
    let sin_w = omega.sin();
    let cos_w = omega.cos();

    match filter.filter_type {
        FilterType::Peak => {
            let alpha = sin_w / (2.0 * q);
            let b0 = 1.0 + alpha * a;
            let b1 = -2.0 * cos_w;
            let b2 = 1.0 - alpha * a;
            let a0 = 1.0 + alpha / a;
            let a1 = -2.0 * cos_w;
            let a2 = 1.0 - alpha / a;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::LowShelf => {
            // Standard RBJ Shelf with Q
            let alpha = (sin_w / 2.0) * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).sqrt();
            let a_minus_1 = a - 1.0;
            let a_plus_1 = a + 1.0;
            let sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
            let b0 = a * (a_plus_1 - a_minus_1 * cos_w + sqrt_a_alpha);
            let b1 = 2.0 * a * (a_minus_1 - a_plus_1 * cos_w);
            let b2 = a * (a_plus_1 - a_minus_1 * cos_w - sqrt_a_alpha);
            let a0 = a_plus_1 + a_minus_1 * cos_w + sqrt_a_alpha;
            let a1 = -2.0 * (a_minus_1 + a_plus_1 * cos_w);
            let a2 = a_plus_1 + a_minus_1 * cos_w - sqrt_a_alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::HighShelf => {
            // Standard RBJ Shelf with Q
            let alpha = (sin_w / 2.0) * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).sqrt();
            let a_minus_1 = a - 1.0;
            let a_plus_1 = a + 1.0;
            let sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
            let b0 = a * (a_plus_1 + a_minus_1 * cos_w + sqrt_a_alpha);
            let b1 = -2.0 * a * (a_minus_1 + a_plus_1 * cos_w);
            let b2 = a * (a_plus_1 + a_minus_1 * cos_w - sqrt_a_alpha);
            let a0 = a_plus_1 - a_minus_1 * cos_w + sqrt_a_alpha;
            let a1 = 2.0 * (a_minus_1 - a_plus_1 * cos_w);
            let a2 = a_plus_1 - a_minus_1 * cos_w - sqrt_a_alpha;
            (b0, b1, b2, a0, a1, a2)
        }
    }
}

pub fn get_magnitude_response(filter: &Filter, f: f64) -> f64 {
    if !filter.enabled || filter.freq == 0 {
        return 0.0;
    }
    let (b0, b1, b2, a0, a1, a2) = get_biquad_coefficients(filter);
    let w = (f * TAU) / DSP_SAMPLE_RATE;
    let cos_w = w.cos();
    let cos_2w = (2.0 * w).cos();
    let sin_w = w.sin();
    let sin_2w = (2.0 * w).sin();

    let num_real = b0 + b1 * cos_w + b2 * cos_2w;
    let num_imag = -(b1 * sin_w + b2 * sin_2w);
    let den_real = a0 + a1 * cos_w + a2 * cos_2w;
    let den_imag = -(a1 * sin_w + a2 * sin_2w);

    let num_mag_sq = num_real * num_real + num_imag * num_imag;
    let den_mag_sq = den_real * den_real + den_imag * den_imag;

    10.0 * (num_mag_sq / den_mag_sq).log10()
}

pub fn calculate_total_response(filters: &[Filter], global_gain: i8, freqs: &[f64]) -> Vec<f64> {
    freqs
        .iter()
        .map(|&f| {
            let mut total_db = global_gain as f64;
            for filter in filters {
                total_db += get_magnitude_response(filter, f);
            }
            total_db
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mag = get_magnitude_response(&filter, 1000.0);
        assert!((mag - 6.0).abs() < 0.1);

        let mag_far = get_magnitude_response(&filter, 100.0);
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
        let mag_low = get_magnitude_response(&filter, 20.0);
        assert!((mag_low - 6.0).abs() < 0.5);

        let mag_high = get_magnitude_response(&filter, 10000.0);
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
        let freqs = vec![1000.0];
        let total = calculate_total_response(&filters, 0, &freqs);
        assert!((total[0] - 4.0).abs() < 0.1);

        let total_with_preamp = calculate_total_response(&filters, -3, &freqs);
        assert!((total_with_preamp[0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_iir_filter_produces_20_bytes() {
        let result = compute_iir_filter(1000.0, 5.0, 1.0);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_compute_iir_filter_with_zero_gain() {
        let result = compute_iir_filter(1000.0, 0.0, 1.0);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_compute_iir_filter_with_max_gain() {
        let result = compute_iir_filter(1000.0, 10.0, 1.0);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_convert_to_byte_array_length() {
        let result = convert_to_byte_array(0x12345678, 4);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_parse_filter_packet_short() {
        let result = parse_filter_packet(&[0u8; 10][..]);
        assert!(result.is_none());
    }

    #[test]
    fn test_quantizer_no_overflow() {
        let d_arr: Vec<f64> = vec![1.0, -1000.0, 1000.0];
        let d_arr2: Vec<f64> = vec![1.0, 2.0, 3.0];
        let result = quantizer(&d_arr, &d_arr2);
        assert_eq!(result.len(), 5);
    }
}
