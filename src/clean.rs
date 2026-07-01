use crate::model::Reclaimable;
use crate::{caches, config, gitinfo, report, scan, tui};
use globset::GlobSet;
use std::path::PathBuf;
use std::time::Duration;

/// What the user asked `clean` to do.
pub struct CleanOptions {
    pub older_than: Option<Duration>,
    pub types: Vec<String>,
    pub exclude_types: Vec<String>,
    pub all: bool,
    pub apply: bool,
    pub force: bool,
    pub include_caches: bool,
    pub min_size: u64,
    /// Permanently delete instead of moving to the trash (frees space immediately;
    /// not recoverable). Trash is the default.
    pub purge: bool,
}

/// Why an item was excluded from cleaning.
#[derive(Debug, PartialEq, Eq)]
pub enum Skip {
    TooNew,
    TooSmall,
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

    if opts
        .exclude_types
        .iter()
        .any(|t| t == item.ecosystem || t == item.label)
    {
        return Err(Skip::WrongType);
    }

    if item.size < opts.min_size {
        return Err(Skip::TooSmall);
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

/// Scan the roots (+ caches if requested), drop `.chaffignore` matches, and sort
/// largest-first. Returns the items and how many were protected by ignore rules.
fn collect(roots: &[PathBuf], include_caches: bool, ignore: &GlobSet) -> (Vec<Reclaimable>, usize) {
    let mut items = scan::scan(roots);
    if include_caches {
        items.extend(caches::scan_caches());
    }
    let before = items.len();
    items.retain(|i| !config::is_ignored(ignore, &i.path));
    let ignored = before - items.len();
    items.sort_by_key(|i| std::cmp::Reverse(i.size));
    (items, ignored)
}

fn filter(items: Vec<Reclaimable>, opts: &CleanOptions) -> (Vec<Reclaimable>, u32) {
    let mut chosen = Vec::new();
    let mut tracked = 0u32;
    for it in items {
        match eligible(&it, opts) {
            Ok(()) => chosen.push(it),
            Err(Skip::Tracked) => tracked += 1,
            Err(_) => {}
        }
    }
    (chosen, tracked)
}

/// Remove the chosen items, either to the trash (default) or permanently (`purge`).
fn delete_all(items: &[Reclaimable], purge: bool) -> (u64, u32) {
    let mut reclaimed = 0u64;
    let mut count = 0u32;
    for it in items {
        let result = if purge {
            std::fs::remove_dir_all(&it.path).map_err(anyhow::Error::from)
        } else {
            trash::delete(&it.path).map_err(anyhow::Error::from)
        };
        match result {
            Ok(()) => {
                reclaimed += it.size;
                count += 1;
            }
            Err(e) => eprintln!("  could not remove {}: {e}", it.path.display()),
        }
    }
    (reclaimed, count)
}

fn report_reclaimed(reclaimed: u64, count: u32, purge: bool) {
    let how = if purge { "deleted" } else { "→ trash" };
    println!(
        "\nReclaimed {} from {} item(s) ({}).",
        report::human(reclaimed),
        count,
        how
    );
}

fn print_protected(tracked: u32, ignored: usize) {
    if tracked > 0 {
        println!("Protected {tracked} item(s) with git-tracked files (use --force to include).");
    }
    if ignored > 0 {
        println!("Protected {ignored} path(s) via .chaffignore/config.");
    }
}

/// Non-interactive scan, filter, preview, and (with `--apply`) reclaim.
pub fn run(roots: &[PathBuf], opts: &CleanOptions, ignore: &GlobSet) -> anyhow::Result<()> {
    let (items, ignored) = collect(roots, opts.include_caches, ignore);
    let (chosen, tracked) = filter(items, opts);

    if chosen.is_empty() {
        println!("Nothing to clean.");
        print_protected(tracked, ignored);
        return Ok(());
    }

    report::print_table(&chosen, None);
    print_protected(tracked, ignored);

    if !opts.apply {
        let how = if opts.purge {
            "delete them permanently"
        } else {
            "send these to the trash"
        };
        println!("\nDry run — nothing deleted. Re-run with --apply to {how}.");
        return Ok(());
    }

    let has_selection = opts.all || !opts.types.is_empty() || opts.older_than.is_some();
    if !has_selection {
        anyhow::bail!(
            "Refusing to clean everything without a filter. Pass --all (or --type / --older-than)."
        );
    }

    let (reclaimed, count) = delete_all(&chosen, opts.purge);
    report_reclaimed(reclaimed, count, opts.purge);
    Ok(())
}

/// Interactive picker: scan, apply safety, let the user choose, then reclaim.
pub fn run_interactive(
    roots: &[PathBuf],
    force: bool,
    include_caches: bool,
    ignore: &GlobSet,
    older_than: Option<Duration>,
    min_size: u64,
) -> anyhow::Result<()> {
    let (items, ignored) = collect(roots, include_caches, ignore);
    let opts = CleanOptions {
        older_than,
        types: vec![],
        exclude_types: vec![],
        all: true,
        apply: true,
        force,
        include_caches,
        min_size,
        purge: false,
    };
    let (chosen, tracked) = filter(items, &opts);

    if chosen.is_empty() {
        println!("Nothing to clean.");
        print_protected(tracked, ignored);
        return Ok(());
    }

    let picks = tui::select(&chosen)?;
    if picks.is_empty() {
        println!("Cancelled — nothing removed.");
        return Ok(());
    }

    let selected: Vec<Reclaimable> = picks.into_iter().map(|i| chosen[i].clone()).collect();
    let (reclaimed, count) = delete_all(&selected, opts.purge);
    report_reclaimed(reclaimed, count, opts.purge);
    Ok(())
}

/// Like `run`, but emits the eligible set as JSON and never deletes.
pub fn run_json(roots: &[PathBuf], opts: &CleanOptions, ignore: &GlobSet) -> anyhow::Result<()> {
    let (items, _ignored) = collect(roots, opts.include_caches, ignore);
    let (chosen, _tracked) = filter(items, opts);
    report::print_json(&chosen);
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
            exclude_types: vec![],
            all: true,
            apply: false,
            force: true, // skip the git check in unit tests
            include_caches: false,
            min_size: 0,
            purge: false,
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
    fn exclude_type_filter_drops_matching() {
        let mut o = opts();
        o.exclude_types = vec!["node".to_string()];
        assert_eq!(
            eligible(&item("node", "node_modules", 1), &o),
            Err(Skip::WrongType)
        );
        assert_eq!(eligible(&item("rust", "target", 1), &o), Ok(()));
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

    #[test]
    fn cache_type_filter_matches_tool() {
        let it = item("cache", "cargo", 1);
        let mut o = opts();
        o.types = vec!["cache".to_string()];
        assert_eq!(eligible(&it, &o), Ok(()));
        o.types = vec!["cargo".to_string()];
        assert_eq!(eligible(&it, &o), Ok(()));
        o.types = vec!["node".to_string()];
        assert_eq!(eligible(&it, &o), Err(Skip::WrongType));
    }

    #[test]
    fn min_size_excludes_small() {
        let mut it = item("node", "node_modules", 1);
        it.size = 50;
        let mut o = opts();
        o.min_size = 100;
        assert_eq!(eligible(&it, &o), Err(Skip::TooSmall));
        o.min_size = 10;
        assert_eq!(eligible(&it, &o), Ok(()));
    }
}
