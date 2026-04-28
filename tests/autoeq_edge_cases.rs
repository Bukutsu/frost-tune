use frost_tune::autoeq::{parse_autoeq_text, peq_to_autoeq};
use frost_tune::models::{Filter, FilterType, PEQData};

#[test]
fn test_empty_string() {
    let result = parse_autoeq_text("");
    assert!(result.is_err());
}

#[test]
fn test_only_comments() {
    let text = "# This is a comment\n# Another comment\n";
    let result = parse_autoeq_text(text);
    assert!(result.is_err());
}

#[test]
fn test_duplicate_filters() {
    let text = "Filter 1: ON PK Fc 100 Hz Gain 1.0 dB Q 1.0\nFilter 1: ON PK Fc 200 Hz Gain 2.0 dB Q 2.0";
    let result = parse_autoeq_text(text).unwrap();
    // It should overwrite the first one
    assert_eq!(result.filters[0].freq, 200);
    assert_eq!(result.filters[0].gain, 2.0);
}

#[test]
fn test_too_many_filters() {
    let mut text = String::new();
    for i in 1..=15 {
        text.push_str(&format!("Filter {}: ON PK Fc 100 Hz Gain 1.0 dB Q 1.0\n", i));
    }
    let result = parse_autoeq_text(&text).unwrap();
    // Only 10 bands supported, filters 11-15 should be ignored
    assert_eq!(result.filters.len(), 10);
    for i in 0..10 {
        assert!(result.filters[i].enabled);
    }
}

#[test]
fn test_mixed_on_off() {
    let text = "Filter 1: ON PK Fc 100 Hz Gain 1.0 dB Q 1.0\nFilter 2: OFF PK Fc 200 Hz Gain 2.0 dB Q 2.0";
    let result = parse_autoeq_text(text).unwrap();
    assert!(result.filters[0].enabled);
    assert!(!result.filters[1].enabled);
}

#[test]
fn test_round_trip() {
    let data = PEQData {
        global_gain: -5,
        filters: vec![
            Filter {
                index: 0,
                enabled: true,
                freq: 50,
                gain: 2.0,
                q: 0.5,
                filter_type: FilterType::LowShelf,
            },
            Filter {
                index: 1,
                enabled: false,
                freq: 100,
                gain: 0.0,
                q: 1.0,
                filter_type: FilterType::Peak,
            },
        ],
    };

    let text = peq_to_autoeq(&data);
    let parsed = parse_autoeq_text(&text).unwrap();

    assert_eq!(parsed.global_gain, data.global_gain);
    // AutoEQ text format always generates 10 filters from our output if we had 10,
    // but here we passed only 2. The parser creates a 10-band array.
    assert_eq!(parsed.filters[0].freq, data.filters[0].freq);
    assert_eq!(parsed.filters[0].gain, data.filters[0].gain);
    assert_eq!(parsed.filters[0].q, data.filters[0].q);
    assert_eq!(parsed.filters[0].filter_type, data.filters[0].filter_type);
    assert_eq!(parsed.filters[0].enabled, data.filters[0].enabled);

    assert_eq!(parsed.filters[1].enabled, data.filters[1].enabled);
}
