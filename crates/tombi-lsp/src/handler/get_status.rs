use std::path::PathBuf;

use itertools::Either;
use tombi_config::TomlVersion;
use tombi_glob::{matches_file_patterns, MatchResult};
use tombi_uri::url_to_file_path;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{backend::Backend, config_manager::ConfigSchemaStore, handler::TomlVersionSource};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_status(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetStatusResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_status");
    tracing::trace!(?params);

    let TextDocumentIdentifier {
        uri: text_document_uri,
    } = params;

    let ConfigSchemaStore {
        config,
        schema_store,
        config_path,
    } = backend
        .config_manager
        .config_schema_store_for_url(&text_document_uri)
        .await;

    let (root, source_schema) = match backend.get_incomplete_ast(&text_document_uri).await {
        Some(root) => {
            let source_schema = schema_store
                .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document_uri)))
                .await
                .ok()
                .flatten();
            (Some(root), source_schema)
        }
        None => (None, None),
    };

    let document_tombi_comment_directive = match root.as_ref() {
        Some(root) => tombi_comment_directive::get_document_tombi_comment_directive(root).await,
        None => None,
    };

    let (toml_version, source) = backend
        .source_toml_version(
            document_tombi_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let mut ignore = None;
    if let Ok(text_document_path) = url_to_file_path(&text_document_uri) {
        ignore = match matches_file_patterns(&text_document_path, config_path.as_deref(), &config) {
            MatchResult::IncludeNotMatched => Some(IgnoreReason::IncludeNotMatched),
            MatchResult::ExcludeMatched => Some(IgnoreReason::ExcludeMatched),
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
#[serde(rename_all = "camelCase")]
pub enum IgnoreReason {
    IncludeNotMatched,
    ExcludeMatched,
}
