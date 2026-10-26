use std::path::{Path, PathBuf};

use fast_glob::glob_match;
use tombi_config::{Config, config_base_dir};

use crate::is_path_ignored;

#[derive(Debug, PartialEq, Eq)]
pub enum MatchResult {
    Matched,
    IncludeNotMatched,
    ExcludeMatched,
    Ignored,
}

pub fn matches_file_patterns(
    text_document_path: &Path,
    config_path: Option<&Path>,
    config: &Config,
    ignore_roots: &[PathBuf],
) -> MatchResult {
    let text_document_absolute_path = match text_document_path.canonicalize() {
        Ok(path) => path,
        Err(_) => text_document_path.to_path_buf(),
    };

    let files = config.files.clone().unwrap_or_default();

    // Determine the path to use for pattern matching
    let path_for_patterns = relative_document_text_path(&text_document_absolute_path, config_path);

    // Check include patterns first
    if let Some(include) = files.include.as_ref() {
        let mut matches_include = false;
        for include_pattern in include.iter() {
            if glob_match(include_pattern, path_for_patterns.as_ref()) {
                matches_include = true;
                break;
            }
        }
        if !matches_include {
            return MatchResult::IncludeNotMatched;
        }
    }

    // Check exclude patterns
    if let Some(exclude) = files.exclude.as_ref() {
        for exclude_pattern in exclude.iter() {
            if glob_match(exclude_pattern, path_for_patterns.as_ref()) {
                return MatchResult::ExcludeMatched;
            }
        }
    }

    if files.respect_ignore_files.value()
        && is_ignored_in_every_root(ignore_roots, text_document_path)
    {
        return MatchResult::Ignored;
    }

    MatchResult::Matched
}

/// Whether `path` is excluded by ignore files in every root containing it.
/// A path outside all roots is never ignored.
fn is_ignored_in_every_root(roots: &[PathBuf], path: &Path) -> bool {
    let mut containing_roots = roots
        .iter()
        .filter(|root| path.starts_with(root))
        .peekable();

    containing_roots.peek().is_some() && containing_roots.all(|root| is_path_ignored(root, path))
}

/// Determine the path to use for pattern matching
/// Returns relative path from config directory if possible, otherwise absolute path
fn relative_document_text_path<'a>(
    text_document_absolute_path: &'a Path,
    config_path: Option<&Path>,
) -> std::borrow::Cow<'a, str> {
    if let Some(config_path) = config_path {
        let config_pathbuf = match config_path.canonicalize() {
            Ok(path) => path,
            Err(_) => config_path.to_path_buf(),
        };

        if let Some(config_dir) = config_base_dir(&config_pathbuf)
            && text_document_absolute_path.starts_with(config_dir)
        {
            // Use relative path from config directory
            if let Ok(rel_path) = text_document_absolute_path.strip_prefix(config_dir) {
                return rel_path.to_string_lossy();
            }
        }
    }
    text_document_absolute_path.to_string_lossy()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use tombi_config::FilesOptions;

    use super::*;

    macro_rules! test_matches_file_patterns_ignore {
        ($name:ident, respect_ignore_files = $respect_ignore_files:literal, in_root = $in_root:literal, $expected:expr) => {
            #[test]
            fn $name() {
                let tempdir = tempdir().unwrap();
                let root = tempdir.path().canonicalize().unwrap();
                fs::write(root.join(".ignore"), ".*\n").unwrap();
                let path = root.join(".test.toml");
                fs::write(&path, "[]\n").unwrap();

                let mut config = Config::default();
                config.files = Some(FilesOptions {
                    respect_ignore_files: $respect_ignore_files.into(),
                    ..Default::default()
                });
                let ignore_roots = if $in_root {
                    vec![root.clone()]
                } else {
                    vec![root.join("other")]
                };

                assert_eq!(
                    matches_file_patterns(&path, None, &config, &ignore_roots),
                    $expected
                );
            }
        };
    }

    test_matches_file_patterns_ignore!(
        ignored_by_ignore_file,
        respect_ignore_files = true,
        in_root = true,
        MatchResult::Ignored
    );
    test_matches_file_patterns_ignore!(
        ignore_files_not_respected,
        respect_ignore_files = false,
        in_root = true,
        MatchResult::Matched
    );
    test_matches_file_patterns_ignore!(
        path_outside_ignore_roots,
        respect_ignore_files = true,
        in_root = false,
        MatchResult::Matched
    );
}
