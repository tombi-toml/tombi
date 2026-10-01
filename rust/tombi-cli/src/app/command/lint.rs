use tokio::io::AsyncReadExt;
use tombi_config::{LintOptions, TomlVersion};
use tombi_diagnostic::{Diagnostic, Print};
use tombi_glob::{FileInputType, FileSearch, FileSearchEntry};

use crate::app::{
    CommonArgs,
    diagnostics::{DiagnosticsArgs, DiagnosticsReporter, FileReport, InputContext},
};

/// Lint TOML files.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// List of files or directories to lint
    ///
    /// If the only argument is "-", the standard input will be used
    ///
    /// [default: if "tombi.toml" exists, lint project directory, otherwise lint current directory]
    files: Vec<String>,

    /// Filename to use when reading from stdin
    ///
    /// This is useful for determining which JSON Schema should be applied, for more rich linting.
    #[arg(long)]
    stdin_filename: Option<String>,

    /// Exit with error code on warnings
    ///
    /// If `true`, the program will exit with error code if there are warnings.
    #[arg(long, default_value_t = false)]
    error_on_warnings: bool,

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
struct LintRunSummary {
    success_num: usize,
    skipped_num: usize,
    error_num: usize,
}

struct LintedFile {
    /// `true` if the file has no errors (or no warnings with `--error-on-warnings`).
    result: Result<bool, crate::Error>,
    report: FileReport,
}

impl LintedFile {
    fn failed(source_path: Option<&std::path::Path>, error: crate::Error) -> Self {
        Self {
            result: Err(error),
            report: FileReport::new(source_path.map(ToOwned::to_owned), None, Vec::new()),
        }
    }
}

fn record_lint_result<P>(
    LintedFile { result, report }: LintedFile,
    summary: &mut LintRunSummary,
    printer: &P,
    diagnostics_reporter: &mut DiagnosticsReporter,
) where
    crate::Error: Print<P>,
{
    diagnostics_reporter.record(report, result.as_ref().err(), printer);
    match result {
        Ok(true) => summary.success_num += 1,
        Ok(false) | Err(_) => summary.error_num += 1,
    }
}

fn record_task_result<P>(
    result: Result<LintedFile, tokio::task::JoinError>,
    summary: &mut LintRunSummary,
    printer: &P,
    diagnostics_reporter: &mut DiagnosticsReporter,
) where
    crate::Error: Print<P>,
{
    match result {
        Ok(result) => record_lint_result(result, summary, printer, diagnostics_reporter),
        Err(err) => {
            log::error!("task failed {err}");
            diagnostics_reporter.record_runtime_error();
            summary.error_num += 1;
        }
    }
}

pub fn run(args: Args) -> Result<(), crate::Error> {
    let quiet = args.quiet;
    let is_stdin = FileInputType::from(args.files.as_ref()) == FileInputType::Stdin;
    let mut diagnostics_reporter = match DiagnosticsReporter::open(
        &args.diagnostics,
        InputContext {
            command: <Args as clap::Args>::augment_args(clap::Command::new("tombi lint")),
            stdin_without_filename: is_stdin && args.stdin_filename.is_none(),
            stdout_in_use: false,
        },
    ) {
        Ok(diagnostics_reporter) => diagnostics_reporter,
        Err(error) => {
            log::error!("{}", error);
            std::process::exit(1);
        }
    };

    let LintRunSummary {
        success_num,
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
        match success_num {
            0 => {
                if error_num == 0 && skipped_num == 0 {
                    eprintln!("No files linted")
                }
            }
            1 => eprintln!("1 file linted successfully"),
            _ => eprintln!("{success_num} files linted successfully"),
        }

        match skipped_num {
            0 => {}
            1 => eprintln!("1 file skipped"),
            _ => eprintln!("{skipped_num} files skipped"),
        }

        match error_num {
            0 => {}
            1 => eprintln!("1 file failed to be linted"),
            _ => eprintln!("{error_num} files failed to be linted"),
        }
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
) -> Result<LintRunSummary, Box<dyn std::error::Error>>
where
    crate::Error: Print<P>,
{
    let (config, config_path, config_level) =
        serde_tombi::config::load_with_path_and_level(std::env::current_dir().ok())?;

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
            tombi_glob::FileSearch::new(&args.files, &config, config_path.as_deref(), config_level)
        );

        // Before `schema_result?`, so that an error does not write to an input file.
        diagnostics_reporter
            .reject_input_conflict(super::input_paths(&input, config_path.as_deref()));
        schema_result?;
        let total_num = input.len();
        let mut summary = LintRunSummary::default();

        match input {
            FileSearch::Stdin => {
                log::debug!("linting... stdin input");
                let stdin_path = args.stdin_filename.as_deref().map(std::path::Path::new);

                // Get lint options with override support
                let Some(lint_options) =
                    tombi_glob::get_lint_options(&config, stdin_path, config_path.as_deref())
                else {
                    log::debug!("linting disabled for stdin by override");
                    summary.success_num += 1;
                    return Ok(summary);
                };

                let result = lint_file(
                    tokio::io::stdin(),
                    stdin_path,
                    toml_version,
                    &lint_options,
                    &schema_store,
                    args.error_on_warnings,
                    encoding,
                )
                .await;
                record_lint_result(result, &mut summary, &printer, diagnostics_reporter);
            }
            FileSearch::Files(files) => {
                let mut tasks = tokio::task::JoinSet::new();
                let concurrency = super::max_concurrency();

                for file in files {
                    match file {
                        FileSearchEntry::Found(source_path) => {
                            log::debug!("linting... {:?}", source_path);

                            // Get lint options with override support
                            let Some(lint_options) = tombi_glob::get_lint_options(
                                &config,
                                Some(source_path.as_ref()),
                                config_path.as_deref(),
                            ) else {
                                log::debug!("linting disabled for {:?} by override", source_path);
                                summary.success_num += 1;
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
                                match tokio::fs::File::open(&source_path).await {
                                    Ok(file) => {
                                        lint_file(
                                            file,
                                            Some(source_path.as_ref()),
                                            toml_version,
                                            &lint_options,
                                            &schema_store,
                                            args.error_on_warnings,
                                            encoding,
                                        )
                                        .await
                                    }
                                    Err(error) => LintedFile::failed(
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
        }

        debug_assert_eq!(
            summary.success_num + summary.skipped_num + summary.error_num,
            total_num
        );

        Ok(summary)
    })
}

async fn lint_file<R>(
    mut reader: R,
    source_path: Option<&std::path::Path>,
    toml_version: TomlVersion,
    lint_options: &LintOptions,
    schema_store: &tombi_schema_store::SchemaStore,
    error_on_warnings: bool,
    encoding: tombi_text::EncodingKind,
) -> LintedFile
where
    R: AsyncReadExt + Unpin + Send,
{
    let mut source = String::new();
    if let Err(error) = reader.read_to_string(&mut source).await {
        return LintedFile::failed(source_path, crate::Error::read_failed(source_path, error));
    }

    let parsed = tombi_parser::parse(&source);
    let diagnostics = tombi_linter::Linter::new(
        toml_version,
        lint_options,
        source_path.map(itertools::Either::Right),
        schema_store,
    )
    .lint_parsed(&parsed)
    .await
    .err()
    .unwrap_or_default();

    let success = if error_on_warnings {
        diagnostics.is_empty()
    } else {
        diagnostics.iter().all(Diagnostic::is_warning)
    };

    LintedFile {
        result: Ok(success),
        report: FileReport::new(
            source_path.map(ToOwned::to_owned),
            Some((parsed.line_index(), encoding)),
            diagnostics,
        ),
    }
}
