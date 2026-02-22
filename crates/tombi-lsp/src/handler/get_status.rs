use std::path::PathBuf;

use itertools::Either;
use tombi_config::TomlVersion;
use tombi_glob::{MatchResult, matches_file_patterns};
use tombi_uri::SchemaUri;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{backend::Backend, config_manager::ConfigSchemaStore, handler::TomlVersionSource};

pub async fn handle_get_status(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetStatusResponse, tower_lsp::jsonrpc::Error> {
    log::info!("handle_get_status");
    log::trace!("{:?}", params);

    let TextDocumentIdentifier { uri } = params;
    let text_document_uri = uri.into();

    let ConfigSchemaStore {
        config,
        config_path,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let (toml_version, source, schema) = {
        let document_sources = backend.document_sources.read().await;
        if let Some(document_source) = document_sources.get(&text_document_uri) {
            let (toml_version, source) = backend
                .text_document_toml_version_and_source(&text_document_uri, document_source.text())
                .await;

            // Get schema information
            let schema = {
                let root = document_source.ast();

                // Get schema URI from resolve_source_schema_from_ast
                // (internally checks comment directive first, then falls back to other methods)
                let schema_uri = match schema_store
                    .resolve_source_schema_from_ast(root, Some(Either::Left(&text_document_uri)))
                    .await
                {
                    Ok(Some(source_schema)) => source_schema
                        .root_schema
                        .as_ref()
                        .map(|s| s.schema_uri.clone()),
                    _ => None,
                };

                // If schema URI is available, get detailed information
                if let Some(schema_uri) = schema_uri {
                    // First, search in list_schemas
                    let schemas = schema_store.list_schemas().await;
                    if let Some(schema) = schemas.iter().find(|s| s.schema_uri == schema_uri) {
                        Some(SchemaStatus {
                            title: schema.title.clone(),
                            description: schema.description.clone(),
                            uri: schema_uri,
                        })
                    } else {
                        // If not found in list_schemas, get from DocumentSchema
                        match schema_store.try_get_document_schema(&schema_uri).await {
                            Ok(Some(doc_schema)) => {
                                // Get title/description from DocumentSchema
                                let (title, description) =
                                    if let Some(value_schema) = &doc_schema.value_schema {
                                        (
                                            value_schema.title().map(|s| s.to_string()),
                                            value_schema.description().map(|s| s.to_string()),
                                        )
                                    } else {
                                        (None, None)
                                    };
                                Some(SchemaStatus {
                                    title,
                                    description,
                                    uri: schema_uri,
                                })
                            }
                            _ => {
                                // Return URI only even if schema cannot be retrieved
                                Some(SchemaStatus {
                                    title: None,
                                    description: None,
                                    uri: schema_uri,
                                })
                            }
                        }
                    }
                } else {
                    None
                }
            };

            (toml_version, source, schema)
        } else {
            (TomlVersion::default(), TomlVersionSource::Default, None)
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
        schema,
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub toml_version: TomlVersion,
    pub source: TomlVersionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<IgnoreReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaStatus>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub uri: SchemaUri,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoreReason {
    IncludeFilePatternNotMatched,
    ExcludeFilePatternMatched,
}
