use tombi_config::TomlVersion;

use crate::workspace::{
    is_nagi_config, workspace_navigation, workspace_source_definition_location,
};

pub async fn goto_definition(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    converter: tombi_extension::SpanConverter<'_, '_>,
    features: Option<&tombi_config::NagiSqlExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    if !is_nagi_config(text_document_uri)
        || !features
            .and_then(|features| features.lsp())
            .and_then(|lsp| lsp.goto_definition())
            .map(|feature| feature.enabled())
            .unwrap_or_default()
            .value()
    {
        return Ok(None);
    }

    let locations = get_current_declaration(text_document_uri, document_tree, accessors, converter)
        .into_iter()
        .chain(workspace_navigation(
            text_document_uri,
            document_tree,
            accessors,
            toml_version,
            converter,
        ))
        .collect::<Vec<_>>();
    Ok((!locations.is_empty()).then_some(locations))
}

pub fn get_current_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    converter: tombi_extension::SpanConverter<'_, '_>,
) -> Option<tombi_extension::Location> {
    is_nagi_config(text_document_uri)
        .then(|| {
            workspace_source_definition_location(
                text_document_uri,
                document_tree,
                accessors,
                converter,
            )
        })
        .flatten()
}
