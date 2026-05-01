use crate::models::{
    Filter, FilterType, PEQData, MAX_BAND_GAIN, MAX_GLOBAL_GAIN, MIN_BAND_GAIN, MIN_GLOBAL_GAIN,
};

pub fn parse_autoeq_text(text: &str) -> Result<PEQData, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut filters: Vec<Filter> = (0..10).map(|i| Filter::enabled(i as u8, false)).collect();
    let mut preamp: i8 = 0;
    let mut parsed_count: usize = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.to_lowercase().starts_with("preamp") {
            if let Some(m) = extract_number(line) {
                let val = m.min(MAX_GLOBAL_GAIN as f64).max(MIN_GLOBAL_GAIN as f64);
                preamp = val.round() as i8;
            } else {
                return Err(format!("Line {}: Failed to parse preamp value", line_num));
            }
            continue;
        }

        if line.to_lowercase().contains("filter") {
            if let Some((idx, enabled, filter_type, freq, gain, q)) = parse_filter_line(line) {
                if idx < 10 {
                    filters[idx].enabled = enabled;
                    filters[idx].filter_type = filter_type;
                    filters[idx].freq = (freq.min(20000.0).max(20.0)) as u16;
                    filters[idx].gain = gain.min(MAX_BAND_GAIN).max(MIN_BAND_GAIN);
                    filters[idx].q = q.min(20.0).max(0.1);
                    parsed_count += 1;
                }
            } else {
                return Err(format!("Line {}: Failed to parse filter. Expected: Filter <N>: <ON/OFF> <Type> Fc <Hz> Gain <dB> Q <Q>", line_num));
            }
        }
    }

    if parsed_count == 0 {
        return Err("No valid filters found in AutoEQ text".into());
    }

    Ok(PEQData {
        filters,
        global_gain: preamp,
    })
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

fn parse_filter_line(line: &str) -> Option<(usize, bool, FilterType, f64, f64, f64)> {
    let regex_match = line.find("Filter")?;
    let rest = &line[regex_match..];

    let idx_str: &str = rest.get(7..)?;
    let idx: usize = idx_str
        .trim_start()
        .get(..idx_str.trim_start().find(|c: char| !c.is_ascii_digit())?)?
        .parse()
        .ok()?;
    let idx = idx.saturating_sub(1);

    let on_off = if rest.to_uppercase().contains("ON") {
        true
    } else if rest.to_uppercase().contains("OFF") {
        false
    } else {
        return None;
    };

    let filter_type = if rest.to_uppercase().contains("LSQ") || rest.to_uppercase().contains("LSC")
    {
        FilterType::LowShelf
    } else if rest.to_uppercase().contains("HSQ") || rest.to_uppercase().contains("HSC") {
        FilterType::HighShelf
    } else {
        FilterType::Peak
    };

    let freq = extract_fc_value(rest).or_else(|| extract_number_after(rest, "Fc"))?;
    let gain = extract_gain_value(rest).or_else(|| extract_number_after(rest, "Gain"))?;
    let q = extract_q_value(rest).or_else(|| extract_number_after(rest, "Q"))?;

    Some((idx, on_off, filter_type, freq, gain, q))
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

pub fn peq_to_autoeq(peq: &PEQData) -> String {
    let mut lines = vec![format!("Preamp: {} dB", peq.global_gain)];

    for (i, f) in peq.filters.iter().enumerate() {
        let on_off = if f.enabled { "ON" } else { "OFF" };
        let type_str = match f.filter_type {
            FilterType::LowShelf => "LSQ",
            FilterType::Peak => "PK",
            FilterType::HighShelf => "HSQ",
        };
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

    #[test]
    fn test_parse_autoeq_with_preamp() {
        let text = "Preamp: -3 dB\nFilter 1: ON PK Fc 100 Hz Gain 5.0 dB Q 1.0";
        let result = parse_autoeq_text(text).unwrap();
        assert_eq!(result.global_gain, -3);
        assert!(result.filters[0].enabled);
    }

    #[test]
    fn test_parse_autoeq_multiple_filters() {
        let text = "Filter 1: ON PK Fc 100 Hz Gain 5.0 dB Q 1.0\nFilter 2: OFF PK Fc 1000 Hz Gain 0 dB Q 2.0";
        let result = parse_autoeq_text(text).unwrap();
        assert!(result.filters[0].enabled);
        assert!(!result.filters[1].enabled);
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
        let result = parse_autoeq_text(text).unwrap();
        assert_eq!(result.filters[0].gain, MAX_BAND_GAIN);
    }

    #[test]
    fn test_parse_real_file_format() {
        let text = "Preamp: -6.3 dB\nFilter 1: ON LSC Fc 36 Hz Gain -2.22 dB Q 0.857\nFilter 2: ON PK Fc 166 Hz Gain -0.79 dB Q 1.669";
        let result = parse_autoeq_text(text).unwrap();
        assert_eq!(result.global_gain, -6);
        assert_eq!(result.filters[0].freq, 36);
        assert!((result.filters[0].gain - (-2.22)).abs() < 0.1);
    }
}
