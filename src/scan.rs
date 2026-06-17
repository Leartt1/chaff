use crate::model::Reclaimable;
use crate::rules::{self, Rule};
use crate::size;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Version-control and similar directories we must never descend into or touch.
const SKIP_DIRS: &[&str] = &[".git", ".hg", ".svn"];

type Hit = (PathBuf, &'static Rule, Option<SystemTime>);

/// Discover reclaimable artifacts under each root, then compute their sizes in
/// parallel. Matched artifact directories are not descended into.
pub fn scan(roots: &[PathBuf]) -> Vec<Reclaimable> {
    let mut hits: Vec<Hit> = Vec::new();
    for root in roots {
        discover(root, &mut hits);
    }
    hits.into_par_iter()
        .map(|(path, rule, modified)| Reclaimable {
            size: size::dir_size(&path),
            path,
            ecosystem: rule.ecosystem,
            label: rule.dir,
            modified,
        })
        .collect()
}

fn discover(root: &Path, out: &mut Vec<Hit>) {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries: Vec<fs::DirEntry> = match fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(Result::ok).collect(),
            Err(_) => continue,
        };

        let siblings: HashSet<String> = entries
            .iter()
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        for e in &entries {
            let ft = match e.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            // Skip files and symlinks (never follow links out of the tree).
            if !ft.is_dir() {
                continue;
            }
            let name = match e.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            if let Some(rule) = rules::match_dir(&name, &siblings) {
                let modified = e.metadata().ok().and_then(|m| m.modified().ok());
                out.push((e.path(), rule, modified));
                // Prune: do not descend into a matched artifact directory.
            } else {
                stack.push(e.path());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn unique_tmp() -> PathBuf {
        std::env::temp_dir().join(format!("chaff_scan_test_{}", std::process::id()))
    }

    fn write_file(path: &Path, bytes: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(&vec![0u8; bytes]).unwrap();
    }

    #[test]
    fn finds_node_modules_and_target_but_not_src() {
        let root = unique_tmp();
        let _ = fs::remove_dir_all(&root);

        // node project
        write_file(&root.join("web/package.json"), 2);
        write_file(&root.join("web/node_modules/dep/index.js"), 1000);
        write_file(&root.join("web/src/app.js"), 50);
        // rust project
        write_file(&root.join("cli/Cargo.toml"), 2);
        write_file(&root.join("cli/target/debug/bin"), 4000);
        // a bare "target" dir with no Cargo.toml must be ignored
        write_file(&root.join("notes/target/file.txt"), 9999);

        let items = scan(std::slice::from_ref(&root));
        let labels: HashSet<&str> = items.iter().map(|i| i.label).collect();

        assert!(labels.contains("node_modules"), "should find node_modules");
        assert!(labels.contains("target"), "should find rust target");
        assert_eq!(items.len(), 2, "must not match the markerless target dir");

        let nm = items.iter().find(|i| i.label == "node_modules").unwrap();
        assert_eq!(nm.size, 1000);

        let _ = fs::remove_dir_all(&root);
    }
}
