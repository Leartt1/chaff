mod caches;
mod clean;
mod config;
mod gitinfo;
mod model;
mod report;
mod rules;
mod scan;
mod size;
mod tui;
mod util;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::IsTerminal;
use std::path::PathBuf;

/// Safe, smart dev-disk reclaimer — winnow the chaff from your projects.
#[derive(Parser, Debug)]
#[command(name = "chaff", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show reclaimable space, biggest first.
    Scan {
        /// Roots to scan (defaults to the current directory).
        paths: Vec<PathBuf>,
        /// Also include global package-manager caches.
        #[arg(long)]
        caches: bool,
        /// Exclude global caches even if enabled in config.
        #[arg(long)]
        no_caches: bool,
        /// Output machine-readable JSON instead of a table.
        #[arg(long)]
        json: bool,
        /// Only show items at least this big (e.g. 100M, 1.5G).
        #[arg(long)]
        min_size: Option<String>,
        /// Show only the N largest items (total still reflects everything).
        #[arg(long)]
        top: Option<usize>,
        /// Sort order: size, age, or name.
        #[arg(long, value_enum, default_value = "size")]
        sort: SortKey,
    },
    /// Reclaim space — interactive picker by default; flags for scripting.
    Clean {
        /// Roots to scan (defaults to the current directory).
        paths: Vec<PathBuf>,
        /// Reclaim everything that passes the safety checks.
        #[arg(long)]
        all: bool,
        /// Limit to these ecosystems/types, comma-separated (e.g. node,rust,cache).
        #[arg(long = "type", value_delimiter = ',')]
        types: Vec<String>,
        /// Only items untouched for at least this long (e.g. 30d, 2w, 6mo).
        #[arg(long)]
        older_than: Option<String>,
        /// Actually delete (to trash). Without this, clean only previews.
        #[arg(long)]
        apply: bool,
        /// Include items containing git-tracked files (off by default).
        #[arg(long)]
        force: bool,
        /// Also include global package-manager caches.
        #[arg(long)]
        caches: bool,
        /// Exclude global caches even if enabled in config.
        #[arg(long)]
        no_caches: bool,
        /// Only consider items at least this big (e.g. 100M, 1.5G).
        #[arg(long)]
        min_size: Option<String>,
        /// Output the would-be-reclaimed set as JSON (never deletes).
        #[arg(long)]
        json: bool,
        /// Permanently delete instead of trash (frees space now; not recoverable).
        #[arg(long)]
        purge: bool,
    },
    /// Print a shell completion script (bash, zsh, fish, …).
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
}

/// `--no-caches` always wins; otherwise the CLI flag or config enables caches.
fn effective_caches(cli_caches: bool, no_caches: bool, config_caches: bool) -> bool {
    !no_caches && (cli_caches || config_caches)
}

/// Make config-enabled cache scope visible (it widens what gets reclaimed).
fn note_config_caches(cli_caches: bool, no_caches: bool, config_caches: bool) {
    if config_caches && !cli_caches && !no_caches {
        eprintln!("chaff: including global caches (enabled in config; --no-caches to skip)");
    }
}

/// Parse an optional `--min-size` value into bytes (absent = no minimum).
fn parse_min_size(s: Option<&str>) -> anyhow::Result<u64> {
    match s {
        Some(s) => util::parse_size(s)
            .ok_or_else(|| anyhow::anyhow!("bad --min-size '{s}' (try 100M, 1.5G)")),
        None => Ok(0),
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum SortKey {
    Size,
    Age,
    Name,
}

/// Sort reclaimables in place by the chosen key.
fn sort_items(items: &mut [model::Reclaimable], key: SortKey) {
    match key {
        SortKey::Size => items.sort_by_key(|i| std::cmp::Reverse(i.size)),
        SortKey::Name => items.sort_by(|a, b| a.path.cmp(&b.path)),
        SortKey::Age => items.sort_by(|a, b| match (a.modified, b.modified) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan {
            paths,
            caches,
            no_caches,
            json,
            min_size,
            top,
            sort,
        } => {
            let roots = roots_or_cwd(paths)?;
            let settings = config::load(&roots);
            let caches_eff = effective_caches(caches, no_caches, settings.caches);
            let min_size = parse_min_size(min_size.as_deref())?;
            if !json {
                note_config_caches(caches, no_caches, settings.caches);
            }

            let mut items = scan::scan(&roots);
            if caches_eff {
                items.extend(caches::scan_caches());
            }
            let before = items.len();
            items.retain(|i| !config::is_ignored(&settings.ignore, &i.path));
            let ignored = before - items.len();
            items.retain(|i| i.size >= min_size);
            sort_items(&mut items, sort);

            if json {
                report::print_json(&items);
            } else {
                report::print_table(&items, top);
                if ignored > 0 {
                    println!("Protected {ignored} path(s) via .chaffignore/config.");
                }
            }
        }
        Command::Clean {
            paths,
            all,
            types,
            older_than,
            apply,
            force,
            caches,
            no_caches,
            min_size,
            json,
            purge,
        } => {
            let roots = roots_or_cwd(paths)?;
            let settings = config::load(&roots);
            let caches_eff = effective_caches(caches, no_caches, settings.caches);
            if !json {
                note_config_caches(caches, no_caches, settings.caches);
            }
            let min_size = parse_min_size(min_size.as_deref())?;

            let no_filters = !all && types.is_empty() && older_than.is_none();

            // CLI --older-than overrides config; parse the effective value once.
            let older_eff =
                match older_than.or_else(|| settings.older_than.clone()) {
                    Some(s) => Some(util::parse_age(&s).ok_or_else(|| {
                        anyhow::anyhow!("bad --older-than '{s}' (try 30d, 2w, 6mo)")
                    })?),
                    None => None,
                };

            let opts = clean::CleanOptions {
                older_than: older_eff,
                types,
                all,
                apply,
                force,
                include_caches: caches_eff,
                min_size,
                purge,
            };

            if json {
                // Machine-readable preview; never deletes.
                clean::run_json(&roots, &opts, &settings.ignore)?;
            } else if no_filters && !apply && std::io::stdout().is_terminal() {
                // Bare `chaff clean` in a terminal opens the interactive picker.
                clean::run_interactive(
                    &roots,
                    force,
                    caches_eff,
                    &settings.ignore,
                    older_eff,
                    min_size,
                )?;
            } else {
                clean::run(&roots, &opts, &settings.ignore)?;
            }
        }
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
    }
    Ok(())
}

fn roots_or_cwd(paths: Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    if paths.is_empty() {
        Ok(vec![std::env::current_dir()?])
    } else {
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn min_size_parsing() {
        assert_eq!(parse_min_size(None).unwrap(), 0);
        assert_eq!(parse_min_size(Some("100M")).unwrap(), 100_000_000);
        assert!(parse_min_size(Some("bogus")).is_err());
    }

    #[test]
    fn sort_items_orders_by_key() {
        use std::time::{Duration, SystemTime};
        let mk = |size: u64, age_days: u64, path: &str| model::Reclaimable {
            path: PathBuf::from(path),
            ecosystem: "x",
            label: "y",
            size,
            modified: Some(SystemTime::now() - Duration::from_secs(age_days * 86_400)),
        };
        let mut v = vec![mk(100, 1, "/b"), mk(300, 365, "/a"), mk(50, 30, "/c")];
        sort_items(&mut v, SortKey::Size);
        assert_eq!(v[0].size, 300);
        sort_items(&mut v, SortKey::Name);
        assert_eq!(v[0].path, PathBuf::from("/a"));
        sort_items(&mut v, SortKey::Age);
        assert_eq!(v[0].size, 300);
    }
}
