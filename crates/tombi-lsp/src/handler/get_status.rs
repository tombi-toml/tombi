use std::path::PathBuf;

use itertools::{Either, Itertools};
use tombi_config::TomlVersion;
use tombi_glob::{MatchResult, matches_file_patterns};
use tombi_schema_store::{SchemaAccessors, SchemaContext};
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
        schema_store,
        ..
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let (toml_version, source, schema, sub_schemas) = {
        let document_sources = backend.document_sources.read().await;
        if let Some(document_source) = document_sources.get(&text_document_uri) {
            let (toml_version, source) = backend
                .text_document_toml_version_and_source(&text_document_uri, document_source.text())
                .await;

            let document_schema = schema_store
                .resolve_document_schema_from_ast(
                    document_source.ast(),
                    Some(Either::Left(&text_document_uri)),
                )
                .await
                .ok()
                .flatten();

            if let Some(document_schema) = document_schema.as_ref() {
                let schema_context = SchemaContext {
                    toml_version,
                    document_schema: Some(document_schema),
                    store: &schema_store,
                    strict: None,
                };

                // Run validation to resolve and register only the sub-schemas
                // that are actually referenced by the current document.
                let _ = tombi_validator::validate(
                    document_source.document_tree().clone(),
                    Some(document_schema),
                    &schema_context,
                )
                .await;
            }

            let (root_schema, sub_schemas) = match document_schema {
                Some(document_schema) => {
                    let root_schema = document_schema.value_schema.as_ref().map(|_| SchemaStatus {
                        uri: document_schema.schema_uri,
                    });

                    let sub_schemas = document_schema
                        .sub_schema_uri_map
                        .read()
                        .await
                        .iter()
                        .map(|(accessors, schema_uri)| SubSchemaStatus {
                            root: SchemaAccessors::from(accessors.clone()),
                            uri: schema_uri.clone(),
                        })
                        .collect_vec();

                    (
                        root_schema,
                        if sub_schemas.is_empty() {
                            None
                        } else {
                            Some(sub_schemas)
                        },
                    )
                }
                None => (None, None),
            };

            (toml_version, source, root_schema, sub_schemas)
        } else {
            (
                TomlVersion::default(),
                TomlVersionSource::Default,
                None,
                None,
            )
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
        sub_schemas,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_schemas: Option<Vec<SubSchemaStatus>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaStatus {
    pub uri: tombi_uri::SchemaUri,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSchemaStatus {
    pub root: SchemaAccessors,
    pub uri: tombi_uri::SchemaUri,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoreReason {
    IncludeFilePatternNotMatched,
    ExcludeFilePatternMatched,
}
