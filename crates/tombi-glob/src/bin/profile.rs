use serde_tombi;
use std::{env, time::Instant};
use tombi_config;
use tombi_glob::{SearchOptions, SearchPatternsOptions};

fn main() {
    let args: Vec<String> = env::args().collect();
    let root = if args.len() > 1 { &args[1] } else { "." };

    println!(
        "Profiling TOML file search in: {} (using tombi.toml config)",
        root
    );

    // Load tombi.toml configuration like tombi format does
    let (config, _config_path) = match serde_tombi::config::load_with_path() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Failed to load tombi.toml: {}", e);
            eprintln!("Using default configuration");
            (Default::default(), None)
        }
    };

    let default_include = vec!["**/*.toml".to_string()];
    let include_patterns = config
        .include
        .as_deref()
        .unwrap_or(&default_include)
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    let exclude_patterns = config
        .exclude
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    println!("Include patterns: {:?}", include_patterns);
    println!("Exclude patterns: {:?}", exclude_patterns);
    println!("Git ignore enabled: true");
    println!("Ignore files enabled: true");

    // Create search options to match tombi format behavior
    let search_options = SearchOptions::default(); // git_ignore: true, ignore_files: true

    let patterns_options = SearchPatternsOptions::new(
        include_patterns.iter().map(|s| s.to_string()).collect(),
        exclude_patterns.iter().map(|s| s.to_string()).collect(),
    )
    .with_search_options(search_options);

    // Profile the same search function that tombi format uses
    println!("\n=== Profiling tombi format search ===");
    let start_time = Instant::now();

    match tombi_glob::search_with_patterns_profiled(root, patterns_options) {
        Ok((results, profile)) => {
            let total_duration = start_time.elapsed();

            println!("Total search time: {:?}", total_duration);
            println!("Found {} TOML files", results.len());

            // Show directory traversal profile
            println!("\n=== Directory Traversal Profile ===");
            println!(
                "Total directories visited: {}",
                profile.total_directories_scanned
            );
            println!("Total files examined: {}", profile.total_files_found);

            // Show slowest directories
            println!("\n=== Top 10 Slowest Directories ===");
            for (i, dir_profile) in profile.slowest_directories.iter().enumerate() {
                println!(
                    "{}. {:?} - {:?} ({} files, {} subdirs)",
                    i + 1,
                    dir_profile.path,
                    dir_profile.duration,
                    dir_profile.file_count,
                    dir_profile.subdirectory_count
                );

                // Show if this directory should have been excluded
                let path_str = dir_profile.path.to_string_lossy();
                if path_str.contains("target") || path_str.contains("node_modules") {
                    println!("   ⚠️  This directory should be excluded by .gitignore!");
                }
            }

            // Show sample results
            println!("\n=== Sample files found ===");
            for (i, result) in results.iter().take(10).enumerate() {
                println!("  {}. {:?}", i + 1, result.path);
            }
            if results.len() > 10 {
                println!("  ... and {} more", results.len() - 10);
            }
        }
        Err(e) => {
            eprintln!("Error during search: {}", e);
        }
    }

    // Compare with ignore-disabled search
    println!("\n=== Comparison: Search without .gitignore ===");
    let search_options_no_ignore = SearchOptions {
        git_ignore: false,
        ignore_files: false,
        ..SearchOptions::default()
    };

    let patterns_options_no_ignore = SearchPatternsOptions::new(
        include_patterns.iter().map(|s| s.to_string()).collect(),
        exclude_patterns.iter().map(|s| s.to_string()).collect(),
    )
    .with_search_options(search_options_no_ignore);

    let start_time_no_ignore = Instant::now();
    match tombi_glob::search_with_patterns(root, patterns_options_no_ignore) {
        Ok(results) => {
            let duration_no_ignore = start_time_no_ignore.elapsed();
            println!("Time without .gitignore: {:?}", duration_no_ignore);
            println!("Found {} TOML files", results.len());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
