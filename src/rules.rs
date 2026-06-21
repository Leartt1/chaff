use std::collections::HashSet;

/// A removable, regenerable artifact directory.
///
/// A directory matches when its name equals `dir` and, if a marker is required,
/// a sibling file satisfies it — either an exact name in `requires_marker` or a
/// suffix in `requires_marker_ext` (e.g. ".csproj"). Empty marker lists mean the
/// directory name alone is unambiguous (e.g. `__pycache__`); ambiguous names like
/// `bin`/`obj`/`deps` are gated by a marker so we never touch the wrong folder.
pub struct Rule {
    pub dir: &'static str,
    pub ecosystem: &'static str,
    pub requires_marker: &'static [&'static str],
    pub requires_marker_ext: &'static [&'static str],
}

pub const RULES: &[Rule] = &[
    Rule {
        dir: "node_modules",
        ecosystem: "node",
        requires_marker: &["package.json"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "target",
        ecosystem: "rust",
        requires_marker: &["Cargo.toml"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".venv",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "venv",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "__pycache__",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".next",
        ecosystem: "next.js",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".nuxt",
        ecosystem: "nuxt",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".svelte-kit",
        ecosystem: "svelte",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".gradle",
        ecosystem: "gradle",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".dart_tool",
        ecosystem: "dart",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "dist",
        ecosystem: "js",
        requires_marker: &["package.json"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "build",
        ecosystem: "build",
        requires_marker: &["package.json", "build.gradle", "CMakeLists.txt"],
        requires_marker_ext: &[],
    },
    // v0.3 — broadened coverage
    Rule {
        dir: ".terraform",
        ecosystem: "terraform",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "Pods",
        ecosystem: "cocoapods",
        requires_marker: &["Podfile"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".stack-work",
        ecosystem: "haskell",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".build",
        ecosystem: "swift",
        requires_marker: &["Package.swift"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "_build",
        ecosystem: "elixir",
        requires_marker: &["mix.exs"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "deps",
        ecosystem: "elixir",
        requires_marker: &["mix.exs"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "zig-cache",
        ecosystem: "zig",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "zig-out",
        ecosystem: "zig",
        requires_marker: &["build.zig"],
        requires_marker_ext: &[],
    },
    Rule {
        dir: "bin",
        ecosystem: "dotnet",
        requires_marker: &[],
        requires_marker_ext: &[".csproj", ".fsproj", ".vbproj", ".sln"],
    },
    Rule {
        dir: "obj",
        ecosystem: "dotnet",
        requires_marker: &[],
        requires_marker_ext: &[".csproj", ".fsproj", ".vbproj", ".sln"],
    },
    // tool caches (unambiguous names — safe without a marker)
    Rule {
        dir: ".pytest_cache",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".mypy_cache",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".ruff_cache",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".tox",
        ecosystem: "python",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".turbo",
        ecosystem: "turbo",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
    Rule {
        dir: ".parcel-cache",
        ecosystem: "parcel",
        requires_marker: &[],
        requires_marker_ext: &[],
    },
];

/// Return the matching rule for a directory `name`, given the set of sibling
/// file names (used to satisfy marker requirements).
pub fn match_dir(name: &str, siblings: &HashSet<String>) -> Option<&'static Rule> {
    RULES
        .iter()
        .find(|r| r.dir == name && marker_satisfied(r, siblings))
}

fn marker_satisfied(r: &Rule, siblings: &HashSet<String>) -> bool {
    if r.requires_marker.is_empty() && r.requires_marker_ext.is_empty() {
        return true;
    }
    r.requires_marker.iter().any(|m| siblings.contains(*m))
        || r.requires_marker_ext
            .iter()
            .any(|ext| siblings.iter().any(|s| s.ends_with(ext)))
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

    #[test]
    fn terraform_needs_no_marker() {
        assert!(match_dir(".terraform", &set(&[])).is_some());
    }

    #[test]
    fn cocoapods_requires_podfile() {
        assert!(match_dir("Pods", &set(&["Podfile"])).is_some());
        assert!(match_dir("Pods", &set(&[])).is_none());
    }

    #[test]
    fn dotnet_bin_requires_project_file_by_extension() {
        assert!(match_dir("bin", &set(&["App.csproj"])).is_some());
        assert!(match_dir("obj", &set(&["Solution.sln"])).is_some());
        // a bare bin/ with no .NET project beside it is left alone
        assert!(match_dir("bin", &set(&["README.md"])).is_none());
    }

    #[test]
    fn elixir_deps_requires_mix_exs() {
        assert!(match_dir("deps", &set(&["mix.exs"])).is_some());
        assert!(match_dir("deps", &set(&[])).is_none());
    }

    #[test]
    fn tool_caches_match_without_marker() {
        for d in [
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".tox",
            ".turbo",
            ".parcel-cache",
        ] {
            assert!(match_dir(d, &set(&[])).is_some(), "{d} should match");
        }
    }
}
