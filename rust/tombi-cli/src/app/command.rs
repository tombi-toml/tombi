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
