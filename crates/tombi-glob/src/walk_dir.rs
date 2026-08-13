#[cfg(not(target_family = "wasm"))]
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
#[cfg(not(target_family = "wasm"))]
use std::sync::{Arc, Mutex};

use tombi_config::FilesOptions;

#[cfg(test)]
use fast_glob::glob_match;

const VCS_METADATA_DIR_NAMES: &[&str] = &[".git", ".hg", ".svn", ".bzr", ".jj", ".sl"];

fn is_vcs_metadata_dir(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        VCS_METADATA_DIR_NAMES
            .iter()
            .any(|candidate| name == *candidate)
    })
}

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

    /// Walk the directory asynchronously and return matching file paths.
    #[cfg(not(target_family = "wasm"))]
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
        if is_vcs_metadata_dir(root_path) {
            return Ok(Vec::new());
        }

        let results = Arc::new(Mutex::new(Vec::new()));

        let mut builder = WalkBuilder::new(root_path);
        let respect_ignore_files = self.options.respect_ignore_files.value();
        builder
            .hidden(false)
            .follow_links(false)
            .parents(respect_ignore_files)
            .ignore(respect_ignore_files)
            .git_ignore(respect_ignore_files)
            .git_exclude(respect_ignore_files)
            .git_global(false)
            .filter_entry(|entry| {
                !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
                    || !is_vcs_metadata_dir(entry.path())
            })
            .threads(rayon::current_num_threads());

        builder.build_parallel().run(|| {
            let results_clone = Arc::clone(&results);
            let include_patterns = self.options.include.clone().unwrap_or_default();
            let exclude_patterns = self.options.exclude.clone().unwrap_or_default();
            let root_path = root_path.to_path_buf();

            Box::new(move |entry_result| {
                match entry_result {
                    Ok(entry) => {
                        if let Some(file_type) = entry.file_type()
                            && file_type.is_file()
                        {
                            let path = entry.path();
                            let path_for_patterns =
                                crate::pattern::path_for_patterns(path, &root_path);

                            // Check if file matches any include pattern
                            let should_include = include_patterns.is_empty()
                                || crate::pattern::matches_any_pattern(
                                    path_for_patterns.as_ref(),
                                    &include_patterns,
                                );

                            if should_include {
                                // Check if file should be excluded
                                let should_exclude = crate::pattern::matches_any_pattern(
                                    path_for_patterns.as_ref(),
                                    &exclude_patterns,
                                );

                                if !should_exclude
                                    && let Ok(mut results_guard) = results_clone.lock()
                                {
                                    results_guard.push(path.to_path_buf());
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

    /// Walk the browser-backed virtual filesystem and return matching file paths.
    #[cfg(target_family = "wasm")]
    pub async fn walk(self) -> Result<Vec<PathBuf>, crate::Error> {
        let root_path = self.root;
        if !tombi_fs::is_file(&root_path) && !tombi_fs::is_dir(&root_path) {
            return Err(crate::Error::RootPathNotFound { path: root_path });
        }
        if !tombi_fs::is_dir(&root_path) {
            return Err(crate::Error::RootPathNotDirectory { path: root_path });
        }
        if is_vcs_metadata_dir(&root_path) {
            return Ok(Vec::new());
        }

        let include_patterns = self.options.include.unwrap_or_default();
        let exclude_patterns = self.options.exclude.unwrap_or_default();
        let mut directories = vec![root_path.clone()];
        let mut results = Vec::new();

        while let Some(directory) = directories.pop() {
            let entries =
                tombi_fs::read_dir(&directory).map_err(|source| crate::Error::IoError {
                    path: directory.clone(),
                    source,
                })?;
            for entry in entries {
                let path = entry.path().to_path_buf();
                if entry.is_dir() {
                    if !is_vcs_metadata_dir(&path) {
                        directories.push(path);
                    }
                    continue;
                }

                let path_for_patterns = crate::pattern::path_for_patterns(&path, &root_path);
                let should_include = include_patterns.is_empty()
                    || crate::pattern::matches_any_pattern(
                        path_for_patterns.as_ref(),
                        &include_patterns,
                    );
                let should_exclude = crate::pattern::matches_any_pattern(
                    path_for_patterns.as_ref(),
                    &exclude_patterns,
                );
                if should_include && !should_exclude {
                    results.push(path);
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    fn make_git_root(root: &Path) {
        fs::create_dir_all(root.join(".git")).unwrap();
    }

    // Convenience functions using the new API
    fn find_rust_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, crate::Error> {
        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["*.rs".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
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
                        if let Some(file_type) = entry.file_type()
                            && file_type.is_file()
                        {
                            let path = entry.path();
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                            if glob_match("*.rs", filename)
                                && let Ok(mut results_guard) = results_clone.lock()
                            {
                                results_guard.push(path.to_path_buf());
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
                include: Some(vec!["*.toml".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
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
                        if let Some(file_type) = entry.file_type()
                            && file_type.is_file()
                        {
                            let path = entry.path();
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                            if glob_match("*.toml", filename)
                                && let Ok(mut results_guard) = results_clone.lock()
                            {
                                results_guard.push(path.to_path_buf());
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
                include: Some(vec!["*.rs".into(), "*.toml".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
            },
        );
        assert_eq!(
            walker.options.include,
            Some(vec!["*.rs".into(), "*.toml".into()])
        );
    }

    #[test]
    fn test_walkdir_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: None,
                exclude: Some(vec!["target/**".into(), "node_modules/**".into()]),
                respect_ignore_files: true.into(),
            },
        );
        assert_eq!(
            walker.options.exclude,
            Some(vec!["target/**".into(), "node_modules/**".into()])
        );
    }

    #[test]
    fn test_walkdir_includes_excludes() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["*.rs".into()]),
                exclude: Some(vec!["target/**".into()]),
                respect_ignore_files: true.into(),
            },
        );
        assert_eq!(walker.options.include, Some(vec!["*.rs".into()]));
        assert_eq!(walker.options.exclude, Some(vec!["target/**".into()]));
    }

    #[test]
    fn test_invalid_pattern() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["invalid[pattern".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
            },
        );
        // Invalid patterns will cause panic at runtime, not compile time
        assert_eq!(walker.options.include, Some(vec!["invalid[pattern".into()]));
    }

    #[tokio::test]
    async fn test_walkdir_walk() {
        let current_dir = std::env::current_dir().unwrap();
        let walker = WalkDir::new_with_options(
            &current_dir,
            FilesOptions {
                include: Some(vec!["*.rs".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
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
                include: Some(vec!["*.toml".into()]),
                exclude: Some(vec!["target/**".into()]),
                respect_ignore_files: true.into(),
            },
        );
        let result = walker.walk().await;
        assert!(result.is_ok());
    }

    macro_rules! test_walkdir_excludes_vcs_metadata_dirs {
        ($name:ident, $respect_ignore_files:literal) => {
            #[tokio::test]
            async fn $name() {
                let tempdir = tempdir().unwrap();
                let root = tempdir.path();
                write_file(&root.join("visible.toml"), "key = 1\n");
                write_file(&root.join(".hidden.toml"), "key = 2\n");
                write_file(&root.join(".claude/settings.toml"), "key = 3\n");
                for name in VCS_METADATA_DIR_NAMES {
                    write_file(&root.join(name).join("metadata.toml"), "not TOML\n");
                }

                let walker = WalkDir::new_with_options(
                    root,
                    FilesOptions {
                        include: Some(vec!["**/*.toml".into()]),
                        exclude: None,
                        respect_ignore_files: $respect_ignore_files.into(),
                    },
                );

                let result = walker.walk().await.unwrap();
                assert!(result.iter().any(|path| path.ends_with("visible.toml")));
                assert!(result.iter().any(|path| path.ends_with(".hidden.toml")));
                assert!(
                    result
                        .iter()
                        .any(|path| path.ends_with(".claude/settings.toml"))
                );
                for name in VCS_METADATA_DIR_NAMES {
                    assert!(
                        !result
                            .iter()
                            .any(|path| path.ends_with(Path::new(name).join("metadata.toml"))),
                        "{name} must not be traversed"
                    );
                }
            }
        };
    }

    test_walkdir_excludes_vcs_metadata_dirs!(
        test_walkdir_excludes_vcs_metadata_dirs_when_respecting_ignore_files,
        true
    );
    test_walkdir_excludes_vcs_metadata_dirs!(
        test_walkdir_excludes_vcs_metadata_dirs_when_ignoring_ignore_files,
        false
    );

    #[tokio::test]
    async fn test_walkdir_does_not_traverse_vcs_metadata_dir_as_root() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path().join(".git");
        write_file(&root.join("metadata.toml"), "not TOML\n");

        let result = WalkDir::new(root).walk().await.unwrap();
        assert!(result.is_empty());
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
        std::assert_matches!(result, Err(crate::Error::RootPathNotFound { .. }));
    }

    #[tokio::test]
    async fn test_walkdir_respects_gitignore_when_enabled() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path();
        make_git_root(root);
        write_file(&root.join(".gitignore"), "ignored.toml\n");
        write_file(&root.join("included.toml"), "key = 1\n");
        write_file(&root.join("ignored.toml"), "key = 2\n");

        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["**/*.toml".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
            },
        );

        let result = walker.walk().await.unwrap();
        assert!(result.iter().any(|path| path.ends_with("included.toml")));
        assert!(!result.iter().any(|path| path.ends_with("ignored.toml")));
    }

    #[tokio::test]
    async fn test_walkdir_ignores_gitignore_when_disabled() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path();
        make_git_root(root);
        write_file(&root.join(".gitignore"), "ignored.toml\n");
        write_file(&root.join("included.toml"), "key = 1\n");
        write_file(&root.join("ignored.toml"), "key = 2\n");

        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["**/*.toml".into()]),
                exclude: None,
                respect_ignore_files: false.into(),
            },
        );

        let result = walker.walk().await.unwrap();
        assert!(result.iter().any(|path| path.ends_with("included.toml")));
        assert!(result.iter().any(|path| path.ends_with("ignored.toml")));
    }

    #[tokio::test]
    async fn test_walkdir_respects_git_exclude_when_enabled() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path();
        make_git_root(root);
        write_file(&root.join(".git/info/exclude"), "ignored.toml\n");
        write_file(&root.join("included.toml"), "key = 1\n");
        write_file(&root.join("ignored.toml"), "key = 2\n");

        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["**/*.toml".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
            },
        );

        let result = walker.walk().await.unwrap();
        assert!(result.iter().any(|path| path.ends_with("included.toml")));
        assert!(!result.iter().any(|path| path.ends_with("ignored.toml")));
    }

    #[tokio::test]
    async fn test_walkdir_ignores_git_exclude_when_disabled() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path();
        make_git_root(root);
        write_file(&root.join(".git/info/exclude"), "ignored.toml\n");
        write_file(&root.join("included.toml"), "key = 1\n");
        write_file(&root.join("ignored.toml"), "key = 2\n");

        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["**/*.toml".into()]),
                exclude: None,
                respect_ignore_files: false.into(),
            },
        );

        let result = walker.walk().await.unwrap();
        assert!(result.iter().any(|path| path.ends_with("included.toml")));
        assert!(result.iter().any(|path| path.ends_with("ignored.toml")));
    }

    #[tokio::test]
    async fn test_walkdir_respects_dot_ignore_when_enabled() {
        let tempdir = tempdir().unwrap();
        let root = tempdir.path();
        write_file(&root.join(".ignore"), "ignored.toml\n");
        write_file(&root.join("included.toml"), "key = 1\n");
        write_file(&root.join("ignored.toml"), "key = 2\n");

        let walker = WalkDir::new_with_options(
            root,
            FilesOptions {
                include: Some(vec!["**/*.toml".into()]),
                exclude: None,
                respect_ignore_files: true.into(),
            },
        );

        let result = walker.walk().await.unwrap();
        assert!(result.iter().any(|path| path.ends_with("included.toml")));
        assert!(!result.iter().any(|path| path.ends_with("ignored.toml")));
    }
}
