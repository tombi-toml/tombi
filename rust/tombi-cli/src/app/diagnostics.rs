mod destination;
mod format_reporter;
mod pretty;
mod report;

use std::io::Write;

use destination::Destination;
use format_reporter::FormatReporter;
use pretty::PrettyReporter;
use report::FileProblem;
pub use report::FileReport;
use report::{
    github::GithubReporter, gitlab::GitlabReporter, json::JsonReporter,
    json_lines::JsonLinesReporter, junit::JunitReporter, sarif::SarifReporter,
};
use tombi_diagnostic::Print;

/// Output format of diagnostics.
#[derive(clap::ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiagnosticsFormat {
    /// Human-readable output
    #[default]
    Pretty,

    /// JSON array of diagnostics
    Json,

    /// JSON Lines, a diagnostic per line
    #[value(alias = "jsonl")]
    JsonLines,

    /// GitHub Actions workflow commands
    Github,

    /// GitLab Code Quality report
    Gitlab,

    /// JUnit XML report
    Junit,

    /// SARIF 2.1.0 report
    Sarif,
}

impl DiagnosticsFormat {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pretty => "pretty",
            Self::Json => "json",
            Self::JsonLines => "json-lines",
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Junit => "junit",
            Self::Sarif => "sarif",
        }
    }

    /// Whether the output must not be mixed with other messages on stderr.
    const fn requires_diagnostics_file(&self) -> bool {
        matches!(
            self,
            Self::Json | Self::JsonLines | Self::Gitlab | Self::Junit | Self::Sarif
        )
    }

    fn reporter(&self, use_ansi_color: bool) -> std::io::Result<Box<dyn FormatReporter>> {
        Ok(match self {
            Self::Pretty => Box::new(PrettyReporter::new(use_ansi_color)),
            Self::Json => Box::<JsonReporter>::default(),
            Self::JsonLines => Box::new(JsonLinesReporter::new()?),
            Self::Github => Box::<GithubReporter>::default(),
            Self::Gitlab => Box::<GitlabReporter>::default(),
            Self::Junit => Box::<JunitReporter>::default(),
            Self::Sarif => Box::<SarifReporter>::default(),
        })
    }
}

#[derive(clap::Args, Debug)]
pub struct DiagnosticsArgs {
    /// Output format of diagnostics
    ///
    /// Formats other than "pretty" and "github" require `--diagnostics-file`.
    #[arg(long, value_enum, default_value_t = DiagnosticsFormat::Pretty)]
    diagnostics_format: DiagnosticsFormat,

    /// File to write diagnostics to
    ///
    /// On Unix, "/dev/stdout", "/dev/stderr" and "/dev/fd/N" write to the inherited file descriptor.
    ///
    /// [default: stderr]
    #[arg(long, value_name = "PATH")]
    diagnostics_file: Option<std::path::PathBuf>,
}

/// How the command reads its input, used to detect conflicting options.
pub struct InputContext {
    /// The subcommand, used for the usage of error messages.
    pub command: clap::Command,
    /// Reading from stdin without `--stdin-filename`, so that reports have no path.
    pub stdin_without_filename: bool,
    /// Stdout is used for other output, such as the formatted text of stdin.
    pub stdout_in_use: bool,
}

/// Writes diagnostics to the destination with the reporter of the format.
pub struct DiagnosticsReporter {
    reporter: Box<dyn FormatReporter>,
    writer: Box<dyn Write + Send>,
    write_error: Option<std::io::Error>,
}

impl DiagnosticsReporter {
    /// Validates the options and opens the destination.
    ///
    /// Exits with a usage error if the options conflict with each other.
    pub fn open(args: &DiagnosticsArgs, context: InputContext) -> std::io::Result<Self> {
        let format = args.diagnostics_format;
        let destination = Destination::new(args.diagnostics_file.as_deref());

        validate_option_combination(format, &destination, context);

        Ok(Self {
            reporter: format
                .reporter(destination.is_default_stderr() && crate::app::use_ansi_color())?,
            writer: destination.open()?,
            write_error: None,
        })
    }

    /// Records a checked file, with the error that made checking it fail, if any.
    ///
    /// The error is printed to stderr, unless the reporter reports it as a [`FileProblem`].
    pub fn record<P>(&mut self, mut file: FileReport, error: Option<&crate::Error>, printer: &P)
    where
        crate::Error: Print<P>,
    {
        file.problem = error.and_then(FileProblem::from_error);
        let reports_problem = file.problem.is_some() && self.reporter.reports_file_problems();

        if let Err(error) = self.reporter.record(file, &mut self.writer)
            && self.write_error.is_none()
        {
            self.write_error = Some(error);
        }

        if let Some(error) = error {
            self.record_failure(error, reports_problem, printer);
        }
    }

    /// Records an error that happened before a file was checked, such as an invalid glob pattern.
    pub fn record_error<P>(&mut self, error: &crate::Error, printer: &P)
    where
        crate::Error: Print<P>,
    {
        if let crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(path)) = error {
            let file = FileReport::new(Some(path.clone()), String::new(), Vec::new());
            self.record(file, Some(error), printer);
        } else {
            self.record_failure(error, false, printer);
        }
    }

    /// Records an error that is not a problem of the checked files, such as a panicked task.
    #[inline]
    pub fn record_runtime_error(&mut self) {
        self.reporter.record_runtime_error();
    }

    fn record_failure<P>(&mut self, error: &crate::Error, reports_problem: bool, printer: &P)
    where
        crate::Error: Print<P>,
    {
        if !matches!(
            error,
            crate::Error::NotFormatted(_)
                | crate::Error::FileParseFailed(_)
                | crate::Error::StdinParseFailed
        ) {
            self.record_runtime_error();
        }
        if !reports_problem {
            // Like `eprintln!`, but a failure to write to stderr is not a panic.
            let _ = error.print(printer, &mut std::io::stderr().lock());
        }
    }

    /// Writes what the reporter has collected and flushes the destination.
    pub fn finish(mut self) -> std::io::Result<()> {
        if let Some(error) = self.write_error.take() {
            return Err(error);
        }
        self.reporter.finish(&mut self.writer)?;
        self.writer.flush()
    }
}

/// Exits with a usage error if the format, the destination, and the input conflict with each other.
fn validate_option_combination(
    format: DiagnosticsFormat,
    destination: &Destination,
    context: InputContext,
) {
    use clap::error::ErrorKind;

    if format.requires_diagnostics_file() {
        if destination.is_default_stderr() {
            usage_error(
                context.command,
                ErrorKind::MissingRequiredArgument,
                format!(
                    "`--diagnostics-format {}` requires `--diagnostics-file`",
                    format.as_str()
                ),
            );
        }
        if destination.is_stderr() {
            usage_error(
                context.command,
                ErrorKind::ArgumentConflict,
                format!(
                    "`--diagnostics-format {}` cannot be written to stderr",
                    format.as_str()
                ),
            );
        }
    }

    if context.stdout_in_use && destination.is_stdout() {
        usage_error(
            context.command,
            ErrorKind::ArgumentConflict,
            "`--diagnostics-file` cannot be stdout when formatting stdin",
        );
    }

    if context.stdin_without_filename && format != DiagnosticsFormat::Pretty {
        usage_error(
            context.command,
            ErrorKind::MissingRequiredArgument,
            format!(
                "`--diagnostics-format {}` requires `--stdin-filename` when reading from stdin",
                format.as_str()
            ),
        );
    }
}

fn usage_error(
    command: clap::Command,
    kind: clap::error::ErrorKind,
    message: impl std::fmt::Display,
) -> ! {
    command
        .styles(crate::app::app_styles())
        .error(kind, message)
        .exit()
}
