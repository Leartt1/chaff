use crate::model::Reclaimable;
use crate::size;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// A global, regenerable package-manager cache. `rel` lists candidate locations
/// relative to the home directory (platform variants); the first that exists wins.
struct CacheDef {
    tool: &'static str,
    rel: &'static [&'static str],
}

const CACHES: &[CacheDef] = &[
    CacheDef {
        tool: "npm",
        rel: &[".npm/_cacache"],
    },
    CacheDef {
        tool: "pnpm",
        rel: &[
            "Library/pnpm/store",
            ".local/share/pnpm/store",
            ".pnpm-store",
        ],
    },
    CacheDef {
        tool: "yarn",
        rel: &["Library/Caches/Yarn", ".cache/yarn"],
    },
    CacheDef {
        tool: "pip",
        rel: &["Library/Caches/pip", ".cache/pip"],
    },
    CacheDef {
        tool: "uv",
        rel: &["Library/Caches/uv", ".cache/uv"],
    },
    CacheDef {
        tool: "cargo",
        rel: &[".cargo/registry/cache"],
    },
    CacheDef {
        tool: "go",
        rel: &["Library/Caches/go-build", ".cache/go-build"],
    },
    CacheDef {
        tool: "go-mod",
        rel: &["go/pkg/mod"],
    },
    CacheDef {
        tool: "gradle",
        rel: &[".gradle/caches"],
    },
    CacheDef {
        tool: "maven",
        rel: &[".m2/repository"],
    },
    CacheDef {
        tool: "huggingface",
        rel: &[".cache/huggingface", "Library/Caches/huggingface"],
    },
    CacheDef {
        tool: "xcode",
        rel: &["Library/Developer/Xcode/DerivedData"],
    },
    CacheDef {
        tool: "homebrew",
        rel: &["Library/Caches/Homebrew", ".cache/Homebrew"],
    },
    CacheDef {
        tool: "deno",
        rel: &["Library/Caches/deno", ".cache/deno", ".deno"],
    },
    CacheDef {
        tool: "composer",
        rel: &[
            "Library/Caches/composer",
            ".cache/composer",
            ".composer/cache",
        ],
    },
];

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Discover and size known global caches under the user's home directory.
pub fn scan_caches() -> Vec<Reclaimable> {
    match home() {
        Some(h) => scan_caches_in(&h),
        None => Vec::new(),
    }
}

fn scan_caches_in(home: &Path) -> Vec<Reclaimable> {
    let mut hits: Vec<(PathBuf, &'static str)> = Vec::new();
    for def in CACHES {
        for rel in def.rel {
            let p = home.join(rel);
            if p.is_dir() {
                hits.push((p, def.tool));
                break;
            }
        }
    }

    hits.into_par_iter()
        .map(|(path, tool)| Reclaimable {
            modified: std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok()),
            size: size::dir_size(&path),
            path,
            ecosystem: "cache",
            label: tool,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;

    #[test]
    fn finds_known_caches_under_home() {
        let home = std::env::temp_dir().join(format!("chaff_cache_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(home.join(".cache/pip")).unwrap();
        fs::create_dir_all(home.join(".cargo/registry/cache")).unwrap();
        fs::create_dir_all(home.join("unrelated")).unwrap();

        let found = scan_caches_in(&home);
        let tools: HashSet<&str> = found.iter().map(|r| r.label).collect();

        assert!(tools.contains("pip"));
        assert!(tools.contains("cargo"));
        assert!(found.iter().all(|r| r.ecosystem == "cache"));

        let _ = fs::remove_dir_all(&home);
    }
}
