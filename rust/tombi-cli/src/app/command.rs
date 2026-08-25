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

pub(super) fn file_open_error(
    source_path: std::path::PathBuf,
    error: std::io::Error,
) -> crate::Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(source_path))
    } else {
        crate::Error::Io(error)
    }
}
