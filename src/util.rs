use std::time::Duration;

/// Parse a human duration like `30d`, `2w`, `6mo`, `1y`, `12h` into a `Duration`.
/// Returns `None` for malformed input.
pub fn parse_age(s: &str) -> Option<Duration> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_alphabetic())?;
    let (num, unit) = s.split_at(split);
    let n: u64 = num.trim().parse().ok()?;
    let secs = match unit {
        "h" => n * 3_600,
        "d" => n * 86_400,
        "w" => n * 7 * 86_400,
        "mo" => n * 30 * 86_400,
        "y" => n * 365 * 86_400,
        _ => return None,
    };
    Some(Duration::from_secs(secs))
}

/// Parse a human size like `100M`, `1.5G`, `500k`, `2GB` into bytes (decimal,
/// 1000-based, to match the sizes chaff displays). Returns `None` if malformed.
pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let split = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let value: f64 = num.trim().parse().ok()?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    let mult: f64 = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" => 1_000.0,
        "m" | "mb" => 1_000_000.0,
        "g" | "gb" => 1_000_000_000.0,
        "t" | "tb" => 1_000_000_000_000.0,
        _ => return None,
    };
    Some((value * mult) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_units() {
        assert_eq!(parse_age("30d"), Some(Duration::from_secs(30 * 86_400)));
        assert_eq!(parse_age("2w"), Some(Duration::from_secs(14 * 86_400)));
        assert_eq!(parse_age("6mo"), Some(Duration::from_secs(180 * 86_400)));
        assert_eq!(parse_age("12h"), Some(Duration::from_secs(12 * 3_600)));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_age("30"), None);
        assert_eq!(parse_age("d"), None);
        assert_eq!(parse_age("abc"), None);
        assert_eq!(parse_age("30x"), None);
    }

    #[test]
    fn parses_sizes() {
        assert_eq!(parse_size("100M"), Some(100_000_000));
        assert_eq!(parse_size("1.5G"), Some(1_500_000_000));
        assert_eq!(parse_size("500k"), Some(500_000));
        assert_eq!(parse_size("2GB"), Some(2_000_000_000));
        assert_eq!(parse_size("1024"), Some(1024));
    }

    #[test]
    fn rejects_bad_sizes() {
        assert_eq!(parse_size("10x"), None);
        assert_eq!(parse_size("abc"), None);
        assert_eq!(parse_size(""), None);
    }
}
