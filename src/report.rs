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
