use std::path::PathBuf;
use std::time::SystemTime;

/// A directory that can be safely removed and regenerated (build output,
/// dependencies, or a tool cache).
#[derive(Debug, Clone)]
pub struct Reclaimable {
    /// Absolute path to the artifact directory.
    pub path: PathBuf,
    /// Ecosystem the artifact belongs to (e.g. "node", "rust").
    pub ecosystem: &'static str,
    /// Directory label (e.g. "node_modules", "target").
    pub label: &'static str,
    /// Total size on disk, in bytes.
    pub size: u64,
    /// Last-modified time of the artifact dir, used as an "age" proxy.
    pub modified: Option<SystemTime>,
}
