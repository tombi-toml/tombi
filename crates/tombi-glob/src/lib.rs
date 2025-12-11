mod error;
mod file_match;
mod file_search;
mod walk_dir;

use std::path::Path;

use fast_glob::glob_match;
use tombi_config::{FormatOptions, LintOptions, OverrideFilesOptions};

pub use error::Error;
pub use file_match::{MatchResult, matches_file_patterns};
pub use file_search::{FileInputType, FileSearch, search_pattern_matched_paths};
pub use walk_dir::WalkDir;

pub fn get_format_options(
    config: &tombi_config::Config,
    text_document_path: Option<&Path>,
    config_path: Option<&Path>,
) -> Option<FormatOptions> {
    if let Some(text_document_path) = text_document_path {
        // Check overrides
        if let Some(overrides) = config.overrides() {
            for override_item in overrides.iter() {
                if matches_override_files(text_document_path, config_path, &override_item.files) {
                    // Check if format is enabled
                    if let Some(format) = &override_item.format {
                        if let Some(enabled) = &format.enabled {
                            if !enabled.value() {
                                return None;
                            }
                        }
                        return Some(config.format(Some(format)));
                    }
                    break;
                }
            }
        }
    }

    Some(config.format(None))
}

pub fn get_lint_options(
    config: &tombi_config::Config,
    text_document_path: Option<&Path>,
    config_path: Option<&Path>,
) -> Option<LintOptions> {
    if let Some(text_document_path) = text_document_path {
        // Check overrides
        if let Some(overrides) = config.overrides() {
            for override_item in overrides.iter() {
                if matches_override_files(text_document_path, config_path, &override_item.files) {
                    // Check if lint is enabled
                    if let Some(lint) = &override_item.lint {
                        if let Some(enabled) = &lint.enabled {
                            if !enabled.value() {
                                return None;
                            }
                        }
                        return Some(config.lint(Some(lint)));
                    }
                    break;
                }
            }
        }
    }

    Some(config.lint(None))
}

/// Check if a path matches override files patterns
fn matches_override_files(
    text_document_path: &Path,
    config_path: Option<&Path>,
    files: &OverrideFilesOptions,
) -> bool {
    let text_document_absolute_path = match text_document_path.canonicalize() {
        Ok(path) => path,
        Err(_) => text_document_path.to_path_buf(),
    };

    // Determine the path to use for pattern matching
    let path_for_patterns = relative_document_text_path(&text_document_absolute_path, config_path);

    // Check include patterns first
    let mut matches_include = false;
    for include_pattern in files.include.iter() {
        if glob_match(include_pattern, path_for_patterns.as_ref()) {
            matches_include = true;
            break;
        }
    }
    if !matches_include {
        return false;
    }

    // Check exclude patterns
    if let Some(exclude) = &files.exclude {
        for exclude_pattern in exclude.iter() {
            if glob_match(exclude_pattern, path_for_patterns.as_ref()) {
                return false;
            }
        }
    }

    true
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

        if let Some(config_dir) = config_pathbuf.parent() {
            if text_document_absolute_path.starts_with(config_dir) {
                // Use relative path from config directory
                if let Ok(rel_path) = text_document_absolute_path.strip_prefix(config_dir) {
                    return rel_path.to_string_lossy();
                }
            }
        }
    }
    text_document_absolute_path.to_string_lossy()
}
