use fast_glob::glob_match;
use futures::{stream, StreamExt};
use ignore::{DirEntry, WalkBuilder};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlobError {
    #[error("Invalid glob pattern: '{pattern}'")]
    InvalidPattern { pattern: String },

    #[error("Empty pattern provided")]
    EmptyPattern,

    #[error("IO error while walking directory '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Search root path does not exist: '{path}'")]
    RootPathNotFound { path: PathBuf },

    #[error("Search root path is not a directory: '{path}'")]
    RootPathNotDirectory { path: PathBuf },

    #[error("Failed to acquire thread synchronization lock")]
    LockError,

    #[error("Maximum file size exceeded: {size} bytes (limit: {limit} bytes)")]
    FileSizeExceeded { size: u64, limit: u64 },

    #[error("Maximum search depth exceeded: {depth} (limit: {limit})")]
    MaxDepthExceeded { depth: usize, limit: usize },
}

impl GlobError {
    pub fn invalid_pattern(pattern: impl Into<String>) -> Self {
        Self::InvalidPattern {
            pattern: pattern.into(),
        }
    }

    pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::IoError {
            path: path.into(),
            source,
        }
    }
}

type GlobResult<T> = Result<T, GlobError>;

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

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub pattern: String,
}

#[derive(Debug, Clone)]
pub struct DirectoryProfile {
    pub path: PathBuf,
    pub duration: Duration,
    pub file_count: usize,
    pub subdirectory_count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub struct SearchProfile {
    pub total_duration: Duration,
    pub directory_profiles: Vec<DirectoryProfile>,
    pub total_files_found: usize,
    pub total_directories_scanned: usize,
    pub slowest_directories: Vec<DirectoryProfile>,
}

impl SearchProfile {
    pub fn new() -> Self {
        Self {
            total_duration: Duration::new(0, 0),
            directory_profiles: Vec::new(),
            total_files_found: 0,
            total_directories_scanned: 0,
            slowest_directories: Vec::new(),
        }
    }

    pub fn add_directory_profile(&mut self, profile: DirectoryProfile) {
        self.total_directories_scanned += 1;
        self.total_files_found += profile.file_count;
        self.directory_profiles.push(profile);
    }

    pub fn finalize(&mut self) {
        // Sort by duration and keep top 10 slowest directories
        self.directory_profiles
            .sort_by(|a, b| b.duration.cmp(&a.duration));
        self.slowest_directories = self.directory_profiles.iter().take(10).cloned().collect();
    }

    pub fn print_report(&self) {
        println!("=== Directory Search Profile Report ===");
        println!("Total duration: {:?}", self.total_duration);
        println!(
            "Total directories scanned: {}",
            self.total_directories_scanned
        );
        println!("Total files found: {}", self.total_files_found);
        println!("\n=== Top 10 Slowest Directories ===");

        for (i, profile) in self.slowest_directories.iter().enumerate() {
            println!(
                "{}. {:?} - {:?} ({} files, {} subdirs, {} bytes)",
                i + 1,
                profile.path,
                profile.duration,
                profile.file_count,
                profile.subdirectory_count,
                profile.total_size
            );
        }
    }
}

/// Validate a glob pattern
pub fn validate_pattern(pattern: &str) -> GlobResult<()> {
    if pattern.is_empty() {
        return Err(GlobError::EmptyPattern);
    }

    // Validate pattern by trying to match empty string (fast-glob will panic on invalid patterns)
    let _ = glob_match(pattern, "");
    Ok(())
}

/// Check if a path matches a glob pattern
pub fn matches_pattern(pattern: &str, path: &str) -> bool {
    glob_match(pattern, path)
}

/// Find files matching a single pattern
pub fn find_files<P: AsRef<Path>>(
    root: P,
    pattern: &str,
    options: Option<SearchOptions>,
) -> GlobResult<Vec<PathBuf>> {
    validate_pattern(pattern)?;

    let root_path = root.as_ref();
    if !root_path.exists() {
        return Err(GlobError::RootPathNotFound {
            path: root_path.to_path_buf(),
        });
    }

    if !root_path.is_dir() {
        return Err(GlobError::RootPathNotDirectory {
            path: root_path.to_path_buf(),
        });
    }

    let search_options = options.unwrap_or_default();
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut builder = WalkBuilder::new(root_path);
    builder
        .follow_links(search_options.follow_links)
        .hidden(search_options.hidden)
        .ignore(!search_options.ignore_files)
        .git_ignore(search_options.git_ignore)
        .threads(search_options.threads);

    if let Some(max_depth) = search_options.max_depth {
        builder.max_depth(Some(max_depth));
    }

    if let Some(max_filesize) = search_options.max_filesize {
        builder.max_filesize(Some(max_filesize));
    }

    let walker = builder.build_parallel();

    walker.run(|| {
        let results_clone = Arc::clone(&results);
        let pattern = pattern.to_string();

        Box::new(move |entry_result| {
            match entry_result {
                Ok(entry) => {
                    if let Some(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                            if glob_match(&pattern, filename) {
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
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    Ok(results)
}

/// Find files matching multiple patterns
pub fn find_files_multi<P: AsRef<Path>>(
    root: P,
    patterns: &[&str],
    options: Option<SearchOptions>,
) -> GlobResult<Vec<SearchResult>> {
    for pattern in patterns {
        validate_pattern(pattern)?;
    }

    let root_path = root.as_ref();
    if !root_path.exists() {
        return Err(GlobError::RootPathNotFound {
            path: root_path.to_path_buf(),
        });
    }

    if !root_path.is_dir() {
        return Err(GlobError::RootPathNotDirectory {
            path: root_path.to_path_buf(),
        });
    }

    let search_options = options.unwrap_or_default();
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut builder = WalkBuilder::new(root_path);
    builder
        .follow_links(search_options.follow_links)
        .hidden(search_options.hidden)
        .ignore(!search_options.ignore_files)
        .git_ignore(search_options.git_ignore)
        .threads(search_options.threads);

    if let Some(max_depth) = search_options.max_depth {
        builder.max_depth(Some(max_depth));
    }

    if let Some(max_filesize) = search_options.max_filesize {
        builder.max_filesize(Some(max_filesize));
    }

    let walker = builder.build_parallel();

    walker.run(|| {
        let results_clone = Arc::clone(&results);
        let patterns = patterns.to_vec();

        Box::new(move |entry_result| {
            match entry_result {
                Ok(entry) => {
                    if let Some(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                            for pattern in &patterns {
                                if glob_match(pattern, filename) {
                                    if let Ok(mut results_guard) = results_clone.lock() {
                                        results_guard.push(SearchResult {
                                            path: path.to_path_buf(),
                                            pattern: pattern.to_string(),
                                        });
                                    }
                                    break; // First match wins
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
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    Ok(results)
}

/// Find files with include/exclude patterns
pub fn find_files_with_exclude<P: AsRef<Path>>(
    root: P,
    include_patterns: &[&str],
    exclude_patterns: &[&str],
    options: Option<SearchOptions>,
) -> GlobResult<Vec<PathBuf>> {
    for pattern in include_patterns {
        validate_pattern(pattern)?;
    }
    for pattern in exclude_patterns {
        validate_pattern(pattern)?;
    }

    let root_path = root.as_ref();
    if !root_path.exists() {
        return Err(GlobError::RootPathNotFound {
            path: root_path.to_path_buf(),
        });
    }

    if !root_path.is_dir() {
        return Err(GlobError::RootPathNotDirectory {
            path: root_path.to_path_buf(),
        });
    }

    let search_options = options.unwrap_or_default();
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut builder = WalkBuilder::new(root_path);
    builder
        .follow_links(search_options.follow_links)
        .hidden(search_options.hidden)
        .ignore(!search_options.ignore_files)
        .git_ignore(search_options.git_ignore)
        .threads(search_options.threads);

    if let Some(max_depth) = search_options.max_depth {
        builder.max_depth(Some(max_depth));
    }

    if let Some(max_filesize) = search_options.max_filesize {
        builder.max_filesize(Some(max_filesize));
    }

    let walker = builder.build_parallel();

    walker.run(|| {
        let results_clone = Arc::clone(&results);
        let include_patterns = include_patterns.to_vec();
        let exclude_patterns = exclude_patterns.to_vec();
        let root_path = root_path.to_path_buf();

        Box::new(move |entry_result| {
            match entry_result {
                Ok(entry) => {
                    if let Some(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            let relative_path = if let Ok(rel_path) = path.strip_prefix(&root_path)
                            {
                                rel_path.to_string_lossy()
                            } else {
                                path.to_string_lossy()
                            };

                            // Check if file matches any include pattern
                            let mut should_include = false;
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
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    Ok(results)
}

/// Find files with profiling information
pub fn find_files_with_profile<P: AsRef<Path>>(
    root: P,
    pattern: &str,
    options: Option<SearchOptions>,
) -> GlobResult<(Vec<PathBuf>, SearchProfile)> {
    validate_pattern(pattern)?;

    let root_path = root.as_ref();
    if !root_path.exists() {
        return Err(GlobError::RootPathNotFound {
            path: root_path.to_path_buf(),
        });
    }

    if !root_path.is_dir() {
        return Err(GlobError::RootPathNotDirectory {
            path: root_path.to_path_buf(),
        });
    }

    let start_time = Instant::now();
    let mut profile = SearchProfile::new();
    let mut results = Vec::new();

    // Use fast walker for profiling
    let mut walker = FastWalker::new(pattern.to_string());
    walker.walk_with_profile(root_path, &mut results, &mut profile)?;

    profile.total_duration = start_time.elapsed();
    profile.finalize();

    Ok((results, profile))
}

// Fast walker for profiling
struct FastWalker {
    pattern: String,
}

impl FastWalker {
    fn new(pattern: String) -> Self {
        Self { pattern }
    }

    fn walk_with_profile(
        &self,
        dir: &Path,
        results: &mut Vec<PathBuf>,
        profile: &mut SearchProfile,
    ) -> GlobResult<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let dir_start = Instant::now();
        let mut file_count = 0;
        let mut subdirectory_count = 0;
        let mut total_size = 0u64;

        let entries = fs::read_dir(dir).map_err(|e| GlobError::io_error(dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| GlobError::io_error(dir, e))?;
            let path = entry.path();

            if path.is_dir() {
                subdirectory_count += 1;
                self.walk_with_profile(&path, results, profile)?;
            } else {
                file_count += 1;

                // Get file size
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }

                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if glob_match(&self.pattern, filename) {
                        results.push(path.clone());
                    }
                }
            }
        }

        let dir_duration = dir_start.elapsed();

        // Record all directories with files or subdirectories
        if file_count > 0 || subdirectory_count > 0 {
            profile.add_directory_profile(DirectoryProfile {
                path: dir.to_path_buf(),
                duration: dir_duration,
                file_count,
                subdirectory_count,
                total_size,
            });
        }

        Ok(())
    }
}

// Convenience functions
pub fn find_rust_files<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    find_files(root, "*.rs", None)
}

pub fn find_toml_files<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    find_files(root, "*.toml", None)
}

pub fn find_config_files<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    find_files_multi(
        root,
        &[
            "*.toml", "*.json", "*.yaml", "*.yml", "*.ini", "*.conf", "*.config",
        ],
        None,
    )
    .map(|results| results.into_iter().map(|r| r.path).collect())
}

pub fn find_toml_files_excluding<P: AsRef<Path>>(
    root: P,
    exclude_patterns: &[&str],
) -> GlobResult<Vec<PathBuf>> {
    find_files_with_exclude(root, &["**/*.toml"], exclude_patterns, None)
}

// Async versions
pub async fn find_files_async<P: AsRef<Path>>(
    root: P,
    pattern: &str,
    options: Option<SearchOptions>,
) -> GlobResult<Vec<PathBuf>> {
    validate_pattern(pattern)?;

    let root_path = root.as_ref();
    if !root_path.exists() {
        return Err(GlobError::RootPathNotFound {
            path: root_path.to_path_buf(),
        });
    }

    if !root_path.is_dir() {
        return Err(GlobError::RootPathNotDirectory {
            path: root_path.to_path_buf(),
        });
    }

    let search_options = options.unwrap_or_default();

    let mut builder = WalkBuilder::new(root_path);
    builder
        .follow_links(search_options.follow_links)
        .hidden(search_options.hidden)
        .ignore(!search_options.ignore_files)
        .git_ignore(search_options.git_ignore);

    if let Some(max_depth) = search_options.max_depth {
        builder.max_depth(Some(max_depth));
    }

    if let Some(max_filesize) = search_options.max_filesize {
        builder.max_filesize(Some(max_filesize));
    }

    let walker = builder.build();
    let mut results = Vec::new();

    let entries: Vec<_> = walker.collect();

    let chunks = entries.chunks(100);
    let chunk_streams = chunks.map(|chunk| {
        let chunk_results: Vec<_> = chunk
            .iter()
            .filter_map(|entry_result| match entry_result {
                Ok(entry) => {
                    if let Some(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                            if glob_match(pattern, filename) {
                                Some(path.to_path_buf())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .collect();
        stream::iter(chunk_results)
    });

    let mut stream = stream::iter(chunk_streams).flatten();

    while let Some(result) = stream.next().await {
        results.push(result);
    }

    Ok(results)
}

pub async fn find_rust_files_async<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    find_files_async(root, "*.rs", None).await
}

pub async fn find_toml_files_async<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    find_files_async(root, "*.toml", None).await
}

pub async fn find_config_files_async<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let patterns = &[
        "*.toml", "*.json", "*.yaml", "*.yml", "*.ini", "*.conf", "*.config",
    ];
    let mut all_results = Vec::new();

    for pattern in patterns {
        let results = find_files_async(root.as_ref(), pattern, None).await?;
        all_results.extend(results);
    }

    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pattern() {
        assert!(validate_pattern("*.rs").is_ok());
        assert!(validate_pattern("test.*").is_ok());
        assert!(matches!(validate_pattern(""), Err(GlobError::EmptyPattern)));
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("*.rs", "main.rs"));
        assert!(matches_pattern("test.*", "test.txt"));
        assert!(matches_pattern("h*o", "hello"));
        assert!(!matches_pattern("*.rs", "main.txt"));
    }

    #[test]
    fn test_find_files() {
        let current_dir = std::env::current_dir().unwrap();
        let result = find_files(&current_dir, "*.rs", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_files_multi() {
        let current_dir = std::env::current_dir().unwrap();
        let result = find_files_multi(&current_dir, &["*.rs", "*.toml"], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_files_with_exclude() {
        let current_dir = std::env::current_dir().unwrap();
        let result = find_files_with_exclude(&current_dir, &["*.toml"], &["target/**"], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convenience_functions() {
        let current_dir = std::env::current_dir().unwrap();

        let _ = find_rust_files(&current_dir);
        let _ = find_toml_files(&current_dir);
        let _ = find_config_files(&current_dir);
        let _ = find_toml_files_excluding(&current_dir, &["target/**"]);
    }

    #[test]
    fn test_find_files_with_profile() {
        let current_dir = std::env::current_dir().unwrap();
        let (results, profile) = find_files_with_profile(&current_dir, "*.toml", None).unwrap();

        assert!(results.len() > 0);
        assert!(profile.total_directories_scanned > 0);
        assert!(profile.total_duration.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_find_files_async() {
        let current_dir = std::env::current_dir().unwrap();
        let result = find_files_async(&current_dir, "*.rs", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_convenience_functions() {
        let current_dir = std::env::current_dir().unwrap();

        let _ = find_rust_files_async(&current_dir).await;
        let _ = find_toml_files_async(&current_dir).await;
        let _ = find_config_files_async(&current_dir).await;
    }

    #[test]
    fn test_error_handling() {
        let result = find_files("/nonexistent/path", "*.rs", None);
        assert!(matches!(result, Err(GlobError::RootPathNotFound { .. })));
    }
}
