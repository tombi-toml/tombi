use std::path::Path;

use fast_glob::glob_match;
use tombi_config::Config;

pub fn is_target_text_document_path(
    text_document_path: &Path,
    config_path: Option<&Path>,
    config: &Config,
) -> bool {
    let Some(files) = config.files.as_ref() else {
        return true;
    };

    let text_document_absolute_path = match text_document_path.canonicalize() {
        Ok(path) => path,
        Err(_) => text_document_path.to_path_buf(),
    };

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
            return false;
        }
    }

    // Check exclude patterns
    if let Some(exclude) = files.exclude.as_ref() {
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
