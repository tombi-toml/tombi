use nu_ansi_term::{Color, Style};
use similar::{ChangeTag, TextDiff};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tombi_config::{FormatOptions, TomlVersion};
use tombi_diagnostic::Print;
use tombi_glob::{FileInputType, FileSearch, FileSearchEntry};

use crate::app::{
    CommonArgs,
    diagnostics::{DiagnosticsArgs, DiagnosticsReporter, FileReport, InputContext},
};

/// Format TOML files.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// List of files or directories to format
    ///
    /// If the only argument is "-", the standard input will be used
    ///
    /// [default: if "tombi.toml" exists, format project directory, otherwise format current directory]
    files: Vec<String>,

    /// Check only and don't overwrite files.
    #[arg(long, default_value_t = false)]
    check: bool,

    /// Show format changes
    #[arg(long, default_value_t = false)]
    diff: bool,

    /// Filename to use when reading from stdin
    ///
    /// This is useful for determining which JSON Schema should be applied, for more rich formatting.
    #[arg(long)]
    stdin_filename: Option<String>,

    /// Quiet mode
    ///
    /// If `true`, the program will not print summary output messages.
    #[arg(long, default_value_t = false)]
    quiet: bool,

    #[command(flatten)]
    diagnostics: DiagnosticsArgs,

    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Debug, Default)]
struct FormatRunSummary {
    success_num: usize,
    not_needed_num: usize,
    skipped_num: usize,
    error_num: usize,
}

struct FormattedFile {
    /// `true` if the file was formatted, `false` if it did not need formatting.
    result: Result<bool, crate::Error>,
    report: FileReport,
}

impl FormattedFile {
    fn failed(source_path: Option<&std::path::Path>, error: crate::Error) -> Self {
        Self {
            result: Err(error),
            report: FileReport::new(source_path.map(ToOwned::to_owned), None, Vec::new()),
        }
    }
}

fn record_format_result<P>(
    FormattedFile { result, report }: FormattedFile,
    summary: &mut FormatRunSummary,
    printer: &P,
    diagnostics_reporter: &mut DiagnosticsReporter,
) where
    crate::Error: Print<P>,
{
    diagnostics_reporter.record(report, result.as_ref().err(), printer);
    match result {
        Ok(true) => summary.success_num += 1,
        Ok(false) => summary.not_needed_num += 1,
        Err(_) => summary.error_num += 1,
    }
}

fn record_task_result<P>(
    result: Result<FormattedFile, tokio::task::JoinError>,
    summary: &mut FormatRunSummary,
    printer: &P,
    diagnostics_reporter: &mut DiagnosticsReporter,
) where
    crate::Error: Print<P>,
{
    match result {
        Ok(result) => record_format_result(result, summary, printer, diagnostics_reporter),
        Err(err) => {
            log::error!("task failed {err}");
            diagnostics_reporter.record_runtime_error();
            summary.error_num += 1;
        }
    }
}

pub fn run(args: Args) -> Result<(), crate::Error> {
    let quiet = args.quiet;

    // Validate options before reading stdin, which is copied to stdout if the config fails to load.
    let is_stdin = FileInputType::from(args.files.as_ref()) == FileInputType::Stdin;
    let mut diagnostics_reporter = match DiagnosticsReporter::open(
        &args.diagnostics,
        InputContext {
            command: <Args as clap::Args>::augment_args(clap::Command::new("tombi format")),
            stdin_without_filename: is_stdin && args.stdin_filename.is_none(),
            stdout_in_use: is_stdin,
        },
    ) {
        Ok(diagnostics_reporter) => diagnostics_reporter,
        Err(error) => {
            log::error!("{}", error);
            std::process::exit(1);
        }
    };

    let FormatRunSummary {
        success_num,
        not_needed_num,
        skipped_num,
        error_num,
    } = match inner_run(args, crate::app::printer(), &mut diagnostics_reporter) {
        Ok(summary) => summary,
        Err(error) => diagnostics_reporter.exit_with_error(&error),
    };

    if let Err(error) = diagnostics_reporter.finish() {
        log::error!("failed to write diagnostics: {}", error);
        std::process::exit(1);
    }

    if !quiet {
        match (success_num, not_needed_num) {
            (0, 0) => {
                if error_num == 0 && skipped_num == 0 {
                    eprintln!("No files formatted")
                }
            }
            (success_num, not_needed_num) => {
                match success_num {
                    0 => {}
                    1 => eprintln!("1 file formatted"),
                    _ => eprintln!("{success_num} files formatted"),
                };
                match not_needed_num {
                    0 => {}
                    1 => eprintln!("1 file did not need formatting"),
                    _ => eprintln!("{not_needed_num} files did not need formatting"),
                }
            }
        };
        match skipped_num {
            0 => {}
            1 => eprintln!("1 file skipped"),
            _ => eprintln!("{skipped_num} files skipped"),
        };
        match error_num {
            0 => {}
            1 => eprintln!("1 file failed to be formatted"),
            _ => eprintln!("{error_num} files failed to be formatted"),
        };
    }

    if error_num > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn inner_run<P>(
    args: Args,
    printer: P,
    diagnostics_reporter: &mut DiagnosticsReporter,
) -> Result<FormatRunSummary, Box<dyn std::error::Error>>
where
    crate::Error: Print<P>,
{
    let (config, config_path, config_level) = serde_tombi::config::load_with_path_and_level(
        std::env::current_dir().ok(),
    )
    .inspect_err(|_| {
        if FileInputType::from(args.files.as_ref()) == FileInputType::Stdin
            && let Err(error) = std::io::copy(&mut std::io::stdin(), &mut std::io::stdout())
        {
            log::error!("failed to copy stdin to stdout: {}", error);
        }
    })?;

    let toml_version = config.toml_version.unwrap_or_default();
    let schema_options = config.schema.as_ref();
    let schema_store =
        tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
            offline: args.common.offline.then_some(true),
            strict: schema_options.and_then(|schema_options| schema_options.strict()),
            cache: Some(tombi_cache::Options {
                no_cache: args.common.no_cache.then_some(true),
                ..Default::default()
            }),
        });

    let runtime = super::runtime(FileInputType::from(args.files.as_ref()) == FileInputType::Stdin)
        .map_err(|error| format!("failed to create tokio runtime: {error}"))?;

    // The tasks convert the spans of the diagnostics before their sources are dropped.
    let encoding = diagnostics_reporter.encoding();

    runtime.block_on(async {
        // Run schema loading and file discovery concurrently
        let (schema_result, input) = tokio::join!(
            schema_store.load_config(&config, config_path.as_deref()),
            FileSearch::new(&args.files, &config, config_path.as_deref(), config_level,)
        );

        // Before `schema_result?`, so that an error does not write to an input file.
        diagnostics_reporter
            .reject_input_conflict(super::input_paths(&input, config_path.as_deref()));
        schema_result?;
        let total_num = input.len();
        let mut summary = FormatRunSummary::default();

        match input {
            FileSearch::Stdin => {
                log::debug!("formatting... stdin input");
                let stdin_path = args.stdin_filename.as_ref().map(std::path::PathBuf::from);

                // Get format options with override support
                let Some(format_options) = tombi_glob::get_format_options(
                    &config,
                    stdin_path.as_deref(),
                    config_path.as_deref(),
                ) else {
                    log::debug!("formatting disabled for stdin by override");
                    summary.not_needed_num += 1;
                    return Ok(summary);
                };

                let formatted = format_stdin(
                    FormatFile::from_stdin(stdin_path),
                    toml_version,
                    args.check,
                    args.diff,
                    &format_options,
                    &schema_store,
                    encoding,
                )
                .await;
                record_format_result(formatted, &mut summary, &printer, diagnostics_reporter);
            }
            FileSearch::Files(files) => {
                let mut tasks = tokio::task::JoinSet::new();
                let concurrency = super::max_concurrency();

                for file in files {
                    match file {
                        FileSearchEntry::Found(source_path) => {
                            log::debug!("formatting... {:?}", source_path);

                            // Get format options with override support
                            let Some(format_options) = tombi_glob::get_format_options(
                                &config,
                                Some(source_path.as_ref()),
                                config_path.as_deref(),
                            ) else {
                                log::debug!(
                                    "formatting disabled for {:?} by override",
                                    source_path
                                );
                                summary.not_needed_num += 1;
                                continue;
                            };

                            while tasks.len() >= concurrency {
                                if let Some(result) = tasks.join_next().await {
                                    record_task_result(
                                        result,
                                        &mut summary,
                                        &printer,
                                        diagnostics_reporter,
                                    );
                                }
                            }

                            let schema_store = schema_store.clone();

                            tasks.spawn(async move {
                                match FormatFile::from_file(&source_path, args.check).await {
                                    Ok(file) => {
                                        format_file(
                                            file,
                                            &source_path,
                                            toml_version,
                                            args.check,
                                            args.diff,
                                            &format_options,
                                            &schema_store,
                                            encoding,
                                        )
                                        .await
                                    }
                                    Err(error) => FormattedFile::failed(
                                        Some(source_path.as_ref()),
                                        super::file_open_error(source_path.clone(), error),
                                    ),
                                }
                            });
                        }
                        FileSearchEntry::Skipped(_) => {
                            summary.skipped_num += 1;
                        }
                        FileSearchEntry::Error(err) => {
                            diagnostics_reporter
                                .record_error(&crate::Error::TombiGlob(err), &printer);
                            summary.error_num += 1;
                        }
                    }
                }

                while let Some(result) = tasks.join_next().await {
                    record_task_result(result, &mut summary, &printer, diagnostics_reporter);
                }
            }
        };

        debug_assert_eq!(
            summary.success_num + summary.not_needed_num + summary.skipped_num + summary.error_num,
            total_num
        );

        Ok(summary)
    })
}

// For standard input: --check outputs formatted TOML and returns error if different
async fn format_stdin(
    mut file: FormatFile,
    toml_version: TomlVersion,
    check: bool,
    diff: bool,
    format_options: &FormatOptions,
    schema_store: &tombi_schema_store::SchemaStore,
    encoding: tombi_text::EncodingKind,
) -> FormattedFile {
    let mut source = String::new();
    if let Err(error) = file.read_to_string(&mut source).await {
        return FormattedFile::failed(
            file.source(),
            crate::Error::read_failed(file.source(), error),
        );
    }

    let parsed = tombi_parser::parse(&source);
    let (result, diagnostics) = match tombi_formatter::Formatter::new(
        toml_version,
        format_options,
        file.source().map(itertools::Either::Right),
        schema_store,
    )
    .format_parsed(&parsed)
    .await
    {
        Ok(formatted) => {
            let has_diff = source != formatted;
            if has_diff && diff {
                log::info!("found format changes in stdin");
                eprint_diff(&source, &formatted);
            }
            let result = if check {
                if has_diff {
                    Err(crate::error::NotFormattedError::from(file.source()).into_error())
                } else {
                    Ok(false)
                }
            } else {
                print!("{formatted}");
                Ok(has_diff)
            };
            (result, Vec::new())
        }
        Err(diagnostics) => {
            print!("{source}");
            (Err(crate::Error::StdinParseFailed), diagnostics)
        }
    };

    FormattedFile {
        result,
        report: FileReport::new(
            file.source().map(ToOwned::to_owned),
            Some((parsed.line_index(), encoding)),
            diagnostics,
        ),
    }
}

async fn format_file(
    mut file: FormatFile,
    source_path: &std::path::Path,
    toml_version: TomlVersion,
    check: bool,
    diff: bool,
    format_options: &FormatOptions,
    schema_store: &tombi_schema_store::SchemaStore,
    encoding: tombi_text::EncodingKind,
) -> FormattedFile {
    let mut source = String::new();
    if let Err(error) = file.read_to_string(&mut source).await {
        return FormattedFile::failed(
            file.source(),
            crate::Error::read_failed(file.source(), error),
        );
    }

    let parsed = tombi_parser::parse(&source);
    let (result, diagnostics) = match tombi_formatter::Formatter::new(
        toml_version,
        format_options,
        Some(itertools::Either::Right(source_path)),
        schema_store,
    )
    .format_parsed(&parsed)
    .await
    {
        Ok(formatted) => {
            let result = if source != formatted {
                if diff {
                    log::info!("found format changes in {:?}", source_path);
                    eprint_diff(&source, &formatted);
                }
                if check {
                    Err(crate::error::NotFormattedError::from(file.source()).into_error())
                } else {
                    match file.write_formatted(formatted.as_bytes()).await {
                        Ok(_) => Ok(true),
                        Err(err) => Err(crate::Error::FileWriteFailed {
                            path: source_path.to_owned(),
                            source: err,
                        }),
                    }
                }
            } else {
                Ok(false)
            };
            (result, Vec::new())
        }
        Err(diagnostics) => (
            Err(crate::Error::FileParseFailed(source_path.to_owned())),
            diagnostics,
        ),
    };

    FormattedFile {
        result,
        report: FileReport::new(
            Some(source_path.to_owned()),
            Some((parsed.line_index(), encoding)),
            diagnostics,
        ),
    }
}

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn eprint_diff(source: &str, formatted: &str) {
    let diff = TextDiff::from_lines(source, formatted);
    const INDENT: &str = "        ";

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            eprintln!("{INDENT}{:-^1$}", "-", 89);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().fg(Color::Red)),
                    ChangeTag::Insert => ("+", Style::new().fg(Color::Green)),
                    ChangeTag::Equal => (" ", Style::new().fg(Color::White).dimmed()),
                };
                eprint!(
                    "{INDENT}{}{} |{}",
                    Style::new()
                        .fg(Color::White)
                        .dimmed()
                        .paint(format!("{}", Line(change.old_index()))),
                    Style::new()
                        .fg(Color::White)
                        .dimmed()
                        .paint(format!("{}", Line(change.new_index()))),
                    s.bold().paint(sign),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        eprint!("{}", s.underline().on(Color::Black).paint(value));
                    } else {
                        eprint!("{}", s.paint(value));
                    }
                }
                if change.missing_newline() {
                    eprintln!();
                }
            }
        }
    }
}

enum FormatFile {
    Stdin {
        stdin: tokio::io::Stdin,
        filename: Option<std::path::PathBuf>,
    },
    File {
        path: std::path::PathBuf,
        file: tokio::fs::File,
    },
}

impl FormatFile {
    fn from_stdin(filename: Option<std::path::PathBuf>) -> Self {
        Self::Stdin {
            stdin: tokio::io::stdin(),
            filename,
        }
    }

    async fn from_file(path: &std::path::Path, read_only: bool) -> std::io::Result<Self> {
        let mut options = tokio::fs::OpenOptions::new();
        options.read(true);
        if !read_only {
            options.write(true);
        }

        Ok(Self::File {
            path: path.to_owned(),
            file: options.open(path).await?,
        })
    }

    #[inline]
    fn source(&self) -> Option<&std::path::Path> {
        match self {
            Self::Stdin { filename, .. } => filename.as_deref(),
            Self::File { path, .. } => Some(path.as_ref()),
        }
    }

    async fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Self::Stdin { stdin, .. } => stdin.read_to_string(buf).await,
            Self::File { file, .. } => file.read_to_string(buf).await,
        }
    }

    /// Overwrites the file from the start and truncates it only after the write succeeded,
    /// so a failed write does not leave the file empty.
    async fn write_formatted(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Stdin { .. } => tokio::io::stdout().write_all(buf).await,
            Self::File { file, .. } => {
                file.seek(std::io::SeekFrom::Start(0)).await?;
                file.write_all(buf).await?;
                // tokio's `write_all` may return before the background write finishes,
                // so confirm it with `flush` before truncating.
                file.flush().await?;
                file.set_len(buf.len() as u64).await
            }
        }
    }
}
