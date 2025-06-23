
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use ignore::{WalkBuilder, DirEntry};

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

    pub fn add(&mut self, pattern: &str, value: i64) -> bool {
        if let Some(states) = parse_glob(pattern) {
            self.patterns.push(GlobPattern::new(states, value));
            self.compiled = false;
            true
        } else {
            false
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

    fn match_recursive(&self, states: &[State], state_idx: usize, input: &[u8], input_idx: usize) -> bool {
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

fn parse_glob(pattern: &str) -> Option<Vec<State>> {
    let mut states: Vec<State> = Vec::new();
    let mut chars = pattern.bytes().peekable();

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
                continue;
            }
            b'?' => {
                char_set.set_all();
            }
            b'\\' => {
                if let Some(&next_char) = chars.peek() {
                    char_set.set_char(next_char);
                    chars.next();
                } else {
                    return None;
                }
            }
            b'[' => {
                let mut negate = false;
                let mut bracket_chars = Vec::new();
                let mut closed = false;

                if let Some(&b'!') | Some(&b'^') = chars.peek() {
                    negate = true;
                    chars.next();
                }

                if let Some(&b']') = chars.peek() {
                    bracket_chars.push(b']');
                    chars.next();
                }

                while let Some(bracket_char) = chars.next() {
                    if bracket_char == b']' {
                        closed = true;
                        break;
                    }

                    if bracket_char == b'-' && !bracket_chars.is_empty() {
                        if let Some(&end_char) = chars.peek() {
                            if end_char != b']' {
                                let start_char = *bracket_chars.last().unwrap();
                                chars.next();
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
                    return None;
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
    }

    if states.is_empty() {
        return None;
    }

    Some(states)
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

    pub fn search<P: AsRef<Path>>(&self, root: P) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
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
            .map_err(|_| "Failed to unwrap results")?
            .into_inner()
            .map_err(|_| "Failed to acquire lock")?;

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

    pub fn search_with_full_paths<P: AsRef<Path>>(&self, root: P) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
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
                        if let Some(search_result) = self.process_entry_with_full_path(&entry, glob_clone, &root_clone) {
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
            .map_err(|_| "Failed to unwrap results")?
            .into_inner()
            .map_err(|_| "Failed to acquire lock")?;

        Ok(results)
    }

    fn process_entry_with_full_path(&self, entry: &DirEntry, glob: &MultiGlob, root: &Path) -> Option<SearchResult> {
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
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let mut glob = MultiGlob::new();
    for (pattern, value) in patterns {
        glob.add(pattern, *value);
    }
    
    let search_options = options.unwrap_or_default();
    let walker = ParallelGlobWalker::new(glob, search_options);
    walker.search(root)
}

pub fn find_rust_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let results = find_files_parallel(root, &[("*.rs", 1)], None)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub fn find_toml_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let results = find_files_parallel(root, &[("*.toml", 1)], None)?;
    Ok(results.into_iter().map(|r| r.path).collect())
}

pub fn find_config_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("hello", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"world"), None);
    }

    #[test]
    fn test_wildcard_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h*o", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"ho"), Some(1));
        assert_eq!(glob.find(b"hilo"), Some(1));
        assert_eq!(glob.find(b"hi"), None);
    }

    #[test]
    fn test_question_mark_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h?llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"hllo"), None);
    }

    #[test]
    fn test_bracket_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[aeiou]llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"hillo"), Some(1));
        assert_eq!(glob.find(b"hyllo"), None);
    }

    #[test]
    fn test_bracket_range_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[a-z]llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"h9llo"), None);
    }

    #[test]
    fn test_bracket_negation_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[!aeiou]llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), None);
        assert_eq!(glob.find(b"hxllo"), Some(1));
        assert_eq!(glob.find(b"h9llo"), Some(1));
    }

    #[test]
    fn test_multiple_patterns() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("*.txt", 10));
        assert!(glob.add("test.*", 5));
        assert!(glob.add("test.txt", 20));
        glob.compile();
        
        assert_eq!(glob.find(b"test.txt"), Some(20));
        assert_eq!(glob.find(b"hello.txt"), Some(10));
        assert_eq!(glob.find(b"test.rs"), Some(5));
    }

    #[test]
    fn test_escape_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("test\\*file", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"test*file"), Some(1));
        assert_eq!(glob.find(b"testfile"), None);
        assert_eq!(glob.find(b"testXfile"), None);
    }

    #[test]
    fn test_parallel_walker_creation() {
        let mut glob = MultiGlob::new();
        glob.add("*.txt", 1);
        glob.add("*.rs", 2);
        
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
        glob.add("*.txt", 1);
        glob.add("*.rs", 2);
        
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
}