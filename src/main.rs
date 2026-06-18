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

use clap::{Parser, Subcommand};
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan {
            paths,
            caches,
            no_caches,
        } => {
            let roots = roots_or_cwd(paths)?;
            let settings = config::load(&roots);
            let caches_eff = effective_caches(caches, no_caches, settings.caches);
            note_config_caches(caches, no_caches, settings.caches);

            let mut items = scan::scan(&roots);
            if caches_eff {
                items.extend(caches::scan_caches());
            }
            let before = items.len();
            items.retain(|i| !config::is_ignored(&settings.ignore, &i.path));
            let ignored = before - items.len();
            items.sort_by_key(|i| std::cmp::Reverse(i.size));

            report::print_table(&items);
            if ignored > 0 {
                println!("Protected {ignored} path(s) via .chaffignore/config.");
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
        } => {
            let roots = roots_or_cwd(paths)?;
            let settings = config::load(&roots);
            let caches_eff = effective_caches(caches, no_caches, settings.caches);
            note_config_caches(caches, no_caches, settings.caches);

            let no_filters = !all && types.is_empty() && older_than.is_none();

            // CLI --older-than overrides config; parse the effective value once.
            let older_eff =
                match older_than.or_else(|| settings.older_than.clone()) {
                    Some(s) => Some(util::parse_age(&s).ok_or_else(|| {
                        anyhow::anyhow!("bad --older-than '{s}' (try 30d, 2w, 6mo)")
                    })?),
                    None => None,
                };

            // Bare `chaff clean` in a terminal opens the interactive picker.
            if no_filters && !apply && std::io::stdout().is_terminal() {
                clean::run_interactive(&roots, force, caches_eff, &settings.ignore, older_eff)?;
            } else {
                let opts = clean::CleanOptions {
                    older_than: older_eff,
                    types,
                    all,
                    apply,
                    force,
                    include_caches: caches_eff,
                };
                clean::run(&roots, &opts, &settings.ignore)?;
            }
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
