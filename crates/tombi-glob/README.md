# tombi-glob

A high-performance Rust library for parallel directory traversal and glob pattern matching.

## Features

- **Fast Glob Pattern Matching**: NFA (Non-deterministic Finite Automaton) based algorithm
- **Parallel Directory Traversal**: High-speed parallel processing using Rayon
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