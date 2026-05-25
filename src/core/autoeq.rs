// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::{Filter, FilterType, PEQData};

pub fn parse_autoeq_text(text: &str) -> Result<(PEQData, Vec<String>), String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut filters: std::collections::BTreeMap<usize, Filter> = std::collections::BTreeMap::new();
    let mut preamp: i8 = 0;
    let mut parsed_count: usize = 0;
    let mut warnings: Vec<String> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.to_lowercase().starts_with("preamp") {
            if let Some(m) = extract_number(line) {
                // Preamp is unbounded here, will be clamped later
                preamp = m.round() as i8;
            } else {
                warnings.push(format!("Line {}: Failed to parse preamp value", line_num));
            }
            continue;
        }

        if line.to_lowercase().contains("filter") {
            if let Some(parsed) = parse_filter_line(line) {
                filters.insert(
                    parsed.index,
                    Filter {
                        index: parsed.index as u8,
                        enabled: parsed.enabled,
                        freq: parsed.freq as u16,
                        gain: parsed.gain,
                        q: parsed.q,
                        filter_type: parsed.filter_type,
                    },
                );
                parsed_count += 1;
            } else {
                warnings.push(format!("Line {}: Failed to parse filter", line_num));
            }
        }
    }

    if parsed_count == 0 && preamp == 0 {
        return Err("No valid filters or preamp found in AutoEQ text".into());
    }

    // Convert BTreeMap to a contiguous Vec<Filter>, padding missing indices with disabled filters
    let max_idx = filters.keys().max().copied().unwrap_or(0);
    let mut contiguous_filters = Vec::with_capacity(max_idx + 1);
    for i in 0..=max_idx {
        if let Some(f) = filters.remove(&i) {
            contiguous_filters.push(f);
        } else {
            contiguous_filters.push(Filter::enabled(i as u8, false));
        }
    }

    Ok((
        PEQData {
            filters: contiguous_filters,
            global_gain: preamp,
        },
        warnings,
    ))
}

fn extract_number(s: &str) -> Option<f64> {
    let start = s.find(|c: char| c == '-' || c == '+' || c.is_ascii_digit())?;

    let mut end = start;
    let mut has_decimal = false;
    for c in s[start..].chars() {
        if c.is_ascii_digit() {
            end += c.len_utf8();
        } else if c == '.' && !has_decimal {
            has_decimal = true;
            end += c.len_utf8();
        } else if (c == '-' || c == '+') && end == start {
            end += c.len_utf8();
        } else {
            break;
        }
    }

    s[start..end].parse().ok()
}

struct ParsedFilterLine {
    index: usize,
    enabled: bool,
    filter_type: FilterType,
    freq: f64,
    gain: f64,
    q: f64,
}

fn parse_filter_line(line: &str) -> Option<ParsedFilterLine> {
    let regex_match = line.find("Filter")?;
    let rest = &line[regex_match..];
    let rest_upper = rest.to_uppercase();

    let idx_str: &str = rest.get(7..)?;
    let idx: usize = idx_str
        .trim_start()
        .get(..idx_str.trim_start().find(|c: char| !c.is_ascii_digit())?)?
        .parse()
        .ok()?;
    let idx = idx.saturating_sub(1);

    let on_off = if rest_upper.contains("ON") {
        true
    } else if rest_upper.contains("OFF") {
        false
    } else {
        return None;
    };

    let filter_type = if contains_token(&rest_upper, "LSC") || contains_token(&rest_upper, "LSQ") {
        FilterType::LowShelf
    } else if contains_token(&rest_upper, "HSC") || contains_token(&rest_upper, "HSQ") {
        FilterType::HighShelf
    } else if contains_token(&rest_upper, "HP") {
        FilterType::HighPass
    } else if contains_token(&rest_upper, "LP") {
        FilterType::LowPass
    } else if contains_token(&rest_upper, "LS") {
        FilterType::LowShelf
    } else if contains_token(&rest_upper, "HS") {
        FilterType::HighShelf
    } else {
        FilterType::Peak
    };

    let freq = extract_fc_value(rest).or_else(|| extract_number_after(rest, "Fc"))?;
    let gain = extract_gain_value(rest).or_else(|| extract_number_after(rest, "Gain"))?;
    let q = extract_q_value(rest).or_else(|| extract_number_after(rest, "Q"))?;

    Some(ParsedFilterLine {
        index: idx,
        enabled: on_off,
        filter_type,
        freq,
        gain,
        q,
    })
}

fn extract_fc_value(s: &str) -> Option<f64> {
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find("fc") {
        extract_number(&s[pos..])
    } else {
        None
    }
}

fn extract_gain_value(s: &str) -> Option<f64> {
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find("gain") {
        extract_number(&s[pos..])
    } else {
        None
    }
}

fn extract_q_value(s: &str) -> Option<f64> {
    let lower = s.to_lowercase();
    // Look for " q " or " q:" to avoid matching "lsq" or "hsq"
    if let Some(pos) = lower
        .find(" q ")
        .or_else(|| lower.find(" q:"))
        .or_else(|| lower.find(" q="))
    {
        extract_number(&s[pos..])
    } else if let Some(pos) = lower.rfind('q') {
        // Fallback to last 'q' in the line, which is usually the Q parameter
        extract_number(&s[pos..])
    } else {
        None
    }
}

fn extract_number_after(s: &str, keyword: &str) -> Option<f64> {
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find(keyword) {
        extract_number(&s[pos + keyword.len()..])
    } else {
        None
    }
}

fn contains_token(haystack: &str, token: &str) -> bool {
    haystack
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|w| w == token)
}

pub fn peq_to_autoeq(peq: &PEQData) -> String {
    let mut lines = vec![format!("Preamp: {} dB", peq.global_gain)];

    for (i, f) in peq.filters.iter().enumerate() {
        let on_off = if f.enabled { "ON" } else { "OFF" };
        let type_str = f.filter_type.autoeq_token();
        lines.push(format!(
            "Filter {}: {} {} Fc {} Hz Gain {:.2} dB Q {:.3}",
            i + 1,
            on_off,
            type_str,
            f.freq,
            f.gain,
            f.q
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::MAX_BAND_GAIN;

    #[test]
    fn test_parse_autoeq_with_preamp() {
        let text = "Preamp: -3 dB\nFilter 1: ON PK Fc 100 Hz Gain 5.0 dB Q 1.0";
        let (result, warnings) = parse_autoeq_text(text).unwrap();
        assert_eq!(result.global_gain, -3);
        assert!(result.filters[0].enabled);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_autoeq_multiple_filters() {
        let text = "Filter 1: ON PK Fc 100 Hz Gain 5.0 dB Q 1.0\nFilter 2: OFF PK Fc 1000 Hz Gain 0 dB Q 2.0";
        let (result, warnings) = parse_autoeq_text(text).unwrap();
        assert!(result.filters[0].enabled);
        assert!(!result.filters[1].enabled);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_peq_to_autoeq_format() {
        let peq = PEQData {
            filters: vec![Filter::enabled(0, true)],
            global_gain: -3,
        };
        let output = peq_to_autoeq(&peq);
        assert!(output.contains("Preamp: -3 dB"));
        assert!(output.contains("Filter 1: ON"));
    }

    #[test]
    fn test_parse_autoeq_clamp_gain() {
        let text = "Filter 1: ON PK Fc 100 Hz Gain 20.0 dB Q 1.0";
        let (mut result, warnings) = parse_autoeq_text(text).unwrap();

        result.clamp_to_capabilities(&crate::core::device::capabilities::DESKTOP_DAC_CAPS);

        assert_eq!(result.filters[0].gain, MAX_BAND_GAIN);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_real_file_format() {
        let text = "Preamp: -6.3 dB\nFilter 1: ON LSC Fc 36 Hz Gain -2.22 dB Q 0.857\nFilter 2: ON PK Fc 166 Hz Gain -0.79 dB Q 1.669";
        let (result, warnings) = parse_autoeq_text(text).unwrap();
        assert_eq!(result.global_gain, -6);
        assert_eq!(result.filters[0].freq, 36);
        assert!((result.filters[0].gain - (-2.22)).abs() < 0.1);
        assert_eq!(result.filters[0].filter_type, FilterType::LowShelf);
        assert_eq!(result.filters[1].filter_type, FilterType::Peak);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_parse_user_clipboard_shelf_types() {
        let text = "Preamp: -6.5 dB
Filter 1: ON PK Fc 22 Hz Gain -0.86 dB Q 1.717
Filter 2: ON LSC Fc 43 Hz Gain -1.38 dB Q 1.004
Filter 8: ON HSC Fc 7624 Hz Gain 0.59 dB Q 3.000";
        let (result, _) = parse_autoeq_text(text).unwrap();
        assert_eq!(result.filters[0].filter_type, FilterType::Peak);
        assert_eq!(result.filters[1].filter_type, FilterType::LowShelf);
        assert_eq!(result.filters[7].filter_type, FilterType::HighShelf);
    }

    #[test]
    fn test_round_trip_shelf_preserves_type() {
        let original = PEQData {
            filters: vec![
                Filter {
                    index: 0,
                    enabled: true,
                    freq: 80,
                    gain: -2.0,
                    q: 0.7,
                    filter_type: FilterType::LowShelf,
                },
                Filter {
                    index: 1,
                    enabled: true,
                    freq: 8000,
                    gain: 1.0,
                    q: 0.7,
                    filter_type: FilterType::HighShelf,
                },
            ],
            global_gain: 0,
        };
        let text = peq_to_autoeq(&original);
        let (parsed, _) = parse_autoeq_text(&text).unwrap();
        assert_eq!(parsed.filters[0].filter_type, FilterType::LowShelf);
        assert_eq!(parsed.filters[1].filter_type, FilterType::HighShelf);
    }

    #[test]
    fn test_parse_legacy_ls_hs_tokens() {
        let text =
            "Filter 1: ON LS Fc 80 Hz Gain -2 dB Q 0.7\nFilter 2: ON HS Fc 8000 Hz Gain 1 dB Q 0.7";
        let (result, _) = parse_autoeq_text(text).unwrap();
        assert_eq!(result.filters[0].filter_type, FilterType::LowShelf);
        assert_eq!(result.filters[1].filter_type, FilterType::HighShelf);
    }

    #[test]
    fn test_parse_autoeq_lenient_with_bad_lines() {
        let text = "Preamp: -3 dB\nFilter 1: ON PK Fc 100 Hz Gain 5.0 dB Q 1.0\nFilter 2: BAD FORMAT\nFilter 3: OFF PK Fc 1000 Hz Gain 0 dB Q 2.0";
        let (result, warnings) = parse_autoeq_text(text).unwrap();
        assert_eq!(result.filters[0].freq, 100);
        assert_eq!(result.filters[2].freq, 1000);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Failed to parse"));
    }
}
