use std::path::PathBuf;

use tombi_glob::WalkDir;

const DEFAULT_INCLUDE_PATTERNS: &[&str] = &["**/*.toml"];

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
        include_patterns: Option<&[&str]>,
        exclude_patterns: Option<&[&str]>,
    ) -> Self {
        let include_patterns = include_patterns.unwrap_or(DEFAULT_INCLUDE_PATTERNS);
        let exclude_patterns = exclude_patterns.unwrap_or_default();

        match files.len() {
            0 => {
                tracing::debug!("Searching for TOML files using configured patterns...");
                tracing::debug!("Include patterns: {:?}", include_patterns);
                tracing::debug!("Exclude patterns: {:?}", exclude_patterns);

                FileInput::Files(
                    search_with_patterns_async(".", include_patterns, exclude_patterns).await,
                )
            }
            1 if files[0].as_ref() == "-" => FileInput::Stdin,
            _ => {
                tracing::debug!("Searching for TOML files using user input patterns...");
                tracing::debug!("Exclude patterns: {:?}", exclude_patterns);

                let mut matched_paths = Vec::with_capacity(100);

                for file_input in files {
                    let file_path = file_input.as_ref();

                    if is_glob_pattern(file_path) {
                        matched_paths.extend(
                            search_with_patterns_async(".", &[file_path], exclude_patterns).await,
                        );
                    } else {
                        let path = PathBuf::from(file_path);
                        if path.exists() {
                            matched_paths.push(Ok(path));
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

async fn search_with_patterns_async(
    root: &str,
    include_patterns: &[&str],
    exclude_patterns: &[&str],
) -> Vec<Result<PathBuf, crate::Error>> {
    let mut walker = WalkDir::new(root);

    if !include_patterns.is_empty() {
        walker = walker.includes(include_patterns);
    }

    if !exclude_patterns.is_empty() {
        walker = walker.excludes(exclude_patterns);
    }

    match walker.walk().await {
        Ok(results) => {
            let matched_paths: Vec<Result<PathBuf, crate::Error>> =
                results.into_iter().map(|r| Ok(r)).collect();
            matched_paths
        }
        Err(err) => {
            vec![Err(crate::Error::GlobSearchFailed(err))]
        }
    }
}
