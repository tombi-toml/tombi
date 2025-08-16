use std::path::{Path, PathBuf};

use fast_glob::glob_match;
use tombi_config::{Config, ConfigLevel, FilesOptions};
use tombi_glob::WalkDir;
mod error;

pub use error::Error;

/// Input source for TOML files.
///
/// Standard input or file paths. Contains a list of files that match the glob pattern.
#[derive(Debug)]
pub enum FileSearch {
    Stdin,
    Files(Vec<Result<PathBuf, crate::Error>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileInputType {
    Stdin,
    Project,
    Files,
}

impl<T: AsRef<str>> From<&[T]> for FileInputType {
    fn from(files: &[T]) -> Self {
        match files.len() {
            0 => FileInputType::Project,
            1 if files[0].as_ref() == "-" => FileInputType::Stdin,
            _ => FileInputType::Files,
        }
    }
}

impl FileSearch {
    pub async fn new<T: AsRef<str>>(
        files: &[T],
        config: &Config,
        config_path: Option<&std::path::Path>,
        config_level: ConfigLevel,
    ) -> Self {
        let root = match config_level {
            ConfigLevel::Project => config_path.and_then(|p| p.parent()).unwrap_or(".".as_ref()),
            _ => ".".as_ref(),
        };
        let files_options = config.files.clone().unwrap_or_default();

        match FileInputType::from(files) {
            FileInputType::Stdin => FileSearch::Stdin,
            FileInputType::Project => {
                tracing::debug!("Searching for TOML files using configured patterns...");

                FileSearch::Files(search_with_patterns_async(root, files_options).await)
            }
            FileInputType::Files => {
                tracing::debug!("Searching for TOML files using user input patterns...");

                let mut matched_paths = Vec::with_capacity(100);

                for file_input in files {
                    let file_path = file_input.as_ref();

                    if is_glob_pattern(file_path) {
                        matched_paths.extend(
                            search_with_patterns_async(
                                root,
                                FilesOptions {
                                    include: Some(vec![file_path.to_string()]),
                                    exclude: None,
                                },
                            )
                            .await,
                        );
                    } else {
                        let path = PathBuf::from(file_path);
                        if path.is_file() {
                            matched_paths.push(Ok(path));
                        } else if path.is_dir() {
                            matched_paths.extend(
                                search_with_patterns_async(path, files_options.clone()).await,
                            );
                        } else {
                            matched_paths.push(Err(crate::Error::FileNotFound(path)));
                        }
                    }
                }

                FileSearch::Files(matched_paths)
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            FileSearch::Stdin => 1,
            FileSearch::Files(files) => files.len(),
        }
    }
}

fn is_glob_pattern(value: &str) -> bool {
    for c in value.chars() {
        if matches!(c, '*' | '?' | '[' | ']') {
            return true;
        }
    }
    false
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

async fn search_with_patterns_async<P: AsRef<std::path::Path>>(
    root: P,
    files_options: FilesOptions,
) -> Vec<Result<PathBuf, crate::Error>> {
    tracing::debug!("Include patterns: {:?}", files_options.include);
    tracing::debug!("Exclude patterns: {:?}", files_options.exclude);

    match WalkDir::new_with_options(root, files_options).walk().await {
        Ok(results) => {
            let matched_paths: Vec<Result<PathBuf, crate::Error>> =
                results.into_iter().map(Ok).collect();
            matched_paths
        }
        Err(err) => {
            vec![Err(crate::Error::GlobSearchFailed(err))]
        }
    }
}
