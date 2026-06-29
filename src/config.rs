use crate::rules;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// User configuration, loaded from `config.toml`. All fields optional.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default age filter for `clean` (e.g. "30d"). CLI `--older-than` overrides.
    pub older_than: Option<String>,
    /// Include global caches by default. `--caches`/`--no-caches` override.
    pub caches: bool,
    /// Glob patterns to always protect (same syntax as `.chaffignore`).
    pub ignore: Vec<String>,
    /// Custom artifact rules (`[[rule]]` tables), added to the built-ins.
    #[serde(rename = "rule")]
    pub rules: Vec<RawRule>,
}

/// A user-defined artifact rule from config: a directory name, its label, and
/// optional sibling markers (exact names and/or extension suffixes) that gate an
/// ambiguous name.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct RawRule {
    pub dir: String,
    pub ecosystem: String,
    pub requires_marker: Vec<String>,
    pub requires_marker_ext: Vec<String>,
}

/// Convert parsed `[[rule]]` entries into [`rules::Rule`]s, skipping (with a
/// warning) any with an empty `dir`.
fn user_rules(raw: &[RawRule]) -> Vec<rules::Rule> {
    raw.iter()
        .filter_map(|r| {
            let dir = r.dir.trim();
            if dir.is_empty() {
                eprintln!("chaff: ignoring custom rule with empty 'dir'");
                return None;
            }
            let eco = r.ecosystem.trim();
            let eco = if eco.is_empty() { "custom" } else { eco };
            Some(rules::user_rule(
                dir.to_string(),
                eco.to_string(),
                r.requires_marker.clone(),
                r.requires_marker_ext.clone(),
            ))
        })
        .collect()
}

/// Resolved settings: config defaults plus all ignore patterns compiled.
pub struct Settings {
    pub older_than: Option<String>,
    pub caches: bool,
    pub ignore: GlobSet,
}

fn config_home() -> Option<PathBuf> {
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            return Some(PathBuf::from(x));
        }
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
}

fn parse_config(text: &str, path: &Path) -> Config {
    match toml::from_str::<Config>(text) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("chaff: ignoring invalid config {}: {e}", path.display());
            Config::default()
        }
    }
}

fn load_config_file() -> Config {
    // An explicit $CHAFF_CONFIG that can't be read is a user mistake — warn.
    if let Some(explicit) = std::env::var_os("CHAFF_CONFIG") {
        let p = PathBuf::from(explicit);
        return match std::fs::read_to_string(&p) {
            Ok(text) => parse_config(&text, &p),
            Err(e) => {
                eprintln!("chaff: cannot read CHAFF_CONFIG {}: {e}", p.display());
                Config::default()
            }
        };
    }
    // A missing default config file is normal — stay silent.
    let Some(p) = config_home().map(|c| c.join("chaff/config.toml")) else {
        return Config::default();
    };
    match std::fs::read_to_string(&p) {
        Ok(text) => parse_config(&text, &p),
        Err(_) => Config::default(),
    }
}

fn read_ignore_file(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .map(|t| {
            t.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Expand one user pattern into globs that match an artifact's **absolute** path
/// at any depth, covering BOTH the directory itself and its contents.
///
/// Artifact paths are absolute and the path *is* the directory we might delete,
/// so every pattern must be depth-anchored (`**/`) and cover the dir node, not
/// just its children. Handles bare names (`keepme`), wildcards (`cache*`),
/// trailing slashes (`node_modules/`, the gitignore habit), sub-paths
/// (`app/node_modules`), and explicit `**/x/**` forms.
fn expand(pattern: &str) -> Vec<String> {
    let p = pattern.trim().trim_end_matches('/');
    // drop a trailing "/**" so we also protect the directory node itself
    let base = p.strip_suffix("/**").unwrap_or(p);
    if base.is_empty() {
        return Vec::new();
    }
    if base.starts_with('/') || base.starts_with("**/") {
        // already absolute or depth-anchored
        vec![base.to_string(), format!("{base}/**")]
    } else {
        // anchor relative patterns so they match anywhere in an absolute path
        vec![format!("**/{base}"), format!("**/{base}/**")]
    }
}

pub fn build_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        let mut any_ok = false;
        let mut last_err = None;
        for g in expand(p) {
            match Glob::new(&g) {
                Ok(glob) => {
                    builder.add(glob);
                    any_ok = true;
                }
                Err(e) => last_err = Some(e),
            }
        }
        // A pattern that compiled to nothing is a silent protection failure — warn.
        if !any_ok {
            if let Some(e) = last_err {
                eprintln!("chaff: ignoring invalid ignore pattern '{p}': {e}");
            }
        }
    }
    builder.build().unwrap_or_else(|_| GlobSet::empty())
}

/// Load config + every `.chaffignore` (global and per-root) into one `Settings`.
pub fn load(roots: &[PathBuf]) -> Settings {
    let cfg = load_config_file();
    rules::set_user_rules(user_rules(&cfg.rules));
    let mut patterns: Vec<String> = cfg.ignore.clone();

    if let Some(home) = config_home() {
        patterns.extend(read_ignore_file(&home.join("chaff/.chaffignore")));
    }
    for r in roots {
        patterns.extend(read_ignore_file(&r.join(".chaffignore")));
    }

    Settings {
        older_than: cfg.older_than,
        caches: cfg.caches,
        ignore: build_globset(&patterns),
    }
}

pub fn is_ignored(set: &GlobSet, path: &Path) -> bool {
    set.is_match(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gs(patterns: &[&str]) -> GlobSet {
        build_globset(&patterns.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn bare_name_protects_dir_and_contents() {
        let set = gs(&["keepme"]);
        for p in ["/a/keepme", "/a/keepme/node_modules", "/a/b/keepme"] {
            assert!(is_ignored(&set, Path::new(p)), "should protect {p}");
        }
        assert!(!is_ignored(&set, Path::new("/a/src/node_modules")));
    }

    #[test]
    fn wildcard_name_matches_at_depth() {
        let set = gs(&["cache*"]);
        assert!(is_ignored(&set, Path::new("/a/b/cachedir")));
        assert!(is_ignored(&set, Path::new("/a/b/cache-stuff/x")));
        let q = gs(&["temp?"]);
        assert!(is_ignored(&q, Path::new("/a/temp1")));
        assert!(!is_ignored(&q, Path::new("/a/temporary")));
    }

    #[test]
    fn star_dir_star_protects_directory_itself() {
        let set = gs(&["**/vendor/**"]);
        assert!(is_ignored(&set, Path::new("/proj/vendor")), "dir itself");
        assert!(is_ignored(&set, Path::new("/proj/vendor/pkg")), "contents");
        assert!(!is_ignored(&set, Path::new("/proj/source")));
    }

    #[test]
    fn trailing_slash_is_normalized() {
        let set = gs(&["node_modules/"]);
        assert!(is_ignored(&set, Path::new("/proj/node_modules")));
        assert!(is_ignored(&set, Path::new("/proj/node_modules/dep")));
    }

    #[test]
    fn relative_subpath_matches_at_depth() {
        let set = gs(&["app/node_modules"]);
        assert!(is_ignored(
            &set,
            Path::new("/home/me/code/app/node_modules")
        ));
    }

    #[test]
    fn parses_toml_config() {
        let c: Config =
            toml::from_str("older_than = \"30d\"\ncaches = true\nignore = [\"foo\"]").unwrap();
        assert_eq!(c.older_than.as_deref(), Some("30d"));
        assert!(c.caches);
        assert_eq!(c.ignore, vec!["foo".to_string()]);
    }

    #[test]
    fn parses_custom_rules() {
        let c: Config = toml::from_str(
            "[[rule]]\ndir = \".mycache\"\necosystem = \"mine\"\nrequires_marker = [\"my.json\"]\n",
        )
        .unwrap();
        assert_eq!(c.rules.len(), 1);
        assert_eq!(c.rules[0].dir, ".mycache");
        assert_eq!(c.rules[0].ecosystem, "mine");
        assert_eq!(c.rules[0].requires_marker, vec!["my.json".to_string()]);

        // Conversion produces a usable rule that matches beside its marker.
        let ur = user_rules(&c.rules);
        let set: std::collections::HashSet<String> = ["my.json".to_string()].into_iter().collect();
        assert!(rules::match_dir_with(".mycache", &set, &ur).is_some());
    }

    #[test]
    fn skips_custom_rule_with_empty_dir() {
        let c: Config = toml::from_str("[[rule]]\ndir = \"\"\necosystem = \"x\"\n").unwrap();
        assert!(user_rules(&c.rules).is_empty());
    }
}
