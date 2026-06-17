mod caches;
mod clean;
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
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scan { paths, caches } => {
            let roots = roots_or_cwd(paths)?;
            let mut items = scan::scan(&roots);
            if caches {
                items.extend(crate::caches::scan_caches());
            }
            items.sort_by_key(|i| std::cmp::Reverse(i.size));
            report::print_table(&items);
        }
        Command::Clean {
            paths,
            all,
            types,
            older_than,
            apply,
            force,
            caches,
        } => {
            let roots = roots_or_cwd(paths)?;
            let no_filters = !all && types.is_empty() && older_than.is_none();

            // Bare `chaff clean` in a terminal opens the interactive picker.
            if no_filters && !apply && std::io::stdout().is_terminal() {
                clean::run_interactive(&roots, force, caches)?;
            } else {
                let older_than = match older_than {
                    Some(s) => Some(util::parse_age(&s).ok_or_else(|| {
                        anyhow::anyhow!("bad --older-than '{s}' (try 30d, 2w, 6mo)")
                    })?),
                    None => None,
                };
                let opts = clean::CleanOptions {
                    older_than,
                    types,
                    all,
                    apply,
                    force,
                    include_caches: caches,
                };
                clean::run(&roots, &opts)?;
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
