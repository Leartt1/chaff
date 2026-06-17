use std::path::Path;
use std::process::Command;

/// Returns true if any file under `artifact` is tracked by git.
///
/// A tracked file means the directory was deliberately committed and is *not*
/// throwaway, so chaff must refuse to delete it. If git is missing or the path
/// is not inside a repository, there is nothing tracked to protect.
pub fn contains_tracked_files(artifact: &Path) -> bool {
    let output = Command::new("git")
        .arg("-C")
        .arg(artifact)
        .args(["ls-files", "-z", "."])
        .output();
    matches!(output, Ok(o) if o.status.success() && !o.stdout.is_empty())
}
