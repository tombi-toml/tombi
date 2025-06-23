use ignore::{DirEntry, WalkBuilder};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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

#[derive(Debug)]
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
                if states.len() > 1000 { // 合理的な制限を設定
                    return Err(GlobError::pattern_too_complex(pattern, 1000));
                }
                self.patterns.push(GlobPattern::new(states, value));
                self.compiled = false;
                Ok(())
            }
            Err(e) => Err(e)
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
                                    return Err(GlobError::invalid_character_class(pattern, bracket_start));
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
        assert!(matches!(result, Err(GlobError::InvalidCharacterClass { .. })));
    }

    #[test]
    fn test_error_invalid_pattern_backslash() {
        let mut glob = MultiGlob::new();
        let result = glob.add("test\\", 1);
        assert!(matches!(result, Err(GlobError::InvalidPattern { .. })));
    }

    #[test]
    fn test_error_root_path_not_found() {
        let options = SearchPatternsOptions::new(
            vec!["*.txt".to_string()],
            vec![],
        );
        
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
}
