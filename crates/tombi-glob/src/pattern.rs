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
pub(crate) fn matches_any_pattern<T: AsRef<str>>(path_for_patterns: &str, patterns: &[T]) -> bool {
    patterns
        .iter()
        .filter_map(|pattern| {
            let pattern = pattern.as_ref();
            (!pattern.is_empty()).then_some(pattern)
        })
        .any(|pattern| glob_match(pattern, path_for_patterns))
}

/// Returns true if `path` matches any pattern, using the same relative-path rules as discovery.
pub(crate) fn path_matches_patterns<T: AsRef<str>>(
    path: &Path,
    root: &Path,
    patterns: &[T],
) -> bool {
    let path_for_patterns = path_for_patterns(path, root);
    matches_any_pattern(path_for_patterns.as_ref(), patterns)
}
