mod error;
use fast_glob::glob_match;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use error::Error;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub follow_links: bool,
    pub hidden: bool,
    pub ignore_files: bool,
    pub git_ignore: bool,
    pub max_depth: Option<usize>,
    pub max_filesize: Option<u64>,
    pub threads: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            follow_links: false,
            hidden: false,
            ignore_files: true,
            git_ignore: true,
            max_depth: None,
            max_filesize: None,
            threads: rayon::current_num_threads(),
        }
    }
}

/// Validate a glob pattern
fn validate_pattern(pattern: &str) -> Result<(), crate::Error> {
    if pattern.is_empty() {
        return Err(crate::Error::EmptyPattern);
    }

    // Validate pattern by trying to match empty string (fast-glob will panic on invalid patterns)
    let _ = glob_match(pattern, "");
    Ok(())
}

/// Check if a path matches a glob pattern
fn matches_pattern(pattern: &str, path: &str) -> bool {
    glob_match(pattern, path)
}

/// WalkDir-like structure for parallel async directory walking
pub struct WalkDir {
    root: PathBuf,
    options: SearchOptions,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl WalkDir {
    /// Create a new WalkDir instance
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            options: SearchOptions::default(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Create a new WalkDir instance with custom options
    pub fn with_options<P: AsRef<Path>>(root: P, options: SearchOptions) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            options,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Add include patterns
    pub fn includes(mut self, patterns: &[&str]) -> Result<Self, crate::Error> {
        for pattern in patterns {
            validate_pattern(pattern)?;
            self.include_patterns.push(pattern.to_string());
        }
        Ok(self)
    }

    /// Add exclude patterns
    pub fn excludes(mut self, patterns: &[&str]) -> Result<Self, crate::Error> {
        for pattern in patterns {
            validate_pattern(pattern)?;
            self.exclude_patterns.push(pattern.to_string());
        }
        Ok(self)
    }

    /// Walk the directory asynchronously and return matching file paths
    pub async fn walk(self) -> Result<Vec<PathBuf>, crate::Error> {
        let root_path = &self.root;

        if !root_path.exists() {
            return Err(crate::Error::RootPathNotFound {
                path: root_path.to_path_buf(),
            });
        }

        if !root_path.is_dir() {
            return Err(crate::Error::RootPathNotDirectory {
                path: root_path.to_path_buf(),
            });
        }

        let results = Arc::new(Mutex::new(Vec::new()));

        let mut builder = WalkBuilder::new(root_path);
        builder
            .follow_links(self.options.follow_links)
            .hidden(self.options.hidden)
            .ignore(!self.options.ignore_files)
            .git_ignore(self.options.git_ignore)
            .threads(self.options.threads);

        if let Some(max_depth) = self.options.max_depth {
            builder.max_depth(Some(max_depth));
        }

        if let Some(max_filesize) = self.options.max_filesize {
            builder.max_filesize(Some(max_filesize));
        }

        let walker = builder.build_parallel();

        walker.run(|| {
            let results_clone = Arc::clone(&results);
            let include_patterns = self.include_patterns.clone();
            let exclude_patterns = self.exclude_patterns.clone();
            let root_path = root_path.to_path_buf();

            Box::new(move |entry_result| {
                match entry_result {
                    Ok(entry) => {
                        if let Some(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                let path = entry.path();
                                let relative_path =
                                    if let Ok(rel_path) = path.strip_prefix(&root_path) {
                                        rel_path.to_string_lossy()
                                    } else {
                                        path.to_string_lossy()
                                    };

                                // Check if file matches any include pattern
                                let mut should_include = include_patterns.is_empty();
                                for pattern in &include_patterns {
                                    if matches_pattern(pattern, &relative_path) {
                                        should_include = true;
                                        break;
                                    }
                                }

                                if should_include {
                                    // Check if file should be excluded
                                    let mut should_exclude = false;
                                    for pattern in &exclude_patterns {
                                        if matches_pattern(pattern, &relative_path) {
                                            should_exclude = true;
                                            break;
                                        }
                                    }

                                    if !should_exclude {
                                        if let Ok(mut results_guard) = results_clone.lock() {
                                            results_guard.push(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore errors and continue
                    }
                }
                ignore::WalkState::Continue
            })
        });

        let results = Arc::try_unwrap(results)
            .map_err(|_| crate::Error::LockError)?
            .into_inner()
            .map_err(|_| crate::Error::LockError)?;

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Convenience functions using the new API
    fn find_rust_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, crate::Error> {
        let walker = WalkDir::new(root).includes(&["*.rs"])?;
        // Note: This is a blocking version, async version would need tokio runtime
        // For now, we'll use a simple implementation
        let root_path = walker.root;
        if !root_path.exists() {
            return Err(crate::Error::RootPathNotFound {
                path: root_path.to_path_buf(),
            });
        }

        if !root_path.is_dir() {
            return Err(crate::Error::RootPathNotDirectory {
                path: root_path.to_path_buf(),
            });
        }

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut builder = WalkBuilder::new(&root_path);
        builder
            .follow_links(false)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .threads(rayon::current_num_threads());

        let walker = builder.build_parallel();

        walker.run(|| {
            let results_clone = Arc::clone(&results);
            Box::new(move |entry_result| {
                match entry_result {
                    Ok(entry) => {
                        if let Some(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                let path = entry.path();
                                let filename =
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                                if matches_pattern("*.rs", filename) {
                                    if let Ok(mut results_guard) = results_clone.lock() {
                                        results_guard.push(path.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore errors and continue
                    }
                }
                ignore::WalkState::Continue
            })
        });

        let results = Arc::try_unwrap(results)
            .map_err(|_| crate::Error::LockError)?
            .into_inner()
            .map_err(|_| crate::Error::LockError)?;

        Ok(results)
    }

    fn find_toml_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, crate::Error> {
        let walker = WalkDir::new(root).includes(&["*.toml"])?;
        // Similar blocking implementation as find_rust_files
        let root_path = walker.root;
        if !root_path.exists() {
            return Err(crate::Error::RootPathNotFound {
                path: root_path.to_path_buf(),
            });
        }

        if !root_path.is_dir() {
            return Err(crate::Error::RootPathNotDirectory {
                path: root_path.to_path_buf(),
            });
        }

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut builder = WalkBuilder::new(&root_path);
        builder
            .follow_links(false)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .threads(rayon::current_num_threads());

        let walker = builder.build_parallel();

        walker.run(|| {
            let results_clone = Arc::clone(&results);
            Box::new(move |entry_result| {
                match entry_result {
                    Ok(entry) => {
                        if let Some(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                let path = entry.path();
                                let filename =
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                                if matches_pattern("*.toml", filename) {
                                    if let Ok(mut results_guard) = results_clone.lock() {
                                        results_guard.push(path.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore errors and continue
                    }
                }
                ignore::WalkState::Continue
            })
        });

        let results = Arc::try_unwrap(results)
            .map_err(|_| crate::Error::LockError)?
            .into_inner()
            .map_err(|_| crate::Error::LockError)?;

        Ok(results)
    }

    #[test]
    fn test_walkdir_creation() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new(&current_dir);
        assert_eq!(walker.root, current_dir);
    }

    #[test]
    fn test_walkdir_with_options() {
        let current_dir = std::env::current_dir().unwrap();
        let options = SearchOptions {
            hidden: true,
            max_depth: Some(3),
            ..Default::default()
        };
        let walker = WalkDir::with_options(&current_dir, options);
        assert_eq!(walker.root, current_dir);
        assert!(walker.options.hidden);
        assert_eq!(walker.options.max_depth, Some(3));
    }

    #[test]
    fn test_walkdir_includes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new(&current_dir)
            .includes(&["*.rs", "*.toml"])
            .unwrap();
        assert_eq!(walker.include_patterns, vec!["*.rs", "*.toml"]);
    }

    #[test]
    fn test_walkdir_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new(&current_dir)
            .excludes(&["target/**", "node_modules/**"])
            .unwrap();
        assert_eq!(
            walker.exclude_patterns,
            vec!["target/**", "node_modules/**"]
        );
    }

    #[test]
    fn test_walkdir_includes_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new(&current_dir)
            .includes(&["*.rs"])
            .unwrap()
            .excludes(&["target/**"])
            .unwrap();
        assert_eq!(walker.include_patterns, vec!["*.rs"]);
        assert_eq!(walker.exclude_patterns, vec!["target/**"]);
    }

    #[test]
    fn test_invalid_pattern() {
        let current_dir = std::env::current_dir().unwrap();
        let result = WalkDir::new(&current_dir).includes(&["invalid[pattern"]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_walkdir_walk() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new(&current_dir).includes(&["*.rs"]).unwrap();
        let result = walker.walk().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_walkdir_walk_with_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new(&current_dir)
            .includes(&["*.toml"])
            .unwrap()
            .excludes(&["target/**"])
            .unwrap();
        let result = walker.walk().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_convenience_functions() {
        let current_dir = std::env::current_dir().unwrap();

        let _ = find_rust_files(&current_dir);
        let _ = find_toml_files(&current_dir);
    }

    #[test]
    fn test_error_handling() {
        let result = find_rust_files("/nonexistent/path");
        assert!(matches!(result, Err(crate::Error::RootPathNotFound { .. })));
    }
}
