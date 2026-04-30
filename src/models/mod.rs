pub mod constants;
pub mod device;
pub mod filter;
pub mod ipc;

pub use constants::*;
pub use device::*;
pub use filter::*;
pub use ipc::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_gain_clamp_at_max() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = 15.0;
        filter.clamp();
        assert_eq!(filter.gain, MAX_BAND_GAIN);
    }

    #[test]
    fn test_band_gain_clamp_at_min() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = -15.0;
        filter.clamp();
        assert_eq!(filter.gain, MIN_BAND_GAIN);
    }

    #[test]
    fn test_band_gain_unchanged_when_in_bounds() {
        let mut filter = Filter::enabled(0, true);
        filter.gain = 5.0;
        filter.clamp();
        assert_eq!(filter.gain, 5.0);
    }

    #[test]
    fn test_global_gain_clamp_max() {
        let mut payload = PushPayload {
            filters: vec![],
            global_gain: Some(15),
        };
        payload.clamp();
        assert_eq!(payload.global_gain, Some(MAX_GLOBAL_GAIN));
    }

    #[test]
    fn test_global_gain_clamp_min() {
        let mut payload = PushPayload {
            filters: vec![],
            global_gain: Some(-15),
        };
        payload.clamp();
        assert_eq!(payload.global_gain, Some(MIN_GLOBAL_GAIN));
    }

    #[test]
    fn test_push_payload_valid_with_10_bands() {
        let filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, true)).collect();
        let payload = PushPayload {
            filters: filters.clone(),
            global_gain: Some(5),
        };
        assert!(payload.is_valid().is_ok());
    }

    #[test]
    fn test_push_payload_invalid_with_wrong_band_count() {
        let filters = vec![Filter::enabled(0, false)];
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_push_payload_invalid_with_disabled_band() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, true)).collect();
        filters[2].enabled = false;
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_push_payload_clamp_enables_all_bands() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        let mut payload = PushPayload {
            filters: std::mem::take(&mut filters),
            global_gain: Some(0),
        };
        payload.clamp();
        assert!(payload.filters.iter().all(|f| f.enabled));
    }

    #[test]
    fn test_push_payload_invalid_with_nan_gain() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        filters[0].gain = f64::NAN;
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_push_payload_invalid_with_inf_q() {
        let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
        filters[0].q = f64::INFINITY;
        let payload = PushPayload {
            filters,
            global_gain: Some(0),
        };
        assert!(payload.is_valid().is_err());
    }

    #[test]
    fn test_default_filter_has_correct_index() {
        for i in 0..10 {
            let filter = Filter::enabled(i, true);
            assert_eq!(filter.index, i as u8);
        }
    }

    #[test]
    fn test_snap_freq_to_iso() {
        assert_eq!(snap_freq_to_iso(100), 100);
        assert_eq!(snap_freq_to_iso(101), 100);
        assert_eq!(snap_freq_to_iso(99), 100);
        assert_eq!(snap_freq_to_iso(150), 160);
        assert_eq!(snap_freq_to_iso(15), 20);
    }

    #[test]
    fn test_snap_q_to_iso() {
        assert_eq!(snap_q_to_iso(1.0), 1.0);
        assert_eq!(snap_q_to_iso(1.1), 1.0);
        assert_eq!(snap_q_to_iso(0.1), 0.1);
        assert_eq!(snap_q_to_iso(3.0), 2.0);
    }

    #[test]
    fn test_snap_gain_step() {
        assert!((snap_gain_step(1.3) - 1.3).abs() < 0.01);
        assert!((snap_gain_step(-1.3) - (-1.3)).abs() < 0.01);
        assert!((snap_gain_step(0.0) - 0.0).abs() < 0.01);
        assert!((snap_gain_step(10.0) - 10.0).abs() < 0.01);
    }
}
