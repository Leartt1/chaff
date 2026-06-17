use crate::model::Reclaimable;
use crate::{gitinfo, report, scan};
use std::path::PathBuf;
use std::time::Duration;

/// What the user asked `clean` to do.
pub struct CleanOptions {
    pub older_than: Option<Duration>,
    pub types: Vec<String>,
    pub all: bool,
    pub apply: bool,
    pub force: bool,
}

/// Why an item was excluded from cleaning.
#[derive(Debug, PartialEq, Eq)]
pub enum Skip {
    TooNew,
    WrongType,
    Tracked,
}

/// Decide whether `item` should be cleaned under `opts`.
pub fn eligible(item: &Reclaimable, opts: &CleanOptions) -> Result<(), Skip> {
    if !opts.types.is_empty()
        && !opts
            .types
            .iter()
            .any(|t| t == item.ecosystem || t == item.label)
    {
        return Err(Skip::WrongType);
    }

    if let Some(max_age) = opts.older_than {
        let old_enough = item
            .modified
            .and_then(|m| m.elapsed().ok())
            .is_some_and(|age| age >= max_age);
        if !old_enough {
            return Err(Skip::TooNew);
        }
    }

    if !opts.force && gitinfo::contains_tracked_files(&item.path) {
        return Err(Skip::Tracked);
    }

    Ok(())
}

/// Scan, filter, preview, and (with `--apply`) reclaim.
pub fn run(roots: &[PathBuf], opts: &CleanOptions) -> anyhow::Result<()> {
    let mut items = scan::scan(roots);
    items.sort_by_key(|i| std::cmp::Reverse(i.size));

    let mut chosen = Vec::new();
    let mut tracked = 0u32;
    for it in items {
        match eligible(&it, opts) {
            Ok(()) => chosen.push(it),
            Err(Skip::Tracked) => tracked += 1,
            Err(_) => {}
        }
    }

    if chosen.is_empty() {
        let note = if tracked > 0 {
            format!(" ({tracked} protected: git-tracked)")
        } else {
            String::new()
        };
        println!("Nothing to clean.{note}");
        return Ok(());
    }

    report::print_table(&chosen);
    if tracked > 0 {
        println!("Protected {tracked} item(s) with git-tracked files (use --force to include).");
    }

    if !opts.apply {
        println!("\nDry run — nothing deleted. Re-run with --apply to send these to the trash.");
        return Ok(());
    }

    let has_selection = opts.all || !opts.types.is_empty() || opts.older_than.is_some();
    if !has_selection {
        anyhow::bail!("Refusing to clean everything without a filter. Pass --all (or --type / --older-than).");
    }

    let mut reclaimed = 0u64;
    let mut count = 0u32;
    for it in &chosen {
        match trash::delete(&it.path) {
            Ok(()) => {
                reclaimed += it.size;
                count += 1;
            }
            Err(e) => eprintln!("  could not remove {}: {e}", it.path.display()),
        }
    }
    println!(
        "\nReclaimed {} from {} item(s) → trash.",
        report::human(reclaimed),
        count
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn item(eco: &'static str, label: &'static str, age_days: u64) -> Reclaimable {
        Reclaimable {
            path: PathBuf::from("/does/not/exist"),
            ecosystem: eco,
            label,
            size: 1,
            modified: Some(SystemTime::now() - Duration::from_secs(age_days * 86_400)),
        }
    }

    fn opts() -> CleanOptions {
        CleanOptions {
            older_than: None,
            types: vec![],
            all: true,
            apply: false,
            force: true, // skip the git check in unit tests
        }
    }

    #[test]
    fn type_filter_excludes_others() {
        let it = item("node", "node_modules", 1);
        let mut o = opts();
        o.types = vec!["rust".to_string()];
        assert_eq!(eligible(&it, &o), Err(Skip::WrongType));
        o.types = vec!["node".to_string()];
        assert_eq!(eligible(&it, &o), Ok(()));
    }

    #[test]
    fn age_filter_excludes_recent() {
        let it = item("node", "node_modules", 40);
        let mut o = opts();
        o.older_than = Some(Duration::from_secs(30 * 86_400));
        assert_eq!(eligible(&it, &o), Ok(()));
        o.older_than = Some(Duration::from_secs(100 * 86_400));
        assert_eq!(eligible(&it, &o), Err(Skip::TooNew));
    }
}
