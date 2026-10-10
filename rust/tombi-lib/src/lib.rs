//! Shared format/lint core for Tombi's wasm-lib, Python and Node.js bindings.
//!
//! [`format_async`]/[`lint_async`] are the shared core: they resolve config and
//! schemas the same way regardless of target, so `tombi-wasm`'s `lib` feature
//! calls them directly from its own async (wasm-bindgen-futures) executor.
//! `format_sync`/`lint_sync` additionally block on a Tokio runtime, for
//! synchronous callers such as the Python (`tombi-lib-python`) and Node.js
//! (`tombi-lib-node`) binding crates; they are only available on non-wasm
//! targets.

mod diagnostic;
mod error;
mod format;
mod lint;

pub use diagnostic::Diagnostic;
pub use error::Error;
pub use format::{FormatResult, format_async};
pub use lint::{LintResult, lint_async};

#[cfg(feature = "python")]
pub use diagnostic::{Position, Range};

#[cfg(not(target_family = "wasm"))]
pub use format::format_sync;
#[cfg(not(target_family = "wasm"))]
pub use lint::lint_sync;

/// An in-memory `tombi.toml` configuration.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum ConfigInput {
    /// The content of a config file at a given path.
    File {
        content: String,
        path: std::path::PathBuf,
    },
    /// The content of a virtual `tombi.toml`.
    Text(String),
}

/// Options shared by `format_sync`/[`format_async`] and `lint_sync`/[`lint_async`].
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub config: Option<ConfigInput>,
}

fn load_config(
    options: Options,
) -> Result<(tombi_config::Config, Option<std::path::PathBuf>), Error> {
    if let Some(config) = options.config {
        let (config_content, config_path) = match config {
            ConfigInput::File { content, path } => (content, path),
            ConfigInput::Text(content) => (
                content,
                std::path::PathBuf::from(tombi_config::TOMBI_TOML_FILENAME),
            ),
        };
        let config =
            serde_tombi::config::from_str(&config_content, &config_path).map_err(|error| {
                log::warn!("{error}");
                tombi_config::Error::ConfigFileParseFailed {
                    config_path: config_path.clone(),
                }
            })?;
        Ok((config, Some(config_path)))
    } else {
        Ok(serde_tombi::config::load_with_path(
            std::env::current_dir().ok(),
        )?)
    }
}

fn new_schema_store(config: &tombi_config::Config) -> tombi_schema_store::SchemaStore {
    let schema_options = config.schema.as_ref();
    // `offline`/`cache` are not yet exposed on `Options`: this issue only
    // establishes the shared core, and the disk cache/offline mode are only
    // meaningful for a real filesystem (the `native` feature), so exposing
    // them is deferred to the `python`/`node` binding work that actually
    // needs them (#2203/#2204).
    tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
        offline: None,
        strict: schema_options.and_then(|schema_options| schema_options.strict()),
        lint: schema_options.and_then(|schema_options| schema_options.lint.clone()),
        cache: None,
        trusted_hosts: None,
    })
}

#[cfg(not(target_family = "wasm"))]
fn runtime() -> Result<tokio::runtime::Runtime, Error> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?)
}

// Shared by format::tests/lint::tests, which each build their own module
// hierarchy and so can't see a private helper defined in the other's file.
#[cfg(all(test, not(target_family = "wasm")))]
mod test_util {
    use crate::{ConfigInput, Options};

    // `Options::default()` (no explicit config) would make `load_config` walk
    // up from the real process cwd and pick up this repository's own
    // `tombi.toml` (which enables a network schema catalog), so tests that
    // don't need schema resolution pass this instead, keeping them hermetic
    // and network-free.
    pub(crate) fn schema_disabled_options() -> Options {
        Options {
            config: Some(ConfigInput::Text("[schema]\nenabled = false\n".to_owned())),
        }
    }
}
