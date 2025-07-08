use std::path::PathBuf;

use tombi_config::{ConfigLevel, FilesOptions};
use tombi_glob::WalkDir;

/// Input source for TOML files.
///
/// Standard input or file paths. Contains a list of files that match the glob pattern.
#[derive(Debug)]
pub enum FileInput {
    Stdin,
    Files(Vec<Result<PathBuf, crate::Error>>),
}

impl FileInput {
    pub async fn new<T: AsRef<str>>(
        files: &[T],
        config_path: Option<&std::path::Path>,
        config_level: ConfigLevel,
        files_options: FilesOptions,
    ) -> Self {
        let root = match config_level {
            ConfigLevel::Project => config_path.and_then(|p| p.parent()).unwrap_or(".".as_ref()),
            _ => ".".as_ref(),
        };

        match files.len() {
            0 => {
                tracing::debug!("Searching for TOML files using configured patterns...");

                FileInput::Files(search_with_patterns_async(root, files_options).await)
            }
            1 if files[0].as_ref() == "-" => FileInput::Stdin,
            _ => {
                tracing::debug!("Searching for TOML files using user input patterns...");

                let mut matched_paths = Vec::with_capacity(100);

                for file_input in files {
                    let file_path = file_input.as_ref();

                    if is_glob_pattern(file_path) || file_path.ends_with(".toml") {
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

                FileInput::Files(matched_paths)
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            FileInput::Stdin => 1,
            FileInput::Files(files) => files.len(),
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
