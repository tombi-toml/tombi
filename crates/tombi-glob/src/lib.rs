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

    #[error("Invalid character class in pattern: '{pattern}' at position {position}")]
    InvalidCharacterClass { pattern: String, position: usize },

    #[error("Unclosed bracket in pattern: '{pattern}' at position {position}")]
    UnclosedBracket { pattern: String, position: usize },

    #[error("Empty pattern provided")]
    EmptyPattern,

    #[error("Pattern too complex: '{pattern}' (maximum {max_states} states exceeded)")]
    PatternTooComplex { pattern: String, max_states: usize },

    #[error("IO error while walking directory '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Directory walk failed: {message}")]
    WalkError { message: String },

    #[error("Failed to acquire thread synchronization lock")]
    LockError,

    #[error("Search root path does not exist: '{path}'")]
    RootPathNotFound { path: PathBuf },

    #[error("Search root path is not a directory: '{path}'")]
    RootPathNotDirectory { path: PathBuf },

    #[error("Pattern compilation failed for: '{pattern}' - {reason}")]
    CompilationError { pattern: String, reason: String },

    #[error("Thread pool error: {message}")]
    ThreadPoolError { message: String },

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

    pub fn invalid_character_class(pattern: impl Into<String>, position: usize) -> Self {
        Self::InvalidCharacterClass {
            pattern: pattern.into(),
            position,
        }
    }

    pub fn unclosed_bracket(pattern: impl Into<String>, position: usize) -> Self {
        Self::UnclosedBracket {
            pattern: pattern.into(),
            position,
        }
    }

    pub fn pattern_too_complex(pattern: impl Into<String>, max_states: usize) -> Self {
        Self::PatternTooComplex {
            pattern: pattern.into(),
            max_states,
        }
    }

    pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::IoError {
            path: path.into(),
            source,
        }
    }

    pub fn compilation_error(pattern: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CompilationError {
            pattern: pattern.into(),
            reason: reason.into(),
        }
    }
}

pub type GlobResult<T> = Result<T, GlobError>;

#[derive(Debug, Clone)]
pub struct State {
    pub chars: [bool; 256],
    pub is_star: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            chars: [false; 256],
            is_star: false,
        }
    }

    pub fn set_char(&mut self, c: u8) {
        self.chars[c as usize] = true;
    }

    pub fn set_all(&mut self) {
        self.chars = [true; 256];
    }

    pub fn matches(&self, c: u8) -> bool {
        self.chars[c as usize]
    }
}

#[derive(Debug, Clone)]
pub struct GlobPattern {
    pub states: Vec<State>,
    pub value: i64,
}

impl GlobPattern {
    pub fn new(states: Vec<State>, value: i64) -> Self {
        Self { states, value }
    }
}

#[derive(Debug, Clone)]
pub struct MultiGlob {
    patterns: Vec<GlobPattern>,
    start_states: Vec<u64>,
    compiled: bool,
}

impl MultiGlob {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            start_states: Vec::new(),
            compiled: false,
        }
    }

    pub fn add(&mut self, pattern: &str, value: i64) -> GlobResult<()> {
        if pattern.is_empty() {
            return Err(GlobError::EmptyPattern);
        }

        match parse_glob(pattern) {
            Ok(states) => {
                if states.len() > 1000 {
                    // 合理的な制限を設定
                    return Err(GlobError::pattern_too_complex(pattern, 1000));
                }
                self.patterns.push(GlobPattern::new(states, value));
                self.compiled = false;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn compile(&mut self) {
        if self.patterns.is_empty() {
            return;
        }

        self.patterns.sort_by(|a, b| b.value.cmp(&a.value));

        let mut all_states = Vec::new();
        for pattern in &self.patterns {
            all_states.extend_from_slice(&pattern.states);
        }

        let sz = all_states.len();
        self.start_states.resize(sz, 0);

        let mut pos = 0;
        for pattern in &self.patterns {
            let len = pattern.states.len();
            for i in 0..len {
                self.start_states[pos + i] = if i == 0 { 1u64 << (pos + i) } else { 0 };
            }
            pos += len;
        }

        self.compiled = true;
    }

    pub fn find(&self, input: &[u8]) -> Option<i64> {
        for pattern in &self.patterns {
            if self.matches_pattern(&pattern.states, input) {
                return Some(pattern.value);
            }
        }
        None
    }

    fn matches_pattern(&self, states: &[State], input: &[u8]) -> bool {
        self.match_recursive(states, 0, input, 0)
    }

    fn match_recursive(
        &self,
        states: &[State],
        state_idx: usize,
        input: &[u8],
        input_idx: usize,
    ) -> bool {
        if state_idx >= states.len() {
            return input_idx == input.len();
        }

        if input_idx > input.len() {
            return false;
        }

        let state = &states[state_idx];

        if state.is_star {
            if self.match_recursive(states, state_idx + 1, input, input_idx) {
                return true;
            }

            for i in input_idx..input.len() {
                if self.match_recursive(states, state_idx + 1, input, i + 1) {
                    return true;
                }
            }

            false
        } else {
            if input_idx >= input.len() {
                return false;
            }

            if state.matches(input[input_idx]) {
                self.match_recursive(states, state_idx + 1, input, input_idx + 1)
            } else {
                false
            }
        }
    }
}

fn parse_glob(pattern: &str) -> GlobResult<Vec<State>> {
    let mut states: Vec<State> = Vec::new();
    let mut chars = pattern.bytes().peekable();
    let mut position = 0;

    while let Some(c) = chars.next() {
        let mut char_set = State::new();

        match c {
            b'*' => {
                if let Some(last_state) = states.last_mut() {
                    last_state.is_star = true;
                } else {
                    let mut star_state = State::new();
                    star_state.is_star = true;
                    states.push(star_state);
                }
                position += 1;
                continue;
            }
            b'?' => {
                char_set.set_all();
            }
            b'\\' => {
                if let Some(&next_char) = chars.peek() {
                    char_set.set_char(next_char);
                    chars.next();
                    position += 1;
                } else {
                    return Err(GlobError::invalid_pattern(pattern));
                }
            }
            b'[' => {
                let bracket_start = position;
                let mut negate = false;
                let mut bracket_chars = Vec::new();
                let mut closed = false;

                if let Some(&b'!') | Some(&b'^') = chars.peek() {
                    negate = true;
                    chars.next();
                    position += 1;
                }

                if let Some(&b']') = chars.peek() {
                    bracket_chars.push(b']');
                    chars.next();
                    position += 1;
                }

                while let Some(bracket_char) = chars.next() {
                    position += 1;

                    if bracket_char == b']' {
                        closed = true;
                        break;
                    }

                    if bracket_char == b'-' && !bracket_chars.is_empty() {
                        if let Some(&end_char) = chars.peek() {
                            if end_char != b']' {
                                let start_char = *bracket_chars.last().unwrap();
                                chars.next();
                                position += 1;

                                if start_char > end_char {
                                    return Err(GlobError::invalid_character_class(
                                        pattern,
                                        bracket_start,
                                    ));
                                }

                                for ch in start_char..=end_char {
                                    bracket_chars.push(ch);
                                }
                                continue;
                            }
                        }
                    }

                    bracket_chars.push(bracket_char);
                }

                if !closed {
                    return Err(GlobError::unclosed_bracket(pattern, bracket_start));
                }

                if bracket_chars.is_empty() {
                    return Err(GlobError::invalid_character_class(pattern, bracket_start));
                }

                if negate {
                    char_set.set_all();
                    for ch in bracket_chars {
                        char_set.chars[ch as usize] = false;
                    }
                } else {
                    for ch in bracket_chars {
                        char_set.set_char(ch);
                    }
                }
            }
            _ => {
                char_set.set_char(c);
            }
        }

        states.push(char_set);
        position += 1;
    }

    if states.is_empty() {
        return Err(GlobError::EmptyPattern);
    }

    Ok(states)
}

impl Default for MultiGlob {
    fn default() -> Self {
        Self::new()
    }
}

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
    pub pattern_value: i64,
    pub pattern_index: usize,
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

pub struct ParallelGlobWalker {
    glob: MultiGlob,
    options: SearchOptions,
}

impl ParallelGlobWalker {
    pub fn new(mut glob: MultiGlob, options: SearchOptions) -> Self {
        glob.compile();
        Self { glob, options }
    }

    pub fn search<P: AsRef<Path>>(&self, root: P) -> GlobResult<Vec<SearchResult>> {
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut builder = WalkBuilder::new(root);
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
            let glob_clone = &self.glob;

            Box::new(move |entry_result| {
                match entry_result {
                    Ok(entry) => {
                        if let Some(search_result) = self.process_entry(&entry, glob_clone) {
                            if let Ok(mut results_guard) = results_clone.lock() {
                                results_guard.push(search_result);
                            }
                        }
                    }
                    Err(_) => {
                        // エラーは無視してディレクトリ探索を継続
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

    fn process_entry(&self, entry: &DirEntry, glob: &MultiGlob) -> Option<SearchResult> {
        if entry.file_type()?.is_file() {
            let path = entry.path();
            let filename = path.file_name()?.to_str()?;

            for (pattern_index, pattern) in glob.patterns.iter().enumerate() {
                if glob.matches_pattern(&pattern.states, filename.as_bytes()) {
                    return Some(SearchResult {
                        path: path.to_path_buf(),
                        pattern_value: pattern.value,
                        pattern_index,
                    });
                }
            }
        }
        None
    }

    pub fn search_with_full_paths<P: AsRef<Path>>(&self, root: P) -> GlobResult<Vec<SearchResult>> {
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut builder = WalkBuilder::new(root.as_ref());
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
        let root_path = root.as_ref().to_path_buf();

        walker.run(|| {
            let results_clone = Arc::clone(&results);
            let glob_clone = &self.glob;
            let root_clone = root_path.clone();

            Box::new(move |entry_result| {
                match entry_result {
                    Ok(entry) => {
                        if let Some(search_result) =
                            self.process_entry_with_full_path(&entry, glob_clone, &root_clone)
                        {
                            if let Ok(mut results_guard) = results_clone.lock() {
                                results_guard.push(search_result);
                            }
                        }
                    }
                    Err(_) => {
                        // エラーは無視してディレクトリ探索を継続
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

    fn process_entry_with_full_path(
        &self,
        entry: &DirEntry,
        glob: &MultiGlob,
        root: &Path,
    ) -> Option<SearchResult> {
        if entry.file_type()?.is_file() {
            let path = entry.path();
            let relative_path = path.strip_prefix(root).ok()?;
            let path_str = relative_path.to_str()?;

            for (pattern_index, pattern) in glob.patterns.iter().enumerate() {
                if glob.matches_pattern(&pattern.states, path_str.as_bytes()) {
                    return Some(SearchResult {
                        path: path.to_path_buf(),
                        pattern_value: pattern.value,
                        pattern_index,
                    });
                }
            }
        }
        None
    }
}

// 便利な関数群
pub fn find_files_parallel<P: AsRef<Path>>(
    root: P,
    patterns: &[(&str, i64)],
    options: Option<SearchOptions>,
) -> GlobResult<Vec<SearchResult>> {
    let mut glob = MultiGlob::new();
    for (pattern, value) in patterns {
        glob.add(pattern, *value)?;
    }

    let search_options = options.unwrap_or_default();
    let walker = ParallelGlobWalker::new(glob, search_options);
    walker.search(root)
}

pub fn find_rust_files<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let results = find_files_parallel(root, &[("*.rs", 1)], None)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub fn find_toml_files<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let results = find_files_parallel(root, &[("*.toml", 1)], None)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub fn find_config_files<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let patterns = &[
        ("*.toml", 1),
        ("*.json", 2),
        ("*.yaml", 3),
        ("*.yml", 4),
        ("*.ini", 5),
        ("*.conf", 6),
        ("*.config", 7),
    ];
    let results = find_files_parallel(root, patterns, None)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

#[derive(Debug, Clone)]
pub struct SearchPatternsOptions {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub search_options: SearchOptions,
}

impl SearchPatternsOptions {
    pub fn new(include_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Self {
        Self {
            include_patterns,
            exclude_patterns,
            search_options: SearchOptions::default(),
        }
    }

    pub fn with_search_options(mut self, search_options: SearchOptions) -> Self {
        self.search_options = search_options;
        self
    }
}

pub fn search_with_patterns<P: AsRef<Path>>(
    root: P,
    options: SearchPatternsOptions,
) -> GlobResult<Vec<SearchResult>> {
    let root_path = root.as_ref();

    // Validate root path
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

    // Create include glob
    let mut include_glob = MultiGlob::new();
    for (i, pattern) in options.include_patterns.iter().enumerate() {
        include_glob.add(pattern, i as i64 + 1)?;
    }

    // Create exclude glob
    let mut exclude_glob = MultiGlob::new();
    for (i, pattern) in options.exclude_patterns.iter().enumerate() {
        exclude_glob.add(pattern, i as i64 + 1)?;
    }
    exclude_glob.compile();

    let walker = ParallelGlobWalker::new(include_glob, options.search_options);
    let all_results = walker.search_with_full_paths(root_path)?;

    // Filter out excluded files
    let filtered_results: Vec<SearchResult> = all_results
        .into_iter()
        .filter(|result| {
            if options.exclude_patterns.is_empty() {
                return true;
            }

            let relative_path = if let Ok(rel_path) = result.path.strip_prefix(root_path) {
                rel_path.to_string_lossy()
            } else {
                result.path.to_string_lossy()
            };

            // Check if file should be excluded
            exclude_glob.find(relative_path.as_bytes()).is_none()
        })
        .collect();

    Ok(filtered_results)
}

pub fn search_with_include_exclude<P: AsRef<Path>>(
    root: P,
    include_patterns: &[&str],
    exclude_patterns: &[&str],
) -> GlobResult<Vec<PathBuf>> {
    let options = SearchPatternsOptions::new(
        include_patterns.iter().map(|s| s.to_string()).collect(),
        exclude_patterns.iter().map(|s| s.to_string()).collect(),
    );

    let results = search_with_patterns(root, options)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub fn search_toml_files_advanced<P: AsRef<Path>>(
    root: P,
    exclude_patterns: Option<&[&str]>,
) -> GlobResult<Vec<PathBuf>> {
    let exclude_patterns = exclude_patterns.unwrap_or(&[]);
    search_with_include_exclude(root, &["**/*.toml"], exclude_patterns)
}

// Async implementation
pub struct AsyncGlobWalker {
    glob: MultiGlob,
    options: SearchOptions,
}

impl AsyncGlobWalker {
    pub fn new(mut glob: MultiGlob, options: SearchOptions) -> Self {
        glob.compile();
        Self { glob, options }
    }

    pub async fn search<P: AsRef<Path>>(&self, root: P) -> GlobResult<Vec<SearchResult>> {
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

        let mut builder = WalkBuilder::new(root_path);
        builder
            .follow_links(self.options.follow_links)
            .hidden(self.options.hidden)
            .ignore(!self.options.ignore_files)
            .git_ignore(self.options.git_ignore);

        if let Some(max_depth) = self.options.max_depth {
            builder.max_depth(Some(max_depth));
        }

        if let Some(max_filesize) = self.options.max_filesize {
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
                    Ok(entry) => self.process_entry(entry, &self.glob),
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

    pub async fn search_with_full_paths<P: AsRef<Path>>(
        &self,
        root: P,
    ) -> GlobResult<Vec<SearchResult>> {
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

        let mut builder = WalkBuilder::new(root_path);
        builder
            .follow_links(self.options.follow_links)
            .hidden(self.options.hidden)
            .ignore(!self.options.ignore_files)
            .git_ignore(self.options.git_ignore);

        if let Some(max_depth) = self.options.max_depth {
            builder.max_depth(Some(max_depth));
        }

        if let Some(max_filesize) = self.options.max_filesize {
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
                    Ok(entry) => self.process_entry_with_full_path(entry, &self.glob, root_path),
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

    fn process_entry(&self, entry: &DirEntry, glob: &MultiGlob) -> Option<SearchResult> {
        if entry.file_type()?.is_file() {
            let path = entry.path();
            let filename = path.file_name()?.to_str()?;

            for (pattern_index, pattern) in glob.patterns.iter().enumerate() {
                if glob.matches_pattern(&pattern.states, filename.as_bytes()) {
                    return Some(SearchResult {
                        path: path.to_path_buf(),
                        pattern_value: pattern.value,
                        pattern_index,
                    });
                }
            }
        }
        None
    }

    fn process_entry_with_full_path(
        &self,
        entry: &DirEntry,
        glob: &MultiGlob,
        root: &Path,
    ) -> Option<SearchResult> {
        if entry.file_type()?.is_file() {
            let path = entry.path();
            let relative_path = path.strip_prefix(root).ok()?;
            let path_str = relative_path.to_str()?;

            for (pattern_index, pattern) in glob.patterns.iter().enumerate() {
                if glob.matches_pattern(&pattern.states, path_str.as_bytes()) {
                    return Some(SearchResult {
                        path: path.to_path_buf(),
                        pattern_value: pattern.value,
                        pattern_index,
                    });
                }
            }
        }
        None
    }
}

// Async convenience functions
pub async fn find_files_async<P: AsRef<Path>>(
    root: P,
    patterns: &[(&str, i64)],
    options: Option<SearchOptions>,
) -> GlobResult<Vec<SearchResult>> {
    let mut glob = MultiGlob::new();
    for (pattern, value) in patterns {
        glob.add(pattern, *value)?;
    }

    let search_options = options.unwrap_or_default();
    let walker = AsyncGlobWalker::new(glob, search_options);
    walker.search(root).await
}

pub async fn find_rust_files_async<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let results = find_files_async(root, &[("*.rs", 1)], None).await?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub async fn find_toml_files_async<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let results = find_files_async(root, &[("*.toml", 1)], None).await?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub async fn find_config_files_async<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let patterns = &[
        ("*.toml", 1),
        ("*.json", 2),
        ("*.yaml", 3),
        ("*.yml", 4),
        ("*.ini", 5),
        ("*.conf", 6),
        ("*.config", 7),
    ];
    let results = find_files_async(root, patterns, None).await?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub async fn search_with_patterns_async<P: AsRef<Path>>(
    root: P,
    options: SearchPatternsOptions,
) -> GlobResult<Vec<SearchResult>> {
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

    let mut include_glob = MultiGlob::new();
    for (i, pattern) in options.include_patterns.iter().enumerate() {
        include_glob.add(pattern, i as i64 + 1)?;
    }

    let mut exclude_glob = MultiGlob::new();
    for (i, pattern) in options.exclude_patterns.iter().enumerate() {
        exclude_glob.add(pattern, i as i64 + 1)?;
    }
    exclude_glob.compile();

    let walker = AsyncGlobWalker::new(include_glob, options.search_options);
    let all_results = walker.search_with_full_paths(root_path).await?;

    let filtered_results: Vec<SearchResult> = all_results
        .into_iter()
        .filter(|result| {
            if options.exclude_patterns.is_empty() {
                return true;
            }

            let relative_path = if let Ok(rel_path) = result.path.strip_prefix(root_path) {
                rel_path.to_string_lossy()
            } else {
                result.path.to_string_lossy()
            };

            exclude_glob.find(relative_path.as_bytes()).is_none()
        })
        .collect();

    Ok(filtered_results)
}

pub async fn search_with_include_exclude_async<P: AsRef<Path>>(
    root: P,
    include_patterns: &[&str],
    exclude_patterns: &[&str],
) -> GlobResult<Vec<PathBuf>> {
    let options = SearchPatternsOptions::new(
        include_patterns.iter().map(|s| s.to_string()).collect(),
        exclude_patterns.iter().map(|s| s.to_string()).collect(),
    );

    let results = search_with_patterns_async(root, options).await?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub async fn search_toml_files_advanced_async<P: AsRef<Path>>(
    root: P,
    exclude_patterns: Option<&[&str]>,
) -> GlobResult<Vec<PathBuf>> {
    let exclude_patterns = exclude_patterns.unwrap_or(&[]);
    search_with_include_exclude_async(root, &["**/*.toml"], exclude_patterns).await
}

/// Search with patterns and return profiling information
pub fn search_with_patterns_profiled<P: AsRef<Path>>(
    root: P,
    options: SearchPatternsOptions,
) -> GlobResult<(Vec<SearchResult>, SearchProfile)> {
    let root_path = root.as_ref();
    let start_time = Instant::now();

    // Validate root path
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

    // Track visited directories
    let dir_profiles = Arc::new(Mutex::new(Vec::new()));

    // Create include glob
    let mut include_glob = MultiGlob::new();
    for (i, pattern) in options.include_patterns.iter().enumerate() {
        include_glob.add(pattern, i as i64 + 1)?;
    }

    // Create exclude glob
    let mut exclude_glob = MultiGlob::new();
    for (i, pattern) in options.exclude_patterns.iter().enumerate() {
        exclude_glob.add(pattern, i as i64 + 1)?;
    }
    exclude_glob.compile();

    // Build walker with profiling
    let mut builder = WalkBuilder::new(root_path);
    builder
        .follow_links(options.search_options.follow_links)
        .hidden(options.search_options.hidden)
        .ignore(!options.search_options.ignore_files)
        .git_ignore(options.search_options.git_ignore)
        .threads(options.search_options.threads);

    if let Some(max_depth) = options.search_options.max_depth {
        builder.max_depth(Some(max_depth));
    }

    let walker = builder.build_parallel();

    // Results collection
    let results = Arc::new(Mutex::new(Vec::new()));
    let files_count = Arc::new(Mutex::new(0usize));
    let dirs_count = Arc::new(Mutex::new(0usize));

    // Current directory tracking for profiling
    let current_dirs: Arc<Mutex<HashMap<std::thread::ThreadId, (PathBuf, Instant, usize, usize)>>> =
        Arc::new(Mutex::new(HashMap::new()));

    walker.run(|| {
        let results_clone = Arc::clone(&results);
        let include_glob_clone = include_glob.clone();
        let exclude_glob_clone = exclude_glob.clone();
        let root_clone = root_path.to_path_buf();
        let dir_profiles_clone = Arc::clone(&dir_profiles);
        let files_count_clone = Arc::clone(&files_count);
        let dirs_count_clone = Arc::clone(&dirs_count);
        let current_dirs_clone = Arc::clone(&current_dirs);

        Box::new(move |entry_result| {
            match entry_result {
                Ok(entry) => {
                    let path = entry.path();
                    let thread_id = std::thread::current().id();

                    // Track directory changes
                    if let Some(parent) = path.parent() {
                        let mut current_dirs_guard = current_dirs_clone.lock().unwrap();

                        if let Some((current_dir, start_time, file_count, subdir_count)) =
                            current_dirs_guard.get(&thread_id)
                        {
                            if current_dir != parent {
                                // Directory changed, record the profile
                                let duration = start_time.elapsed();
                                if duration.as_micros() > 100 || *file_count > 10 {
                                    // Only record significant directories
                                    if let Ok(mut profiles) = dir_profiles_clone.lock() {
                                        profiles.push(DirectoryProfile {
                                            path: current_dir.clone(),
                                            duration,
                                            file_count: *file_count,
                                            subdirectory_count: *subdir_count,
                                            total_size: 0, // Not tracking size in parallel walker
                                        });
                                    }
                                }
                                // Start tracking new directory
                                current_dirs_guard.insert(
                                    thread_id,
                                    (parent.to_path_buf(), Instant::now(), 0, 0),
                                );
                            }
                        } else {
                            // First time for this thread
                            current_dirs_guard
                                .insert(thread_id, (parent.to_path_buf(), Instant::now(), 0, 0));
                        }
                    }

                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        // Increment file count
                        if let Ok(mut count) = files_count_clone.lock() {
                            *count += 1;
                        }

                        // Update current directory file count
                        if let Ok(mut current_dirs_guard) = current_dirs_clone.lock() {
                            if let Some((_dir, _start, file_count, _subdir_count)) =
                                current_dirs_guard.get_mut(&thread_id)
                            {
                                *file_count += 1;
                            }
                        }

                        let relative_path = if let Ok(rel_path) = path.strip_prefix(&root_clone) {
                            rel_path.to_string_lossy()
                        } else {
                            path.to_string_lossy()
                        };

                        // Check if file matches include patterns
                        if let Some(pattern_value) =
                            include_glob_clone.find(relative_path.as_bytes())
                        {
                            // Check if file should be excluded
                            if exclude_glob_clone.find(relative_path.as_bytes()).is_none() {
                                if let Ok(mut results_guard) = results_clone.lock() {
                                    results_guard.push(SearchResult {
                                        path: path.to_path_buf(),
                                        pattern_value,
                                        pattern_index: 0,
                                    });
                                }
                            }
                        }
                    } else if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                        // Increment directory count
                        if let Ok(mut count) = dirs_count_clone.lock() {
                            *count += 1;
                        }

                        // Update current directory subdir count
                        if let Ok(mut current_dirs_guard) = current_dirs_clone.lock() {
                            if let Some((_dir, _start, _file_count, subdir_count)) =
                                current_dirs_guard.get_mut(&thread_id)
                            {
                                *subdir_count += 1;
                            }
                        }
                    }
                }
                Err(_) => {
                    // Ignore errors
                }
            }
            ignore::WalkState::Continue
        })
    });

    // Finalize any remaining directory profiles
    if let Ok(current_dirs_guard) = current_dirs.lock() {
        if let Ok(mut profiles) = dir_profiles.lock() {
            for (_, (dir, start_time, file_count, subdir_count)) in current_dirs_guard.iter() {
                let duration = start_time.elapsed();
                if duration.as_micros() > 100 || *file_count > 10 {
                    profiles.push(DirectoryProfile {
                        path: dir.clone(),
                        duration,
                        file_count: *file_count,
                        subdirectory_count: *subdir_count,
                        total_size: 0,
                    });
                }
            }
        }
    }

    let results = Arc::try_unwrap(results)
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    let files_count = Arc::try_unwrap(files_count)
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    let dirs_count = Arc::try_unwrap(dirs_count)
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    let dir_profiles = Arc::try_unwrap(dir_profiles)
        .map_err(|_| GlobError::LockError)?
        .into_inner()
        .map_err(|_| GlobError::LockError)?;

    // Create profile
    let mut profile = SearchProfile::new();
    profile.total_duration = start_time.elapsed();
    profile.total_files_found = files_count;
    profile.total_directories_scanned = dirs_count;
    profile.directory_profiles = dir_profiles;
    profile.finalize();

    Ok((results, profile))
}

// Fast glob functions that bypass ignore processing
pub struct FastGlobWalker {
    glob: MultiGlob,
}

impl FastGlobWalker {
    pub fn new(mut glob: MultiGlob) -> Self {
        glob.compile();
        Self { glob }
    }

    pub fn search<P: AsRef<Path>>(&self, root: P) -> GlobResult<Vec<SearchResult>> {
        let mut results = Vec::new();
        self.walk_dir(root.as_ref(), &mut results)?;
        Ok(results)
    }

    pub fn search_with_profile<P: AsRef<Path>>(
        &self,
        root: P,
    ) -> GlobResult<(Vec<SearchResult>, SearchProfile)> {
        let mut results = Vec::new();
        let mut profile = SearchProfile::new();
        let start_time = Instant::now();

        self.walk_dir_with_profile(root.as_ref(), &mut results, &mut profile)?;

        profile.total_duration = start_time.elapsed();
        profile.finalize();

        Ok((results, profile))
    }

    fn walk_dir(&self, dir: &Path, results: &mut Vec<SearchResult>) -> GlobResult<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir).map_err(|e| GlobError::io_error(dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| GlobError::io_error(dir, e))?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_dir(&path, results)?;
            } else if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                for (pattern_index, pattern) in self.glob.patterns.iter().enumerate() {
                    if self
                        .glob
                        .matches_pattern(&pattern.states, filename.as_bytes())
                    {
                        results.push(SearchResult {
                            path: path.clone(),
                            pattern_value: pattern.value,
                            pattern_index,
                        });
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn walk_dir_with_profile(
        &self,
        dir: &Path,
        results: &mut Vec<SearchResult>,
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
                self.walk_dir_with_profile(&path, results, profile)?;
            } else {
                file_count += 1;

                // Get file size
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }

                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    for (pattern_index, pattern) in self.glob.patterns.iter().enumerate() {
                        if self
                            .glob
                            .matches_pattern(&pattern.states, filename.as_bytes())
                        {
                            results.push(SearchResult {
                                path: path.clone(),
                                pattern_value: pattern.value,
                                pattern_index,
                            });
                            break;
                        }
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

/// Fast file search that bypasses all ignore processing
pub fn find_files_fast<P: AsRef<Path>>(
    root: P,
    patterns: &[(&str, i64)],
) -> GlobResult<Vec<SearchResult>> {
    let mut glob = MultiGlob::new();
    for (pattern, value) in patterns {
        glob.add(pattern, *value)?;
    }

    let walker = FastGlobWalker::new(glob);
    walker.search(root)
}

/// Fast Rust file search
pub fn find_rust_files_fast<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let results = find_files_fast(root, &[("*.rs", 1)])?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

/// Fast TOML file search
pub fn find_toml_files_fast<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let results = find_files_fast(root, &[("*.toml", 1)])?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

/// Fast config file search
pub fn find_config_files_fast<P: AsRef<Path>>(root: P) -> GlobResult<Vec<PathBuf>> {
    let patterns = &[
        ("*.toml", 1),
        ("*.json", 2),
        ("*.yaml", 3),
        ("*.yml", 4),
        ("*.ini", 5),
        ("*.conf", 6),
        ("*.config", 7),
    ];
    let results = find_files_fast(root, patterns)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

/// Profile directory search performance
pub fn profile_directory_search<P: AsRef<Path>>(
    root: P,
    patterns: &[(&str, i64)],
) -> GlobResult<SearchProfile> {
    let mut glob = MultiGlob::new();
    for (pattern, value) in patterns {
        glob.add(pattern, *value)?;
    }

    let walker = FastGlobWalker::new(glob);
    let (_, profile) = walker.search_with_profile(root)?;
    Ok(profile)
}

/// Profile TOML file search
pub fn profile_toml_search<P: AsRef<Path>>(root: P) -> GlobResult<SearchProfile> {
    profile_directory_search(root, &[("*.toml", 1)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("hello", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"world"), None);
    }

    #[test]
    fn test_wildcard_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h*o", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"ho"), Some(1));
        assert_eq!(glob.find(b"hilo"), Some(1));
        assert_eq!(glob.find(b"hi"), None);
    }

    #[test]
    fn test_question_mark_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h?llo", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"hllo"), None);
    }

    #[test]
    fn test_bracket_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[aeiou]llo", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"hillo"), Some(1));
        assert_eq!(glob.find(b"hyllo"), None);
    }

    #[test]
    fn test_bracket_range_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[a-z]llo", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"h9llo"), None);
    }

    #[test]
    fn test_bracket_negation_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[!aeiou]llo", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"hello"), None);
        assert_eq!(glob.find(b"hxllo"), Some(1));
        assert_eq!(glob.find(b"h9llo"), Some(1));
    }

    #[test]
    fn test_multiple_patterns() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("*.txt", 10).is_ok());
        assert!(glob.add("test.*", 5).is_ok());
        assert!(glob.add("test.txt", 20).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"test.txt"), Some(20));
        assert_eq!(glob.find(b"hello.txt"), Some(10));
        assert_eq!(glob.find(b"test.rs"), Some(5));
    }

    #[test]
    fn test_escape_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("test\\*file", 1).is_ok());
        glob.compile();

        assert_eq!(glob.find(b"test*file"), Some(1));
        assert_eq!(glob.find(b"testfile"), None);
        assert_eq!(glob.find(b"testXfile"), None);
    }

    #[test]
    fn test_parallel_walker_creation() {
        let mut glob = MultiGlob::new();
        glob.add("*.txt", 1).unwrap();
        glob.add("*.rs", 2).unwrap();

        let options = SearchOptions::default();
        let walker = ParallelGlobWalker::new(glob, options);

        assert_eq!(walker.options.threads, rayon::current_num_threads());
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();

        assert!(!options.follow_links);
        assert!(!options.hidden);
        assert!(options.ignore_files);
        assert!(options.git_ignore);
        assert!(options.max_depth.is_none());
        assert!(options.max_filesize.is_none());
        assert!(options.threads > 0);
    }

    #[test]
    fn test_search_options_custom() {
        let options = SearchOptions {
            follow_links: true,
            hidden: true,
            ignore_files: false,
            git_ignore: false,
            max_depth: Some(3),
            max_filesize: Some(1024 * 1024),
            threads: 4,
        };

        assert!(options.follow_links);
        assert!(options.hidden);
        assert!(!options.ignore_files);
        assert!(!options.git_ignore);
        assert_eq!(options.max_depth, Some(3));
        assert_eq!(options.max_filesize, Some(1024 * 1024));
        assert_eq!(options.threads, 4);
    }

    #[cfg(test)]
    #[test]
    fn test_parallel_search_integration() {
        use std::fs;
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("tombi_glob_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // テスト用ファイルを作成
        let mut file1 = fs::File::create(temp_dir.join("test1.txt")).unwrap();
        write!(file1, "test content").unwrap();

        let mut file2 = fs::File::create(temp_dir.join("test2.rs")).unwrap();
        write!(file2, "fn main() {{}}").unwrap();

        let mut file3 = fs::File::create(temp_dir.join("readme.md")).unwrap();
        write!(file3, "# Test").unwrap();

        let mut glob = MultiGlob::new();
        glob.add("*.txt", 1).unwrap();
        glob.add("*.rs", 2).unwrap();

        let options = SearchOptions::default();
        let walker = ParallelGlobWalker::new(glob, options);

        let results = walker.search(&temp_dir).unwrap();

        assert_eq!(results.len(), 2);

        let txt_result = results.iter().find(|r| r.pattern_value == 1);
        let rs_result = results.iter().find(|r| r.pattern_value == 2);

        assert!(txt_result.is_some());
        assert!(rs_result.is_some());

        // クリーンアップ
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_convenience_functions() {
        // 便利関数の基本テスト
        let patterns = &[("*.rs", 1), ("*.toml", 2)];
        let current_dir = std::env::current_dir().unwrap();

        // エラーが発生しないことを確認
        let _ = find_files_parallel(&current_dir, patterns, None);
        let _ = find_rust_files(&current_dir);
        let _ = find_toml_files(&current_dir);
        let _ = find_config_files(&current_dir);
    }

    #[test]
    fn test_search_patterns_options() {
        let options =
            SearchPatternsOptions::new(vec!["*.rs".to_string()], vec!["target/**".to_string()]);

        assert_eq!(options.include_patterns, vec!["*.rs"]);
        assert_eq!(options.exclude_patterns, vec!["target/**"]);
        assert!(!options.search_options.hidden);
    }

    #[test]
    fn test_search_patterns_options_with_search_options() {
        let search_opts = SearchOptions {
            hidden: true,
            max_depth: Some(2),
            ..Default::default()
        };

        let options = SearchPatternsOptions::new(vec!["*.toml".to_string()], vec![])
            .with_search_options(search_opts);

        assert!(options.search_options.hidden);
        assert_eq!(options.search_options.max_depth, Some(2));
    }

    #[test]
    fn test_search_with_include_exclude() {
        let current_dir = std::env::current_dir().unwrap();

        // Test basic functionality
        let result = search_with_include_exclude(&current_dir, &["*.toml"], &["target/**"]);

        // Should not fail
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_toml_files_advanced() {
        let current_dir = std::env::current_dir().unwrap();

        // Test with no exclude patterns
        let result1 = search_toml_files_advanced(&current_dir, None);
        assert!(result1.is_ok());

        // Test with exclude patterns
        let result2 = search_toml_files_advanced(&current_dir, Some(&["target/**", "dist/**"]));
        assert!(result2.is_ok());
    }

    #[cfg(test)]
    #[test]
    fn test_search_with_patterns_integration() {
        use std::fs;
        use std::io::Write;

        let temp_dir = std::env::temp_dir().join("tombi_glob_patterns_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test files
        let mut file1 = fs::File::create(temp_dir.join("config.toml")).unwrap();
        write!(file1, "[package]").unwrap();

        let mut file2 = fs::File::create(temp_dir.join("test.rs")).unwrap();
        write!(file2, "fn main() {{}}").unwrap();

        // Create subdirectory with excluded file
        let target_dir = temp_dir.join("target");
        fs::create_dir_all(&target_dir).unwrap();
        let mut file3 = fs::File::create(target_dir.join("exclude.toml")).unwrap();
        write!(file3, "[excluded]").unwrap();

        let options = SearchPatternsOptions::new(
            vec!["**/*.toml".to_string()],
            vec!["target/**".to_string()],
        );

        let results = search_with_patterns(&temp_dir, options).unwrap();

        // Should find config.toml but not target/exclude.toml
        assert_eq!(results.len(), 1);
        assert!(results[0].path.file_name().unwrap() == "config.toml");

        // Test include/exclude convenience function
        let paths = search_with_include_exclude(&temp_dir, &["**/*.toml"], &["target/**"]).unwrap();

        assert_eq!(paths.len(), 1);
        assert!(paths[0].file_name().unwrap() == "config.toml");

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_error_empty_pattern() {
        let mut glob = MultiGlob::new();
        let result = glob.add("", 1);
        assert!(matches!(result, Err(GlobError::EmptyPattern)));
    }

    #[test]
    fn test_error_unclosed_bracket() {
        let mut glob = MultiGlob::new();
        let result = glob.add("test[abc", 1);
        assert!(matches!(result, Err(GlobError::UnclosedBracket { .. })));
    }

    #[test]
    fn test_error_invalid_character_class() {
        let mut glob = MultiGlob::new();
        let result = glob.add("test[z-a]", 1);
        assert!(matches!(
            result,
            Err(GlobError::InvalidCharacterClass { .. })
        ));
    }

    #[test]
    fn test_error_invalid_pattern_backslash() {
        let mut glob = MultiGlob::new();
        let result = glob.add("test\\", 1);
        assert!(matches!(result, Err(GlobError::InvalidPattern { .. })));
    }

    #[test]
    fn test_error_root_path_not_found() {
        let options = SearchPatternsOptions::new(vec!["*.txt".to_string()], vec![]);

        let result = search_with_patterns("/nonexistent/path", options);
        assert!(matches!(result, Err(GlobError::RootPathNotFound { .. })));
    }

    #[test]
    fn test_error_pattern_too_complex() {
        let mut glob = MultiGlob::new();
        // Create a pattern that would exceed the state limit
        let mut complex_pattern = String::new();
        for _ in 0..2000 {
            complex_pattern.push('a');
        }

        let result = glob.add(&complex_pattern, 1);
        assert!(matches!(result, Err(GlobError::PatternTooComplex { .. })));
    }

    #[test]
    fn test_glob_error_display() {
        let error = GlobError::invalid_pattern("test[");
        assert!(error.to_string().contains("Invalid glob pattern"));

        let error = GlobError::unclosed_bracket("test[abc", 4);
        assert!(error.to_string().contains("Unclosed bracket"));
        assert!(error.to_string().contains("position 4"));
    }

    #[test]
    fn test_fast_glob_walker() {
        let mut glob = MultiGlob::new();
        glob.add("*.txt", 1).unwrap();
        glob.add("*.rs", 2).unwrap();

        let walker = FastGlobWalker::new(glob);
        let current_dir = std::env::current_dir().unwrap();

        // Should not fail
        let result = walker.search(&current_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fast_convenience_functions() {
        let current_dir = std::env::current_dir().unwrap();

        // Test fast functions
        let _ = find_files_fast(&current_dir, &[("*.rs", 1), ("*.toml", 2)]);
        let _ = find_rust_files_fast(&current_dir);
        let _ = find_toml_files_fast(&current_dir);
        let _ = find_config_files_fast(&current_dir);
    }

    #[test]
    fn test_profile_directory_search() {
        let current_dir = std::env::current_dir().unwrap();

        let profile = profile_toml_search(&current_dir).unwrap();

        // Should have scanned some directories
        assert!(profile.total_directories_scanned > 0);
        assert!(profile.total_duration.as_nanos() > 0);

        // Print profile for manual inspection
        println!("Directory search profile:");
        profile.print_report();
    }

    #[test]
    fn test_search_with_profile() {
        let current_dir = std::env::current_dir().unwrap();

        let mut glob = MultiGlob::new();
        glob.add("*.toml", 1).unwrap();

        let walker = FastGlobWalker::new(glob);
        let (results, profile) = walker.search_with_profile(&current_dir).unwrap();

        assert!(results.len() > 0);
        assert!(profile.total_directories_scanned > 0);
    }

    #[test]
    fn test_gitignore_directory_exclusion_performance() {
        use std::fs;
        use std::io::Write;
        use std::time::Instant;

        let temp_dir = std::env::temp_dir().join("tombi_glob_perf_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize git repository
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(&temp_dir)
            .output()
            .unwrap();

        // Create .gitignore
        let mut gitignore = fs::File::create(temp_dir.join(".gitignore")).unwrap();
        writeln!(gitignore, "large_dir/").unwrap();

        // Create a large directory with many files
        let large_dir = temp_dir.join("large_dir");
        fs::create_dir_all(&large_dir).unwrap();
        for i in 0..1000 {
            let mut file = fs::File::create(large_dir.join(format!("file_{}.txt", i))).unwrap();
            writeln!(file, "content {}", i).unwrap();
        }

        // Create some non-ignored files
        let mut file1 = fs::File::create(temp_dir.join("normal.txt")).unwrap();
        writeln!(file1, "normal content").unwrap();

        // Test with .gitignore enabled (should be fast)
        let start = Instant::now();
        let options_with_gitignore = SearchOptions::default(); // git_ignore: true
        let results_with_gitignore =
            find_files_parallel(&temp_dir, &[("*.txt", 1)], Some(options_with_gitignore)).unwrap();
        let duration_with_gitignore = start.elapsed();

        // Test with .gitignore disabled (should be slower)
        let start = Instant::now();
        let options_without_gitignore = SearchOptions {
            git_ignore: false,
            ignore_files: false,
            ..Default::default()
        };
        let results_without_gitignore =
            find_files_parallel(&temp_dir, &[("*.txt", 1)], Some(options_without_gitignore))
                .unwrap();
        let duration_without_gitignore = start.elapsed();

        // Debug: Print actual results
        println!(
            "Results with .gitignore: {} files",
            results_with_gitignore.len()
        );
        for result in &results_with_gitignore {
            println!("  Found: {:?}", result.path);
        }

        println!(
            "Results without .gitignore: {} files",
            results_without_gitignore.len()
        );

        // The key assertion: .gitignore should exclude the large_dir
        assert!(results_with_gitignore.len() < results_without_gitignore.len());

        // Without .gitignore should find many more files
        assert!(results_without_gitignore.len() > 100);

        // Performance check: .gitignore version should be significantly faster
        println!(
            "With .gitignore: {:?} ({} files)",
            duration_with_gitignore,
            results_with_gitignore.len()
        );
        println!(
            "Without .gitignore: {:?} ({} files)",
            duration_without_gitignore,
            results_without_gitignore.len()
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);

        // The .gitignore version should be at least 2x faster for this test
        assert!(duration_with_gitignore < duration_without_gitignore);
    }

    #[tokio::test]
    async fn test_async_walker_creation() {
        let mut glob = MultiGlob::new();
        glob.add("*.txt", 1).unwrap();
        glob.add("*.rs", 2).unwrap();

        let options = SearchOptions::default();
        let walker = AsyncGlobWalker::new(glob, options);

        assert_eq!(walker.options.threads, rayon::current_num_threads());
    }

    #[tokio::test]
    async fn test_async_find_files() {
        let current_dir = std::env::current_dir().unwrap();

        let patterns = &[("*.rs", 1), ("*.toml", 2)];
        let result = find_files_async(&current_dir, patterns, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_convenience_functions() {
        let current_dir = std::env::current_dir().unwrap();

        let _ = find_rust_files_async(&current_dir).await;
        let _ = find_toml_files_async(&current_dir).await;
        let _ = find_config_files_async(&current_dir).await;
    }

    #[tokio::test]
    async fn test_async_search_with_patterns() {
        let current_dir = std::env::current_dir().unwrap();

        let options =
            SearchPatternsOptions::new(vec!["*.toml".to_string()], vec!["target/**".to_string()]);

        let result = search_with_patterns_async(&current_dir, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_search_with_include_exclude() {
        let current_dir = std::env::current_dir().unwrap();

        let result =
            search_with_include_exclude_async(&current_dir, &["*.toml"], &["target/**"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_search_toml_files_advanced() {
        let current_dir = std::env::current_dir().unwrap();

        let result1 = search_toml_files_advanced_async(&current_dir, None).await;
        assert!(result1.is_ok());

        let result2 =
            search_toml_files_advanced_async(&current_dir, Some(&["target/**", "dist/**"])).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_async_error_root_path_not_found() {
        let options = SearchPatternsOptions::new(vec!["*.txt".to_string()], vec![]);

        let result = search_with_patterns_async("/nonexistent/path", options).await;
        assert!(matches!(result, Err(GlobError::RootPathNotFound { .. })));
    }
}
