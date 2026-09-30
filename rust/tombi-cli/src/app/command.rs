pub mod completion;
pub mod format;
pub mod lint;
pub mod lsp;

#[derive(clap::Subcommand)]
pub enum TomlCommand {
    #[command(alias = "fmt")]
    Format(format::Args),

    #[command(alias = "check")]
    Lint(lint::Args),

    #[command(alias = "serve")]
    Lsp(lsp::Args),

    Completion(completion::Args),
}

pub(super) fn max_concurrency() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

pub(super) fn runtime(single_threaded: bool) -> std::io::Result<tokio::runtime::Runtime> {
    let mut builder = if single_threaded {
        tokio::runtime::Builder::new_current_thread()
    } else {
        tokio::runtime::Builder::new_multi_thread()
    };
    builder.enable_all().build()
}

/// The files that are read by the command: the config file and the found files.
pub(super) fn input_paths<'a>(
    input: &'a tombi_glob::FileSearch,
    config_path: Option<&'a std::path::Path>,
) -> impl Iterator<Item = &'a std::path::Path> {
    let entries: &[tombi_glob::FileSearchEntry] = match input {
        tombi_glob::FileSearch::Files(entries) => entries,
        tombi_glob::FileSearch::Stdin => &[],
    };
    config_path
        .into_iter()
        .chain(entries.iter().filter_map(|entry| match entry {
            tombi_glob::FileSearchEntry::Found(path) => Some(path.as_path()),
            _ => None,
        }))
}

pub(super) fn file_open_error(
    source_path: std::path::PathBuf,
    error: std::io::Error,
) -> crate::Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(source_path))
    } else {
        crate::Error::FileOpenFailed {
            path: source_path,
            source: error,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tombi_glob::{FileSearch, FileSearchEntry};

    macro_rules! test_input_paths {
        ($name:ident: $input:expr, $config:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let input: FileSearch = $input;
                let config: Option<&Path> = $config;
                pretty_assertions::assert_eq!(
                    super::input_paths(&input, config)
                        .map(Path::to_owned)
                        .collect::<Vec<_>>(),
                    $expected.map(PathBuf::from).to_vec()
                );
            }
        };
    }

    test_input_paths!(
        config_and_found_files_are_inputs:
        FileSearch::Files(vec![
            FileSearchEntry::Found("a.toml".into()),
            FileSearchEntry::Skipped("skipped.toml".into()),
            FileSearchEntry::Error(tombi_glob::Error::FileNotFound("missing.toml".into())),
            FileSearchEntry::Found("b.toml".into()),
        ]),
        Some(Path::new("tombi.toml")) => ["tombi.toml", "a.toml", "b.toml"]
    );
    test_input_paths!(stdin_has_only_config: FileSearch::Stdin, Some(Path::new("tombi.toml")) => ["tombi.toml"]);
    test_input_paths!(stdin_without_config_has_no_inputs: FileSearch::Stdin, None => [] as [&str; 0]);
}
