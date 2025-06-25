mod error;
use fast_glob::glob_match;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use error::Error;
use tombi_config::FilesOptions;

/// WalkDir-like structure for parallel async directory walking
pub struct WalkDir {
    root: PathBuf,
    options: FilesOptions,
}

impl WalkDir {
    /// Create a new WalkDir instance
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            options: FilesOptions::default(),
        }
    }

    /// Create a new WalkDir instance with custom options
    pub fn new_with_options<P: AsRef<Path>>(root: P, options: FilesOptions) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            options,
        }
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
            .follow_links(false)
            .ignore(false)
            .git_global(false)
            .threads(rayon::current_num_threads());

        builder.build_parallel().run(|| {
            let results_clone = Arc::clone(&results);
            let include_patterns = self.options.include.clone().unwrap_or_default();
            let exclude_patterns = self.options.exclude.clone().unwrap_or_default();
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
                                    if glob_match(pattern, &relative_path) {
                                        should_include = true;
                                        break;
                                    }
                                }

                                if should_include {
                                    // Check if file should be excluded
                                    let mut should_exclude = false;
                                    for pattern in &exclude_patterns {
                                        if glob_match(pattern, &relative_path) {
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
        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["*.rs".to_string()]),
                exclude: None,
            },
        );
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

                                if glob_match("*.rs", filename) {
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
        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["*.toml".to_string()]),
                exclude: None,
            },
        );
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

                                if glob_match("*.toml", filename) {
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
    fn test_walkdir_includes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["*.rs".to_string(), "*.toml".to_string()]),
                exclude: None,
            },
        );
        assert_eq!(
            walker.options.include,
            Some(vec!["*.rs".to_string(), "*.toml".to_string()])
        );
    }

    #[test]
    fn test_walkdir_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: None,
                exclude: Some(vec!["target/**".to_string(), "node_modules/**".to_string()]),
            },
        );
        assert_eq!(
            walker.options.exclude,
            Some(vec!["target/**".to_string(), "node_modules/**".to_string()])
        );
    }

    #[test]
    fn test_walkdir_includes_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["*.rs".to_string()]),
                exclude: Some(vec!["target/**".to_string()]),
            },
        );
        assert_eq!(walker.options.include, Some(vec!["*.rs".to_string()]));
        assert_eq!(walker.options.exclude, Some(vec!["target/**".to_string()]));
    }

    #[test]
    fn test_invalid_pattern() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["invalid[pattern".to_string()]),
                exclude: None,
            },
        );
        // Invalid patterns will cause panic at runtime, not compile time
        assert_eq!(
            walker.options.include,
            Some(vec!["invalid[pattern".to_string()])
        );
    }

    #[tokio::test]
    async fn test_walkdir_walk() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["*.rs".to_string()]),
                exclude: None,
            },
        );
        let result = walker.walk().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_walkdir_walk_with_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["*.toml".to_string()]),
                exclude: Some(vec!["target/**".to_string()]),
            },
        );
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
