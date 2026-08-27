use tombi_config::TomlVersion;

use crate::workspace::{is_nagi_config, workspace_source_reference_locations};

pub async fn references(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::NagiSqlExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    if !is_nagi_config(text_document_uri) || !references_enabled(features) {
        return Ok(None);
    }

    let locations = workspace_source_reference_locations(
        text_document_uri,
        document_tree,
        accessors,
        toml_version,
    );
    Ok((!locations.is_empty()).then_some(locations))
}

pub fn references_enabled(features: Option<&tombi_config::NagiSqlExtensionFeatures>) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.references())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}
