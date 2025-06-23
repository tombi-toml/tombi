# tombi-glob

A high-performance Rust library for parallel directory traversal and glob pattern matching.

## Features

- **Fast Glob Pattern Matching**: NFA (Non-deterministic Finite Automaton) based algorithm
- **Parallel Directory Traversal**: High-speed parallel processing using Rayon
- **Async Support**: Runtime-independent async file search functionality
- **Flexible Configuration**: Control over hidden files, symbolic links, depth limits, etc.
- **Git Integration**: Automatic recognition of .gitignore files
- **Memory Efficient**: Efficient operation even with large directory structures

## Supported Patterns

- `*` - Matches zero or more characters
- `?` - Matches exactly one character
- `[abc]` - Character class (matches a, b, or c)
- `[a-z]` - Character range (matches a through z)
- `[!abc]` or `[^abc]` - Negated character class (matches anything except a, b, or c)
- `\` - Escape character

## Usage Examples

### Basic Glob Matching

```rust
use tombi_glob::MultiGlob;

let mut glob = MultiGlob::new();
glob.add("*.rs", 1);
glob.add("*.toml", 2);
glob.compile();

// Match filenames as byte strings
assert_eq!(glob.find(b"main.rs"), Some(1));
assert_eq!(glob.find(b"Cargo.toml"), Some(2));
```

### Parallel Directory Search

```rust
use tombi_glob::{ParallelGlobWalker, SearchOptions, MultiGlob};

let mut glob = MultiGlob::new();
glob.add("*.rs", 1);
glob.add("*.toml", 2);

let options = SearchOptions {
    hidden: false,
    max_depth: Some(5),
    threads: 8,
    ..Default::default()
};

let walker = ParallelGlobWalker::new(glob, options);
let results = walker.search("./src")?;

for result in results {
    println!("Found: {} (pattern value: {})", 
             result.path.display(), result.pattern_value);
}
```

### Convenience Functions

```rust
use tombi_glob::{find_rust_files, find_config_files};

// Find Rust files
let rust_files = find_rust_files("./src")?;

// Find configuration files (.toml, .json, .yaml, etc.)
let config_files = find_config_files(".")?;
```

### Fast File Search (No .gitignore Processing)

For maximum performance when you don't need .gitignore support:

```rust
use tombi_glob::{find_rust_files_fast, find_toml_files_fast, FastGlobWalker, SearchOptions};

// Fast functions that bypass all ignore processing
let rust_files = find_rust_files_fast("./src")?;
let toml_files = find_toml_files_fast(".")?;

// Fast search options
let fast_options = SearchOptions::fast();  // Disables .gitignore processing
let minimal_options = SearchOptions::minimal();  // Single-threaded, depth-limited

// Custom fast walker
let mut glob = MultiGlob::new();
glob.add("*.rs", 1);
let walker = FastGlobWalker::new(glob);
let results = walker.search("./src")?;
```

### Async File Search

```rust
use tombi_glob::{AsyncGlobWalker, SearchOptions, MultiGlob};
use tombi_glob::{find_rust_files_async, search_with_patterns_async, SearchPatternsOptions};

// Async parallel directory search
let mut glob = MultiGlob::new();
glob.add("*.rs", 1);
glob.add("*.toml", 2);

let options = SearchOptions::default();
let walker = AsyncGlobWalker::new(glob, options);
let results = walker.search("./src").await?;

// Async convenience functions
let rust_files = find_rust_files_async("./src").await?;

// Async search with patterns
let options = SearchPatternsOptions::new(
    vec!["**/*.rs".to_string()],
    vec!["target/**".to_string()],
);
let results = search_with_patterns_async(".", options).await?;
```

## Performance

- **Parallel Processing**: Automatically adjusts thread count based on CPU cores
- **Efficient NFA**: Minimizes memory usage for pattern matching
- **Git Integration**: Excludes unnecessary files using .gitignore

## Configuration Options

```rust
use tombi_glob::SearchOptions;

let options = SearchOptions {
    follow_links: false,      // Whether to follow symbolic links
    hidden: false,            // Whether to include hidden files
    ignore_files: true,       // Whether to use .ignore/.gitignore files
    git_ignore: true,         // Whether to use .gitignore files
    max_depth: Some(10),      // Maximum search depth
    max_filesize: Some(1024), // Maximum file size in bytes
    threads: 8,               // Number of threads to use
};
```

## License

MIT