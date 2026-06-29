use crate::model::Reclaimable;
use humansize::{format_size, DECIMAL};
use std::collections::BTreeMap;
use std::time::SystemTime;

/// Format a byte count as a human-readable size (e.g. "1.2 GB").
pub fn human(bytes: u64) -> String {
    format_size(bytes, DECIMAL)
}

/// Print a size-sorted table of reclaimable artifacts, a total, and a
/// per-ecosystem breakdown. `limit` caps the rows shown; the total still
/// reflects every item.
pub fn print_table(items: &[Reclaimable], limit: Option<usize>) {
    if items.is_empty() {
        println!("Nothing reclaimable found. Your projects are already tidy. 🌾");
        return;
    }

    let shown = visible_count(items.len(), limit);
    println!("{:>11}  {:>6}  {:<13} PATH", "SIZE", "AGE", "TYPE");
    for i in &items[..shown] {
        println!(
            "{:>11}  {:>6}  {:<13} {}",
            format_size(i.size, DECIMAL),
            age(i.modified),
            i.label,
            i.path.display()
        );
    }

    let total: u64 = items.iter().map(|i| i.size).sum();
    let mut by_eco: BTreeMap<&str, u64> = BTreeMap::new();
    for i in items {
        *by_eco.entry(i.ecosystem).or_default() += i.size;
    }
    let breakdown = by_eco
        .iter()
        .map(|(eco, sz)| format!("{} {}", format_size(*sz, DECIMAL), eco))
        .collect::<Vec<_>>()
        .join(", ");

    let more = if shown < items.len() {
        format!("  (showing top {shown})")
    } else {
        String::new()
    };
    println!(
        "\nReclaimable: {} across {} item(s)  ({}){}",
        format_size(total, DECIMAL),
        items.len(),
        breakdown,
        more
    );
}

/// How many rows to display given an optional cap.
fn visible_count(total: usize, limit: Option<usize>) -> usize {
    limit.map_or(total, |n| n.min(total))
}

/// Print a per-ecosystem horizontal bar chart, largest first.
pub fn print_chart(items: &[Reclaimable]) {
    if items.is_empty() {
        return;
    }
    let mut by_eco: BTreeMap<&str, u64> = BTreeMap::new();
    for i in items {
        *by_eco.entry(i.ecosystem).or_default() += i.size;
    }
    let max = by_eco.values().copied().max().unwrap_or(0);
    let mut rows: Vec<(&str, u64)> = by_eco.into_iter().collect();
    rows.sort_by_key(|(_, sz)| std::cmp::Reverse(*sz));

    const WIDTH: usize = 28;
    println!();
    for (eco, sz) in rows {
        let bar = "█".repeat(bar_len(sz, max, WIDTH));
        println!("{eco:<12} {bar:<WIDTH$} {}", human(sz));
    }
}

/// Length of a bar (in cells) for `size` relative to `max`, capped at `width`.
fn bar_len(size: u64, max: u64, width: usize) -> usize {
    if max == 0 {
        return 0;
    }
    ((size as f64 / max as f64) * width as f64).round() as usize
}

/// Compact human age (e.g. "3d", "2w", "5mo") from a last-modified time.
fn age(modified: Option<SystemTime>) -> String {
    let Some(t) = modified else {
        return "-".to_string();
    };
    let Ok(elapsed) = t.elapsed() else {
        return "-".to_string();
    };
    let secs = elapsed.as_secs();
    let days = secs / 86_400;
    if days >= 365 {
        format!("{}y", days / 365)
    } else if days >= 30 {
        format!("{}mo", days / 30)
    } else if days >= 7 {
        format!("{}w", days / 7)
    } else if days >= 1 {
        format!("{}d", days)
    } else if secs >= 3600 {
        format!("{}h", secs / 3600)
    } else {
        "new".to_string()
    }
}

#[derive(serde::Serialize)]
struct JsonItem<'a> {
    path: String,
    kind: &'a str,
    ecosystem: &'a str,
    size_bytes: u64,
    age_seconds: Option<u64>,
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    total_bytes: u64,
    count: usize,
    items: Vec<JsonItem<'a>>,
}

/// Serialize reclaimables to pretty JSON for scripting (`scan --json`).
pub fn to_json(items: &[Reclaimable]) -> String {
    let report = JsonReport {
        total_bytes: items.iter().map(|i| i.size).sum(),
        count: items.len(),
        items: items
            .iter()
            .map(|i| JsonItem {
                path: i.path.display().to_string(),
                kind: i.label,
                ecosystem: i.ecosystem,
                size_bytes: i.size,
                age_seconds: i
                    .modified
                    .and_then(|m| m.elapsed().ok())
                    .map(|d| d.as_secs()),
            })
            .collect(),
    };
    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

/// Print reclaimables as JSON to stdout.
pub fn print_json(items: &[Reclaimable]) {
    println!("{}", to_json(items));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn json_has_items_and_total() {
        let items = vec![Reclaimable {
            path: PathBuf::from("/a/node_modules"),
            ecosystem: "node",
            label: "node_modules",
            size: 100,
            modified: None,
        }];
        let v: serde_json::Value = serde_json::from_str(&to_json(&items)).unwrap();
        assert_eq!(v["count"], 1);
        assert_eq!(v["total_bytes"], 100);
        assert_eq!(v["items"][0]["kind"], "node_modules");
        assert_eq!(v["items"][0]["size_bytes"], 100);
        assert_eq!(v["items"][0]["ecosystem"], "node");
    }

    #[test]
    fn visible_count_caps() {
        assert_eq!(visible_count(10, None), 10);
        assert_eq!(visible_count(10, Some(3)), 3);
        assert_eq!(visible_count(2, Some(5)), 2);
    }

    #[test]
    fn bar_len_scales() {
        assert_eq!(bar_len(100, 100, 30), 30);
        assert_eq!(bar_len(50, 100, 30), 15);
        assert_eq!(bar_len(0, 100, 30), 0);
        assert_eq!(bar_len(100, 0, 30), 0);
    }
}
