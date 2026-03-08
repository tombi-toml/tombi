use std::{env, path::PathBuf, time::Instant};

use tombi_config::FilesOptions;
use tombi_glob::{FileSearchEntry, search_pattern_matched_paths};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let root = args.get(1).map_or(".", String::as_str);

    println!(
        "Profiling TOML file search in: {} (using .tombi.toml/tombi.toml config)",
        root
    );

    let search_dir = Some(PathBuf::from(root));
    let (config, _config_path) = match serde_tombi::config::load_with_path(search_dir) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Warning: Failed to load .tombi.toml/tombi.toml: {err}");
            eprintln!("Using default configuration");
            (Default::default(), None)
        }
    };

    let files_options = config.files.clone().unwrap_or_else(|| FilesOptions {
        include: Some(vec!["**/*.toml".to_string()]),
        exclude: None,
    });

    println!("Include patterns: {:?}", files_options.include);
    println!("Exclude patterns: {:?}", files_options.exclude);

    let start_time = Instant::now();
    let entries = search_pattern_matched_paths(root, files_options).await;
    let duration = start_time.elapsed();

    let mut found = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for entry in &entries {
        match entry {
            FileSearchEntry::Found(_) => found += 1,
            FileSearchEntry::Skipped(_) => skipped += 1,
            FileSearchEntry::Error(_) => errors += 1,
        }
    }

    println!("Search completed in: {duration:?}");
    println!("Found: {found}, Skipped: {skipped}, Errors: {errors}");

    for (index, entry) in entries.iter().take(10).enumerate() {
        if let FileSearchEntry::Found(path) = entry {
            println!("{}. {}", index + 1, path.display());
        }
    }
}
