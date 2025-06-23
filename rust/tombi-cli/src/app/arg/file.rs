use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

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
    pub fn new<T: AsRef<str>>(
        files: &[T],
        include_patterns: Option<&[&str]>,
        exclude_patterns: Option<&[&str]>,
    ) -> Self {
        let include_patterns = include_patterns.unwrap_or(DEFAULT_INCLUDE_PATTERNS);
        let exclude_patterns = exclude_patterns.unwrap_or_default();

        // Pre-allocate with estimated capacity
        let mut matched_paths = Vec::with_capacity(100);

        match files.len() {
            0 => {
                tracing::debug!("Searching for TOML files using configured patterns...");
                tracing::debug!("Include patterns: {:?}", include_patterns);
                tracing::debug!("Exclude patterns: {:?}", exclude_patterns);

                let exclude_matchers = match compile_exclude_patterns(exclude_patterns) {
                    Ok(matchers) => matchers,
                    Err(errors) => {
                        return FileInput::Files(errors.into_iter().map(Err).collect());
                    }
                };

                let pattern_results = include_patterns
                    .par_iter()
                    .map(|pattern| compile_include_patterns(pattern, &exclude_matchers))
                    .collect::<Vec<_>>();

                // Collect results
                for result in pattern_results {
                    match result {
                        Ok(paths) => matched_paths.extend(paths.into_iter().map(Ok)),
                        Err(err) => matched_paths.push(Err(err)),
                    }
                }

                FileInput::Files(matched_paths)
            }
            1 if files[0].as_ref() == "-" => FileInput::Stdin,
            _ => {
                tracing::debug!("Searching for TOML files using user input patterns...");
                tracing::debug!("Exclude patterns: {:?}", exclude_patterns);

                let exclude_matchers = match compile_exclude_patterns(exclude_patterns) {
                    Ok(matchers) => matchers,
                    Err(errors) => {
                        return FileInput::Files(errors.into_iter().map(Err).collect());
                    }
                };

                // Convert to owned strings for parallel processing
                let file_strings: Vec<String> =
                    files.iter().map(|f| f.as_ref().to_string()).collect();

                // Process files in parallel
                let file_results = file_strings
                    .par_iter()
                    .map(|file_path| {
                        if is_glob_pattern(file_path) {
                            compile_include_patterns(file_path, &exclude_matchers)
                        } else {
                            let path = PathBuf::from(file_path);
                            if path.exists() {
                                Ok(vec![path])
                            } else {
                                Err(crate::Error::FileNotFound(path))
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                // Collect results
                for result in file_results {
                    match result {
                        Ok(paths) => matched_paths.extend(paths.into_iter().map(Ok)),
                        Err(e) => matched_paths.push(Err(e)),
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

#[inline]
fn compile_include_patterns(
    pattern: &str,
    exclude_matchers: &[glob::Pattern],
) -> Result<Vec<PathBuf>, crate::Error> {
    match glob::glob(pattern) {
        Ok(paths) => Ok(paths
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .filter(|path| {
                !exclude_matchers
                    .iter()
                    .any(|matcher| matcher.matches_path(path))
            })
            .collect()),
        Err(_) => Err(crate::Error::GlobPatternInvalid(pattern.to_string())),
    }
}

fn compile_exclude_patterns(
    exclude_patterns: &[&str],
) -> Result<Vec<glob::Pattern>, Vec<crate::Error>> {
    let mut exclude_matchers = Vec::with_capacity(exclude_patterns.len());
    let mut exclude_errors = Vec::with_capacity(exclude_patterns.len());

    for pattern in exclude_patterns {
        match glob::Pattern::new(pattern) {
            Ok(matcher) => exclude_matchers.push(matcher),
            Err(_) => exclude_errors.push(crate::Error::GlobPatternInvalid(pattern.to_string())),
        }
    }

    if !exclude_errors.is_empty() {
        return Err(exclude_errors);
    }

    Ok(exclude_matchers)
}
