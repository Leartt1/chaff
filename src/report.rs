use crate::model::Reclaimable;
use humansize::{format_size, DECIMAL};
use std::collections::BTreeMap;
use std::time::SystemTime;

/// Format a byte count as a human-readable size (e.g. "1.2 GB").
pub fn human(bytes: u64) -> String {
    format_size(bytes, DECIMAL)
}

/// Print a size-sorted table of reclaimable artifacts, a total, and a
/// per-ecosystem breakdown.
pub fn print_table(items: &[Reclaimable]) {
    if items.is_empty() {
        println!("Nothing reclaimable found. Your projects are already tidy. 🌾");
        return;
    }

    println!("{:>11}  {:>6}  {:<13} PATH", "SIZE", "AGE", "TYPE");
    for i in items {
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

    println!(
        "\nReclaimable: {} across {} item(s)  ({})",
        format_size(total, DECIMAL),
        items.len(),
        breakdown
    );
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
}
