use tombi_config::TomlVersion;

use crate::workspace::{is_nagi_config, workspace_navigation};

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::NagiSqlExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    if !is_nagi_config(text_document_uri)
        || !features
            .and_then(|features| features.lsp())
            .and_then(|lsp| lsp.goto_declaration())
            .map(|feature| feature.enabled())
            .unwrap_or_default()
            .value()
    {
        return Ok(None);
    }

    let locations = workspace_navigation(text_document_uri, document_tree, accessors, toml_version);
    Ok((!locations.is_empty()).then_some(locations))
}
