use tokio::io::AsyncReadExt;
use tombi_config::{LintOptions, TomlVersion};
use tombi_diagnostic::{Diagnostic, Print};
use tombi_glob::FileSearch;

use crate::app::CommonArgs;

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

    #[command(flatten)]
    common: CommonArgs,
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run(args: Args) -> Result<(), crate::Error> {
    let (success_num, error_num) = match inner_run(args, crate::app::printer()) {
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
        _ => eprintln!("{success_num} files linted successfully"),
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

fn inner_run<P>(args: Args, mut printer: P) -> Result<(usize, usize), Box<dyn std::error::Error>>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Clone + Send + 'static,
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

    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        tracing::error!("Failed to create tokio runtime");
        std::process::exit(1);
    };

    runtime.block_on(async {
        // Run schema loading and file discovery concurrently
        let (schema_result, input) = tokio::join!(
            schema_store.load_config(&config, config_path.as_deref()),
            tombi_glob::FileSearch::new(&args.files, &config, config_path.as_deref(), config_level)
        );

        schema_result?;
        let total_num = input.len();
        let mut success_num = 0;
        let mut error_num = 0;

        match input {
            FileSearch::Stdin => {
                tracing::debug!("linting... stdin input");
                let stdin_path = args.stdin_filename.as_deref().map(std::path::Path::new);

                // Get lint options with override support
                let Some(lint_options) =
                    tombi_glob::get_lint_options(&config, stdin_path, config_path.as_deref())
                else {
                    tracing::debug!("Linting disabled for stdin by override");
                    success_num += 1;
                    return Ok((success_num, error_num));
                };

                if lint_file(
                    tokio::io::stdin(),
                    printer,
                    stdin_path,
                    toml_version,
                    &lint_options,
                    &schema_store,
                    args.error_on_warnings,
                )
                .await
                {
                    success_num += 1;
                } else {
                    error_num += 1;
                }
            }
            FileSearch::Files(files) => {
                let mut tasks = tokio::task::JoinSet::new();

                for file in files {
                    match file {
                        Ok(source_path) => {
                            tracing::debug!("linting... {:?}", source_path);

                            // Get lint options with override support
                            let Some(lint_options) = tombi_glob::get_lint_options(
                                &config,
                                Some(source_path.as_ref()),
                                config_path.as_deref(),
                            ) else {
                                tracing::debug!(
                                    "Linting disabled for {:?} by override",
                                    source_path
                                );
                                success_num += 1;
                                continue;
                            };

                            match tokio::fs::File::open(&source_path).await {
                                Ok(file) => {
                                    let printer = printer.clone();
                                    let schema_store = schema_store.clone();

                                    tasks.spawn(async move {
                                        lint_file(
                                            file,
                                            printer,
                                            Some(source_path.as_ref()),
                                            toml_version,
                                            &lint_options,
                                            &schema_store,
                                            args.error_on_warnings,
                                        )
                                        .await
                                    });
                                }
                                Err(err) => {
                                    if err.kind() == std::io::ErrorKind::NotFound {
                                        crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(
                                            source_path,
                                        ))
                                        .print(&mut printer);
                                    } else {
                                        crate::Error::Io(err).print(&mut printer);
                                    }
                                    error_num += 1;
                                }
                            }
                        }
                        Err(err) => {
                            crate::Error::TombiGlob(err).print(&mut printer);
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
                            tracing::error!("Task failed {}", e);
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
    error_on_warnings: bool,
) -> bool
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Send,
    R: AsyncReadExt + Unpin + Send,
{
    let mut source = String::new();
    if reader.read_to_string(&mut source).await.is_err() {
        return false;
    }
    let Err(diagnostics) = tombi_linter::Linter::new(
        toml_version,
        lint_options,
        source_path.map(itertools::Either::Right),
        schema_store,
    )
    .lint(&source)
    .await
    else {
        return true;
    };

    let diagnostics = if let Some(source_path) = source_path {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_file(source_path))
            .collect()
    } else {
        diagnostics
    };

    diagnostics.print(&mut printer);

    if error_on_warnings {
        diagnostics.is_empty()
    } else {
        diagnostics.iter().all(Diagnostic::is_warning)
    }
}
