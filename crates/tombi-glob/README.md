# tombi-glob

A high-performance Rust library for parallel directory traversal and glob pattern matching.

## Features

- **Fast Glob Pattern Matching**: NFA-based algorithm using fast-glob
- **Parallel Directory Traversal**: High-speed parallel processing using Rayon
- **Async Support**: Runtime-independent async file search functionality
- **Git Integration**: Automatic recognition of .gitignore files
- **Memory Efficient**: Efficient operation even with large directory structures

## Supported Patterns

- `*` - Matches zero or more characters
- `?` - Matches exactly one character
- `[abc]` - Character class (matches a, b, or c)
- `[a-z]` - Character range (matches a through z)
- `[!abc]` or `[^abc]` - Negated character class
- `\` - Escape character

## Main Components

- `MultiGlob` - Multi-pattern glob matching with priority values
- `ParallelGlobWalker` - Parallel directory traversal with glob matching
- `AsyncGlobWalker` - Async directory traversal with glob matching
- `FastGlobWalker` - Fast traversal without .gitignore processing
- `SearchOptions` - Configuration for search behavior
- `SearchPatternsOptions` - Include/exclude pattern configuration

## Convenience Functions

- `find_rust_files()` - Find Rust source files
- `find_toml_files()` - Find TOML configuration files
- `find_config_files()` - Find common configuration files
- `search_with_patterns()` - Search with include/exclude patterns
- Async variants available for all functions

## Performance

- Parallel processing with automatic thread count adjustment
- Efficient NFA-based pattern matching
- Optional .gitignore integration for file exclusion
- Fast mode available for maximum performance

## License

MIT
