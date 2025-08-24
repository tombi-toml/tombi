use itertools::Either;
use tombi_config::TomlVersion;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_toml_version(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetTomlVersionResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_toml_version");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;
    let text_document_uri = uri.into();

    let ConfigSchemaStore {
        config,
        schema_store,
        ..
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let root_ast = backend.get_incomplete_ast(&text_document_uri).await;

    let source_schema = match &root_ast {
        Some(root) => schema_store
            .resolve_source_schema_from_ast(root, Some(Either::Left(&text_document_uri)))
            .await
            .ok()
            .flatten(),
        None => None,
    };

    let tombi_document_comment_directive = match root_ast.as_ref() {
        Some(root) => {
            tombi_validator::comment_directive::get_tombi_document_comment_directive(root).await
        }
        None => None,
    };

    let (toml_version, source) = backend
        .source_toml_version(
            tombi_document_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    Ok(GetTomlVersionResponse {
        toml_version,
        source,
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTomlVersionResponse {
    pub toml_version: TomlVersion,
    pub source: TomlVersionSource,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TomlVersionSource {
    /// Comment directive
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    /// ```
    Comment,

    /// Schema directive
    ///
    /// ```toml
    /// #:schema "https://example.com/schema.json"
    /// ```
    Schema,

    /// Config file
    ///
    /// ```toml
    /// [tombi]
    /// toml-version = "v1.0.0"
    /// ```
    Config,

    /// Default
    Default,
}
