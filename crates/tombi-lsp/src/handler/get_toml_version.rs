use tombi_config::TomlVersion;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_toml_version(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetTomlVersionResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_toml_version");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;
    let text_document_uri = uri.into();

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
