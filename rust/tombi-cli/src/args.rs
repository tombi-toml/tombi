use clap::{
    builder::styling::{AnsiColor, Color, Style},
    Parser,
};
use clap_complete::Shell;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(clap::Parser)]
#[command(
    name = "tombi",
    about = app_about(),
    version = option_env!("__TOMBI_VERSION").map_or(env!("CARGO_PKG_VERSION"), |v| v.trim_start_matches('v')),
    styles = app_styles(),
    disable_help_subcommand(true),
)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: TomlCommand,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

impl<I, T> From<I> for Args
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    #[inline]
    fn from(value: I) -> Self {
        Self::parse_from(value)
    }
}

#[derive(clap::Args, Debug)]
pub struct CommonArgs {
    /// Disable network access.
    ///
    /// Don't fetch from remote and use local schemas cache.
    #[clap(long, global = true, env("TOMBI_OFFLINE"))]
    pub offline: bool,

    /// Do not use cache.
    ///
    /// Fetch the latest data from remote and save it to the cache.
    #[clap(long, global = true, env("TOMBI_NO_CACHE"))]
    pub no_cache: bool,
}

#[derive(clap::Subcommand)]
pub enum TomlCommand {
    #[command(alias = "fmt")]
    Format(FormatArgs),

    #[command(alias = "check")]
    Lint(LintArgs),

    #[command(alias = "serve")]
    Lsp(LspArgs),

    Completion(CompletionArgs),
}

/// Format TOML files.
#[derive(clap::Args, Debug)]
pub struct FormatArgs {
    /// List of files or directories to format.
    ///
    /// If the only argument is "-", the standard input will be used.
    ///
    /// [default: if "tombi.toml" exists, format project directory, otherwise format current directory]
    pub files: Vec<String>,

    /// Check only and don't overwrite files.
    #[arg(long, default_value_t = false)]
    pub check: bool,

    /// Filename to use when reading from stdin.
    ///
    /// This is useful for determining which JSON Schema should be applied, for more rich formatting.
    #[arg(long)]
    pub stdin_filename: Option<String>,

    #[command(flatten)]
    pub common: CommonArgs,
}

/// Lint TOML files.
#[derive(clap::Args, Debug)]
pub struct LintArgs {
    /// List of files or directories to lint.
    ///
    /// If the only argument is "-", the standard input will be used.
    ///
    /// [default: if "tombi.toml" exists, lint project directory, otherwise lint current directory]
    pub files: Vec<String>,

    /// Filename to use when reading from stdin.
    ///
    /// This is useful for determining which JSON Schema should be applied, for more rich linting.
    #[arg(long)]
    pub stdin_filename: Option<String>,

    #[command(flatten)]
    pub common: CommonArgs,
}

/// Run TOML Language Server.
#[derive(Debug, clap::Args)]
pub struct LspArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Generate shell completion.
#[derive(clap::Args, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completion for.
    #[arg(value_enum)]
    pub shell: Shell,
}

fn app_about() -> String {
    let title = "Tombi";
    let title_style = Style::new()
        .bold()
        .bg_color(Some(Color::Ansi(AnsiColor::Blue)))
        .fg_color(Some(Color::Ansi(AnsiColor::White)));

    let desc_style = Style::new()
        .bg_color(Some(Color::Ansi(AnsiColor::Blue)))
        .fg_color(Some(Color::Ansi(AnsiColor::White)));

    format!(
        "{title_style}                          {title} {title_style:#}{desc_style}: TOML Toolkit                          {desc_style:#}"
    )
}

const fn app_styles() -> clap::builder::Styles {
    clap::builder::Styles::plain()
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .valid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
}
