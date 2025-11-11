use itertools::Itertools;
use nu_ansi_term::{Color, Style};
use similar::{ChangeTag, TextDiff};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tombi_config::{FormatOptions, TomlVersion};
use tombi_diagnostic::{printer::Pretty, Diagnostic, Print};
use tombi_glob::{FileInputType, FileSearch};

use crate::app::CommonArgs;

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

    #[command(flatten)]
    common: CommonArgs,
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run(args: Args) -> Result<(), crate::Error> {
    let (success_num, not_needed_num, error_num) = match inner_run(args, Pretty) {
        Ok((success_num, not_needed_num, error_num)) => (success_num, not_needed_num, error_num),
        Err(error) => {
            tracing::error!("{}", error);
            std::process::exit(1);
        }
    };

    match (success_num, not_needed_num) {
        (0, 0) => {
            if error_num == 0 {
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
    match error_num {
        0 => {}
        1 => eprintln!("1 file failed to be formatted"),
        _ => eprintln!("{error_num} files failed to be formatted"),
    };

    if error_num > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn inner_run<P>(
    args: Args,
    mut printer: P,
) -> Result<(usize, usize, usize), Box<dyn std::error::Error>>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Clone + Send + 'static,
{
    let use_ansi_color = std::env::var("NO_COLOR").map_or(true, |v| v.is_empty());

    let (config, config_path, config_level) = serde_tombi::config::load_with_path_and_level(
        std::env::current_dir().ok(),
    )
    .inspect_err(|_| {
        if FileInputType::from(args.files.as_ref()) == FileInputType::Stdin {
            if let Err(error) = std::io::copy(&mut std::io::stdin(), &mut std::io::stdout()) {
                tracing::error!("Failed to copy stdin to stdout: {}", error);
            }
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

    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        tracing::error!("Failed to create tokio runtime");
        std::process::exit(1);
    };

    runtime.block_on(async {
        let format_options = config.format.clone().unwrap_or_default();

        // Run schema loading and file discovery concurrently
        let (schema_result, input) = tokio::join!(
            schema_store.load_config(&config, config_path.as_deref()),
            FileSearch::new(&args.files, &config, config_path.as_deref(), config_level,)
        );

        schema_result?;
        let total_num = input.len();
        let mut success_num = 0;
        let mut not_needed_num = 0;
        let mut error_num = 0;

        match input {
            FileSearch::Stdin => {
                tracing::debug!("Formatting... stdin input");
                match format_stdin(
                    FormatFile::from_stdin(args.stdin_filename.map(std::path::PathBuf::from)),
                    printer,
                    toml_version,
                    args.check,
                    args.diff,
                    &format_options,
                    &schema_store,
                    use_ansi_color,
                )
                .await
                {
                    Ok(true) => success_num += 1,
                    Ok(false) => not_needed_num += 1,
                    Err(_) => error_num += 1,
                }
            }
            FileSearch::Files(files) => {
                let mut tasks = tokio::task::JoinSet::new();
                let mut errors = Vec::new();
                for file in files {
                    match file {
                        Ok(source_path) => {
                            tracing::debug!("Formatting... {:?}", &source_path);
                            match FormatFile::from_file(&source_path).await {
                                Ok(file) => {
                                    let printer = printer.clone();
                                    let format_options = format_options.clone();
                                    let schema_store = schema_store.clone();

                                    tasks.spawn(async move {
                                        format_file(
                                            file,
                                            printer,
                                            &source_path,
                                            toml_version,
                                            args.check,
                                            args.diff,
                                            &format_options,
                                            &schema_store,
                                            use_ansi_color,
                                        )
                                        .await
                                    });
                                }
                                Err(err) => {
                                    if err.kind() == std::io::ErrorKind::NotFound {
                                        crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(
                                            source_path,
                                        ))
                                        .print(&mut printer, use_ansi_color);
                                    } else {
                                        crate::Error::Io(err).print(&mut printer, use_ansi_color);
                                    }
                                    error_num += 1;
                                }
                            }
                        }
                        Err(err) => {
                            crate::Error::TombiGlob(err).print(&mut printer, use_ansi_color);
                            error_num += 1;
                        }
                    }
                }

                while let Some(result) = tasks.join_next().await {
                    match result {
                        Ok(Ok(formatted)) => {
                            if formatted {
                                success_num += 1;
                            } else {
                                not_needed_num += 1;
                            }
                        }
                        Ok(Err(error)) => {
                            errors.push(error);
                            error_num += 1;
                        }
                        Err(e) => {
                            tracing::error!("Task failed {}", e);
                            error_num += 1;
                        }
                    }
                }

                if !errors.is_empty() {
                    for error in errors {
                        error.print(&mut printer, use_ansi_color);
                    }
                }
            }
        };

        debug_assert_eq!(success_num + not_needed_num + error_num, total_num);

        Ok((success_num, not_needed_num, error_num))
    })
}

// For standard input: --check outputs formatted TOML and returns error if different
async fn format_stdin<P>(
    mut file: FormatFile,
    mut printer: P,
    toml_version: TomlVersion,
    check: bool,
    diff: bool,
    format_options: &FormatOptions,
    schema_store: &tombi_schema_store::SchemaStore,
    use_ansi_color: bool,
) -> Result<bool, crate::Error>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
{
    let mut source = String::new();
    if let Err(err) = file.read_to_string(&mut source).await {
        return Err(crate::Error::Io(err));
    }
    match tombi_formatter::Formatter::new(
        toml_version,
        format_options,
        file.source().map(itertools::Either::Right),
        schema_store,
    )
    .format(&source)
    .await
    {
        Ok(formatted) => {
            let has_diff = source != formatted;
            if has_diff && diff {
                tracing::info!("Found format changes in stdin");
                eprint_diff(&source, &formatted);
            }
            if check {
                if has_diff {
                    Err(crate::error::NotFormattedError::from(file.source()).into_error())
                } else {
                    Ok(false)
                }
            } else {
                print!("{formatted}");
                Ok(has_diff)
            }
        }
        Err(diagnostics) => {
            print!("{source}");
            diagnostics.print(&mut printer, use_ansi_color);
            Err(crate::Error::StdinParseFailed)
        }
    }
}

async fn format_file<P>(
    mut file: FormatFile,
    mut printer: P,
    source_path: &std::path::Path,
    toml_version: TomlVersion,
    check: bool,
    diff: bool,
    format_options: &FormatOptions,
    schema_store: &tombi_schema_store::SchemaStore,
    use_ansi_color: bool,
) -> Result<bool, crate::Error>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
{
    let mut source = String::new();
    if let Err(err) = file.read_to_string(&mut source).await {
        return Err(crate::Error::Io(err));
    }
    match tombi_formatter::Formatter::new(
        toml_version,
        format_options,
        Some(itertools::Either::Right(source_path)),
        schema_store,
    )
    .format(&source)
    .await
    {
        Ok(formatted) => {
            if source != formatted {
                if diff {
                    tracing::info!("Found format changes in {:?}", source_path);
                    eprint_diff(&source, &formatted);
                }
                if check {
                    Err(crate::error::NotFormattedError::from(file.source()).into_error())
                } else if let Err(err) = file.reset().await {
                    Err(crate::Error::Io(err))
                } else {
                    match file.write_all(formatted.as_bytes()).await {
                        Ok(_) => Ok(true),
                        Err(err) => Err(crate::Error::Io(err)),
                    }
                }
            } else {
                Ok(false)
            }
        }
        Err(diagnostics) => {
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.with_source_file(source_path))
                .collect_vec()
                .print(&mut printer, use_ansi_color);
            Err(crate::Error::FileParseFailed(source_path.to_owned()))
        }
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

    async fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        Ok(Self::File {
            path: path.to_owned(),
            file: tokio::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .await?,
        })
    }

    #[inline]
    fn source(&self) -> Option<&std::path::Path> {
        match self {
            Self::Stdin { filename, .. } => filename.as_deref(),
            Self::File { path, .. } => Some(path.as_ref()),
        }
    }

    async fn reset(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdin { .. } => Ok(()),
            Self::File { file, .. } => {
                file.seek(std::io::SeekFrom::Start(0)).await?;
                file.set_len(0).await
            }
        }
    }

    async fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Self::Stdin { stdin, .. } => stdin.read_to_string(buf).await,
            Self::File { file, .. } => file.read_to_string(buf).await,
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Stdin { .. } => tokio::io::stdout().write_all(buf).await,
            Self::File { file, .. } => file.write_all(buf).await,
        }
    }
}
