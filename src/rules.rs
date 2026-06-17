use std::collections::HashSet;

/// A removable, regenerable artifact directory.
///
/// `requires_marker` guards ambiguous names: a directory called `target` is only
/// treated as Rust build output when a `Cargo.toml` sits beside it. An empty
/// marker list means the directory name alone is unambiguous (e.g. `__pycache__`).
pub struct Rule {
    pub dir: &'static str,
    pub ecosystem: &'static str,
    pub requires_marker: &'static [&'static str],
}

pub const RULES: &[Rule] = &[
    Rule { dir: "node_modules", ecosystem: "node", requires_marker: &["package.json"] },
    Rule { dir: "target", ecosystem: "rust", requires_marker: &["Cargo.toml"] },
    Rule { dir: ".venv", ecosystem: "python", requires_marker: &[] },
    Rule { dir: "venv", ecosystem: "python", requires_marker: &[] },
    Rule { dir: "__pycache__", ecosystem: "python", requires_marker: &[] },
    Rule { dir: ".next", ecosystem: "next.js", requires_marker: &[] },
    Rule { dir: ".nuxt", ecosystem: "nuxt", requires_marker: &[] },
    Rule { dir: ".svelte-kit", ecosystem: "svelte", requires_marker: &[] },
    Rule { dir: ".gradle", ecosystem: "gradle", requires_marker: &[] },
    Rule { dir: ".dart_tool", ecosystem: "dart", requires_marker: &[] },
    Rule { dir: "dist", ecosystem: "js", requires_marker: &["package.json"] },
    Rule {
        dir: "build",
        ecosystem: "build",
        requires_marker: &["package.json", "build.gradle", "CMakeLists.txt"],
    },
];

/// Return the matching rule for a directory `name`, given the set of sibling
/// file names (used to satisfy marker requirements).
pub fn match_dir(name: &str, siblings: &HashSet<String>) -> Option<&'static Rule> {
    RULES.iter().find(|r| {
        r.dir == name
            && (r.requires_marker.is_empty()
                || r.requires_marker.iter().any(|m| siblings.contains(*m)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn node_modules_requires_package_json() {
        assert!(match_dir("node_modules", &set(&["package.json"])).is_some());
        assert!(match_dir("node_modules", &set(&[])).is_none());
    }

    #[test]
    fn pycache_needs_no_marker() {
        assert!(match_dir("__pycache__", &set(&[])).is_some());
    }

    #[test]
    fn target_requires_cargo_toml() {
        assert!(match_dir("target", &set(&["Cargo.toml"])).is_some());
        assert!(match_dir("target", &set(&["package.json"])).is_none());
    }

    #[test]
    fn unknown_dir_does_not_match() {
        assert!(match_dir("src", &set(&["Cargo.toml"])).is_none());
    }
}
