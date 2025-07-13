use tokio::io::AsyncReadExt;
use tombi_config::{LintOptions, TomlVersion};
use tombi_diagnostic::{printer::Pretty, Diagnostic, Print};

use crate::app::arg;

/// Lint TOML files.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// List of files or directories to lint
    ///
    /// If the only argument is "-", the standard input will be used
    ///
    /// [default: if "tombi.toml" exists, lint project directory, otherwise lint current directory]
    files: Vec<String>,
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run(args: Args, offline: bool, no_cache: bool) -> Result<(), crate::Error> {
    let (success_num, error_num) = match inner_run(args, Pretty, offline, no_cache) {
        Ok((success_num, error_num)) => (success_num, error_num),
        Err(error) => {
            tracing::error!("{}", error);
            std::process::exit(1);
        }
    };

    match success_num {
        0 => {
            if error_num == 0 {
                eprintln!("No files linted")
            }
        }
        1 => eprintln!("1 file linted successfully"),
        _ => eprintln!("{} files linted successfully", success_num),
    }

    match error_num {
        0 => {}
        1 => eprintln!("1 file failed to be linted"),
        _ => eprintln!("{error_num} files failed to be linted"),
    }

    if error_num > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn inner_run<P>(
    args: Args,
    mut printer: P,
    offline: bool,
    no_cache: bool,
) -> Result<(usize, usize), Box<dyn std::error::Error>>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Clone + Send + 'static,
{
    let (config, config_path, config_level) = serde_tombi::config::load_with_path_and_level()?;

    let toml_version = config.toml_version.unwrap_or_default();
    let schema_options = config.schema.as_ref();
    let schema_store =
        tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
            offline: offline.then_some(true),
            strict: schema_options.and_then(|schema_options| schema_options.strict()),
            cache: Some(tombi_cache::Options {
                no_cache: no_cache.then_some(true),
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
        let files_options = config.files.clone().unwrap_or_default();
        let lint_options = config.lint.clone().unwrap_or_default();

        // Run schema loading and file discovery concurrently
        let (schema_result, input) = tokio::join!(
            schema_store.load_config(&config, config_path.as_deref()),
            arg::FileInput::new(
                &args.files,
                config_path.as_deref(),
                config_level,
                files_options,
            )
        );

        schema_result?;
        let total_num = input.len();
        let mut success_num = 0;
        let mut error_num = 0;

        match input {
            arg::FileInput::Stdin => {
                tracing::debug!("linting... stdin input");
                if lint_file(
                    tokio::io::stdin(),
                    printer,
                    None,
                    toml_version,
                    &lint_options,
                    &schema_store,
                )
                .await
                {
                    success_num += 1;
                } else {
                    error_num += 1;
                }
            }
            arg::FileInput::Files(files) => {
                let mut tasks = tokio::task::JoinSet::new();

                for file in files {
                    match file {
                        Ok(source_path) => {
                            tracing::debug!("linting... {:?}", source_path);
                            match tokio::fs::File::open(&source_path).await {
                                Ok(file) => {
                                    let printer = printer.clone();
                                    let options = lint_options.clone();
                                    let schema_store = schema_store.clone();

                                    tasks.spawn(async move {
                                        lint_file(
                                            file,
                                            printer,
                                            Some(source_path.as_ref()),
                                            toml_version,
                                            &options,
                                            &schema_store,
                                        )
                                        .await
                                    });
                                }
                                Err(err) => {
                                    if err.kind() == std::io::ErrorKind::NotFound {
                                        crate::Error::FileNotFound(source_path).print(&mut printer);
                                    } else {
                                        crate::Error::Io(err).print(&mut printer);
                                    }
                                    error_num += 1;
                                }
                            }
                        }
                        Err(err) => {
                            err.print(&mut printer);
                            error_num += 1;
                        }
                    }
                }

                while let Some(result) = tasks.join_next().await {
                    match result {
                        Ok(success) => {
                            if success {
                                success_num += 1;
                            } else {
                                error_num += 1;
                            }
                        }
                        Err(e) => {
                            tracing::error!("task failed {}", e);
                            error_num += 1;
                        }
                    }
                }
            }
        }

        debug_assert_eq!(success_num + error_num, total_num);

        Ok((success_num, error_num))
    })
}

async fn lint_file<R, P>(
    mut reader: R,
    mut printer: P,
    source_path: Option<&std::path::Path>,
    toml_version: TomlVersion,
    lint_options: &LintOptions,
    schema_store: &tombi_schema_store::SchemaStore,
) -> bool
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Send,
    R: AsyncReadExt + Unpin + Send,
{
    let mut source = String::new();
    if reader.read_to_string(&mut source).await.is_ok() {
        match tombi_linter::Linter::new(
            toml_version,
            lint_options,
            source_path.map(itertools::Either::Right),
            schema_store,
        )
        .lint(&source)
        .await
        {
            Ok(()) => {
                return true;
            }
            Err(diagnostics) => if let Some(source_path) = source_path {
                diagnostics
                    .into_iter()
                    .map(|diagnostic| diagnostic.with_source_file(source_path))
                    .collect()
            } else {
                diagnostics
            }
            .print(&mut printer),
        }
    }
    false
}
