use std::path::PathBuf;

use tombi_config::TomlVersion;
use tombi_glob::{matches_file_patterns, MatchResult};
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{backend::Backend, config_manager::ConfigSchemaStore, handler::TomlVersionSource};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_status(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetStatusResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_status");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;
    let text_document_uri = uri.into();

    let ConfigSchemaStore {
        config,
        config_path,
        ..
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let (toml_version, source) = {
        let document_sources = backend.document_sources.read().await;
        if let Some(document_source) = document_sources.get(&text_document_uri) {
            backend
                .text_document_toml_version_and_source(&text_document_uri, document_source.text())
                .await
        } else {
            (TomlVersion::default(), TomlVersionSource::Default)
        }
    };

    let mut ignore = None;
    if let Ok(text_document_path) = text_document_uri.to_file_path() {
        ignore = match matches_file_patterns(&text_document_path, config_path.as_deref(), &config) {
            MatchResult::IncludeNotMatched => Some(IgnoreReason::IncludeFilePatternNotMatched),
            MatchResult::ExcludeMatched => Some(IgnoreReason::ExcludeFilePatternMatched),
            MatchResult::Matched => None,
        };
    }

    Ok(GetStatusResponse {
        toml_version,
        source,
        config_path,
        ignore,
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub toml_version: TomlVersion,
    pub source: TomlVersionSource,
    pub config_path: Option<PathBuf>,
    pub ignore: Option<IgnoreReason>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoreReason {
    IncludeFilePatternNotMatched,
    ExcludeFilePatternMatched,
}
