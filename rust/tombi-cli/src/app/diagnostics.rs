mod destination;
mod format_reporter;
mod pretty;
mod report;

use std::io::Write;
use std::path::{Path, PathBuf};

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
    /// The regular file that diagnostics are written to.
    file_path: Option<PathBuf>,
    /// The subcommand, used for the usage of error messages.
    command: clap::Command,
    /// Whether the diagnostics file has been checked not to be an input of the command.
    inputs_checked: bool,
}

impl DiagnosticsReporter {
    /// Validates the options and opens the destination.
    ///
    /// Exits with a usage error if the options conflict with each other.
    pub fn open(args: &DiagnosticsArgs, context: InputContext) -> std::io::Result<Self> {
        let format = args.diagnostics_format;
        let destination = Destination::new(args.diagnostics_file.as_deref());

        validate_option_combination(format, &destination, &context);

        Ok(Self {
            reporter: format
                .reporter(destination.is_default_stderr() && crate::app::use_ansi_color())?,
            writer: destination.open()?,
            write_error: None,
            file_path: destination.file_path().map(Path::to_owned),
            command: context.command,
            inputs_checked: false,
        })
    }

    /// Exits with a usage error if the diagnostics file is also read by the command.
    ///
    /// The file is not truncated until something is written, so this must be called
    /// after the inputs are known and before the first file is recorded.
    pub fn reject_input_conflict<'a>(&mut self, inputs: impl IntoIterator<Item = &'a Path>) {
        if let Some(file_path) = &self.file_path
            && self.conflicts_with(inputs)
        {
            usage_error(
                self.command.clone(),
                clap::error::ErrorKind::ArgumentConflict,
                format!("`--diagnostics-file` {file_path:?} is also an input file of the command"),
            );
        }
        self.inputs_checked = true;
    }

    /// Whether the diagnostics file is one of the `inputs`, including via symlinks and relative paths.
    fn conflicts_with<'a>(&self, inputs: impl IntoIterator<Item = &'a Path>) -> bool {
        let Some(file) = self
            .file_path
            .as_deref()
            .and_then(|file_path| file_path.canonicalize().ok())
        else {
            return false;
        };
        inputs
            .into_iter()
            .any(|input| input.canonicalize().is_ok_and(|input| input == file))
    }

    /// Logs the error, writes what has been collected so that the report is complete, and exits.
    ///
    /// A diagnostics file is left untouched if the error happened before the inputs were checked,
    /// because it may be an input of the command.
    pub fn exit_with_error(mut self, error: &dyn std::fmt::Display) -> ! {
        log::error!("{error}");
        if self.file_path.is_none() || self.inputs_checked {
            self.record_runtime_error();
            if let Err(error) = self.finish() {
                log::error!("failed to write diagnostics: {error}");
            }
        }
        std::process::exit(1);
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
    context: &InputContext,
) {
    use clap::error::ErrorKind;

    if format.requires_diagnostics_file() {
        if destination.is_default_stderr() {
            usage_error(
                context.command.clone(),
                ErrorKind::MissingRequiredArgument,
                format!(
                    "`--diagnostics-format {}` requires `--diagnostics-file`",
                    format.as_str()
                ),
            );
        }
        if destination.is_stderr() {
            usage_error(
                context.command.clone(),
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
            context.command.clone(),
            ErrorKind::ArgumentConflict,
            "`--diagnostics-file` cannot be stdout when formatting stdin",
        );
    }

    if context.stdin_without_filename && format != DiagnosticsFormat::Pretty {
        usage_error(
            context.command.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Opens a reporter writing to `$file` in a directory that has `tombi.toml` and `input.toml`.
    macro_rules! test_conflicts_with {
        ($name:ident: $file:expr, $inputs:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let dir = tempfile::tempdir().unwrap();
                for name in ["tombi.toml", "input.toml", "other.toml"] {
                    std::fs::write(dir.path().join(name), "").unwrap();
                }
                let reporter = DiagnosticsReporter::open(
                    &DiagnosticsArgs {
                        diagnostics_format: DiagnosticsFormat::Json,
                        diagnostics_file: Some(dir.path().join($file)),
                    },
                    InputContext {
                        command: clap::Command::new("tombi lint"),
                        stdin_without_filename: false,
                        stdout_in_use: false,
                    },
                )
                .unwrap();

                let inputs = $inputs.map(|name: &str| dir.path().join(name));
                pretty_assertions::assert_eq!(
                    reporter.conflicts_with(inputs.iter().map(PathBuf::as_path)),
                    $expected
                );
            }
        };
    }

    test_conflicts_with!(input_file_conflicts: "input.toml", ["input.toml", "other.toml"] => true);
    test_conflicts_with!(config_file_conflicts: "tombi.toml", ["tombi.toml", "input.toml"] => true);
    test_conflicts_with!(path_with_dot_conflicts: "./input.toml", ["input.toml"] => true);
    test_conflicts_with!(other_file_does_not_conflict: "report.json", ["input.toml", "tombi.toml"] => false);
    test_conflicts_with!(no_inputs_do_not_conflict: "input.toml", [] => false);

    #[cfg(unix)]
    #[test]
    fn symlink_to_input_conflicts() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("input.toml"), "").unwrap();
        std::os::unix::fs::symlink("input.toml", dir.path().join("report.json")).unwrap();
        let reporter = DiagnosticsReporter::open(
            &DiagnosticsArgs {
                diagnostics_format: DiagnosticsFormat::Json,
                diagnostics_file: Some(dir.path().join("report.json")),
            },
            InputContext {
                command: clap::Command::new("tombi lint"),
                stdin_without_filename: false,
                stdout_in_use: false,
            },
        )
        .unwrap();

        assert!(reporter.conflicts_with([dir.path().join("input.toml").as_path()]));
    }
}
