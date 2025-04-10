use config::{FormatOptions, TomlVersion};
use diagnostic::{printer::Pretty, Diagnostic, Print};
use formatter::formatter::definitions::FormatDefinitions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::app::arg;

/// Format TOML files.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Paths or glob patterns to TOML documents.
    ///
    /// If the only argument is "-", the standard input will be used.
    files: Vec<String>,

    /// Check only and don't overwrite files.
    #[arg(long, default_value_t = false)]
    check: bool,
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run(args: Args, offline: bool) -> Result<(), crate::Error> {
    let (success_num, not_needed_num, error_num) = match inner_run(args, Pretty, offline) {
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
    printer: P,
    offline: bool,
) -> Result<(usize, usize, usize), Box<dyn std::error::Error>>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Copy + Send + 'static,
{
    let (config, config_path) = serde_tombi::config::load_with_path()?;

    let toml_version = config.toml_version.unwrap_or_default();
    let schema_options = config.schema.as_ref();
    let schema_store = schema_store::SchemaStore::new_with_options(schema_store::Options {
        offline: offline.then_some(true),
        strict: schema_options.and_then(|schema_options| schema_options.strict()),
    });

    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        tracing::error!("Failed to create tokio runtime");
        std::process::exit(1);
    };

    runtime.block_on(async {
        schema_store
            .load_config(&config, config_path.as_deref())
            .await?;

        let include_patterns: Option<Vec<&str>> = config
            .include
            .as_deref()
            .map(|p| p.iter().map(|s| s.as_str()).collect());
        let exclude_patterns: Option<Vec<&str>> = config
            .exclude
            .as_deref()
            .map(|p| p.iter().map(|s| s.as_str()).collect());
        let format_options = config.format.unwrap_or_default();

        let input = arg::FileInput::new(
            &args.files,
            include_patterns.as_ref().map(|v| &v[..]),
            exclude_patterns.as_ref().map(|v| &v[..]),
        );
        let total_num = input.len();
        let mut success_num = 0;
        let mut not_needed_num = 0;
        let mut error_num = 0;

        match input {
            arg::FileInput::Stdin => {
                tracing::debug!("formatting... stdin input");
                match format_file(
                    FormatFile::from_stdin(),
                    printer,
                    None,
                    toml_version,
                    args.check,
                    &format_options,
                    &schema_store,
                )
                .await
                {
                    Ok(true) => success_num += 1,
                    Ok(false) => not_needed_num += 1,
                    Err(_) => error_num += 1,
                }
            }
            arg::FileInput::Files(files) => {
                let mut tasks = tokio::task::JoinSet::new();

                for file in files {
                    match file {
                        Ok(source_path) => {
                            tracing::debug!("formatting... {:?}", &source_path);
                            match FormatFile::from_file(&source_path).await {
                                Ok(file) => {
                                    let format_options = format_options.clone();
                                    let schema_store = schema_store.clone();

                                    tasks.spawn(async move {
                                        format_file(
                                            file,
                                            printer,
                                            Some(&source_path),
                                            toml_version,
                                            args.check,
                                            &format_options,
                                            &schema_store,
                                        )
                                        .await
                                    });
                                }
                                Err(err) => {
                                    if err.kind() == std::io::ErrorKind::NotFound {
                                        crate::Error::FileNotFound(source_path).print(printer);
                                    } else {
                                        crate::Error::Io(err).print(printer);
                                    }
                                    error_num += 1;
                                }
                            }
                        }
                        Err(err) => {
                            err.print(printer);
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
                        Ok(Err(_)) => {
                            error_num += 1;
                        }
                        Err(e) => {
                            tracing::error!("task failed {}", e);
                            error_num += 1;
                        }
                    }
                }
            }
        };

        assert_eq!(success_num + not_needed_num + error_num, total_num);

        Ok((success_num, not_needed_num, error_num))
    })
}

async fn format_file<P>(
    mut file: FormatFile,
    printer: P,
    source_path: Option<&std::path::Path>,
    toml_version: TomlVersion,
    check: bool,
    format_options: &FormatOptions,
    schema_store: &schema_store::SchemaStore,
) -> Result<bool, ()>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Copy,
{
    let mut source = String::new();
    if file.read_to_string(&mut source).await.is_ok() {
        match formatter::Formatter::new(
            toml_version,
            FormatDefinitions::default(),
            format_options,
            source_path.map(itertools::Either::Right),
            schema_store,
        )
        .format(&source)
        .await
        {
            Ok(formatted) => {
                if source != formatted {
                    if check {
                        crate::error::NotFormattedError::from(file.source())
                            .into_error()
                            .print(printer);
                    } else {
                        if let Err(err) = file.reset().await {
                            crate::Error::Io(err).print(printer);
                            return Err(());
                        }
                        match file.write_all(formatted.as_bytes()).await {
                            Ok(_) => return Ok(true),
                            Err(err) => {
                                crate::Error::Io(err).print(printer);
                            }
                        }
                    }
                } else {
                    return Ok(false);
                }
            }
            Err(diagnostics) => if let Some(source_path) = source_path {
                diagnostics
                    .into_iter()
                    .map(|diagnostic| diagnostic.with_source_file(source_path))
                    .collect()
            } else {
                diagnostics
            }
            .print(printer),
        }
    }
    Err(())
}

enum FormatFile {
    Stdin(tokio::io::Stdin),
    File {
        path: std::path::PathBuf,
        file: tokio::fs::File,
    },
}

impl FormatFile {
    fn from_stdin() -> Self {
        Self::Stdin(tokio::io::stdin())
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
            Self::Stdin(_) => None,
            Self::File { path, .. } => Some(path.as_ref()),
        }
    }

    async fn reset(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdin(_) => Ok(()),
            Self::File { file, .. } => {
                file.seek(std::io::SeekFrom::Start(0)).await?;
                file.set_len(0).await
            }
        }
    }

    async fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Self::Stdin(stdin) => stdin.read_to_string(buf).await,
            Self::File { file, .. } => file.read_to_string(buf).await,
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Stdin(_) => tokio::io::stdout().write_all(buf).await,
            Self::File { file, .. } => file.write_all(buf).await,
        }
    }
}
