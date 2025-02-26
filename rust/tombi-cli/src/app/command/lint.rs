use config::{LintOptions, TomlVersion};
use diagnostic::{printer::Pretty, Diagnostic, Print};
use schema_store::json::CatalogUrl;
use tokio::io::AsyncReadExt;

use crate::app::arg;

/// Lint TOML files.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Paths or glob patterns to TOML documents.
    ///
    /// If the only argument is "-", the standard input is used.
    files: Vec<String>,
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run(args: Args) -> Result<(), crate::Error> {
    let (success_num, error_num) = match inner_run(args, Pretty) {
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
        1 => eprintln!("1 file linted"),
        _ => eprintln!("{} files linted", success_num),
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

fn inner_run<P>(args: Args, printer: P) -> Result<(usize, usize), Box<dyn std::error::Error>>
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Copy + Clone + Send + 'static,
{
    let (config, config_dirpath) = config::load_with_path()?;

    let toml_version = config.toml_version.unwrap_or_default();
    let include_patterns: Option<Vec<&str>> = config
        .include
        .as_deref()
        .map(|p| p.iter().map(|s| s.as_str()).collect());
    let exclude_patterns: Option<Vec<&str>> = config
        .exclude
        .as_deref()
        .map(|p| p.iter().map(|s| s.as_str()).collect());
    let lint_options = config.lint.unwrap_or_default();
    let schema_options = config.schema.unwrap_or_default();
    let schema_store = schema_store::SchemaStore::default();

    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        tracing::error!("Failed to create tokio runtime");
        std::process::exit(1);
    };

    runtime.block_on(async {
        if schema_options.enabled.unwrap_or_default().value() {
            schema_store
                .load_schemas(
                    match &config.schemas {
                        Some(schemas) => schemas,
                        None => &[],
                    },
                    config_dirpath.as_deref(),
                )
                .await;

            for catalog_path in schema_options.catalog_paths().unwrap_or_default().iter() {
                schema_store
                    .load_catalog_from_url(&CatalogUrl::new(catalog_path.try_into()?))
                    .await?;
            }
        }

        let input = arg::FileInput::new(
            &args.files,
            include_patterns.as_ref().map(|v| &v[..]),
            exclude_patterns.as_ref().map(|v| &v[..]),
        );
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

        assert_eq!(success_num + error_num, total_num);

        Ok((success_num, error_num))
    })
}

async fn lint_file<R, P>(
    mut reader: R,
    printer: P,
    source_path: Option<&std::path::Path>,
    toml_version: TomlVersion,
    lint_options: &LintOptions,
    schema_store: &schema_store::SchemaStore,
) -> bool
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Copy + Send,
    R: AsyncReadExt + Unpin + Send,
{
    let mut source = String::new();
    if reader.read_to_string(&mut source).await.is_ok() {
        match linter::Linter::try_new(
            toml_version,
            lint_options,
            source_path.map(itertools::Either::Right),
            schema_store,
        )
        .await
        {
            Ok(linter) => match linter.lint(&source).await {
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
                .print(printer),
            },
            Err(err) => {
                tracing::error!("Failed to create linter: {:?}", err);
            }
        }
    }
    false
}
