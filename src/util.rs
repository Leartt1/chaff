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
}
