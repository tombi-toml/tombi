use std::borrow::Cow;
use std::path::Path;

use fast_glob::glob_match;

/// Returns a path string for glob matching.
/// If `path` is under `root`, returns a relative path from `root`.
/// Otherwise returns the full path string.
pub(crate) fn path_for_patterns<'a>(path: &'a Path, root: &Path) -> Cow<'a, str> {
    if let Ok(rel_path) = path.strip_prefix(root) {
        rel_path.to_string_lossy()
    } else {
        path.to_string_lossy()
    }
}

/// Returns true if `path_for_patterns` matches any pattern.
pub(crate) fn matches_any_pattern(path_for_patterns: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| glob_match(pattern, path_for_patterns))
}

/// Returns true if `path` matches any pattern, using the same relative-path rules as discovery.
pub(crate) fn path_matches_patterns(path: &Path, root: &Path, patterns: &[String]) -> bool {
    let path_for_patterns = path_for_patterns(path, root);
    matches_any_pattern(path_for_patterns.as_ref(), patterns)
}
