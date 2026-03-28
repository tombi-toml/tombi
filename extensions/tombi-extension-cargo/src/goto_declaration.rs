use crate::{
    CargoNavigationFeature, classify_cargo_navigation_feature, goto_definition_for_crate_cargo_toml,
};
use tombi_config::TomlVersion;
use tombi_schema_store::matches_accessors;

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
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

    let locations = if matches_accessors!(accessors[..accessors.len().min(1)], ["workspace"]) {
        vec![]
    } else {
        goto_definition_for_crate_cargo_toml(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
            false,
        )?
    };

    if locations.is_empty() {
        return Ok(Default::default());
    }

    Ok(Some(locations))
}

fn cargo_navigation_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    let feature = classify_cargo_navigation_feature(accessors);
    features.map_or(true, |features| match feature {
        CargoNavigationFeature::Dependency => features.goto_declaration_dependency_enabled(),
        CargoNavigationFeature::Member => features.goto_declaration_member_enabled(),
        CargoNavigationFeature::Path => features.goto_declaration_path_enabled(),
    })
}
