use crate::goto_declaration_for_crate_cargo_toml;
use tombi_config::TomlVersion;

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is Cargo.toml
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(Default::default());
    }
    let Some(cargo_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(Default::default());
    };

    if !cargo_navigation_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = goto_declaration_for_crate_cargo_toml(
        document_tree,
        accessors,
        &cargo_toml_path,
        toml_version,
        false,
    )?;

    if locations.is_empty() {
        return Ok(Default::default());
    }

    Ok(Some(locations))
}

fn cargo_navigation_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_declaration())
        .and_then(|goto_declaration| {
            if matches!(
                accessors.last(),
                Some(tombi_schema_store::Accessor::Key(key)) if key == "path"
            ) {
                goto_declaration.path()
            } else {
                goto_declaration.dependency()
            }
        })
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}
