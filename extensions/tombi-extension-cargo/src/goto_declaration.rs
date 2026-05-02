use crate::{
    CargoNavigationFeature, classify_cargo_navigation_feature, dependency_parent_accessors,
    feature_key_at_accessors, goto_workspace_managed_dependency_locations, is_optional_dependency,
};
use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tombi_schema_store::{Accessor, matches_accessors};

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

    if !cargo_goto_declaration_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = goto_workspace_managed_dependency_locations(
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

fn cargo_goto_declaration_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_declaration())
        .and_then(
            |goto_declaration| match classify_cargo_navigation_feature(accessors) {
                CargoNavigationFeature::Dependency => goto_declaration.dependency(),
                CargoNavigationFeature::Member => goto_declaration.member(),
                CargoNavigationFeature::Path => None,
            },
        )
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}

pub fn get_current_declaration(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    cargo_toml_uri: &tombi_uri::Uri,
) -> Option<tombi_extension::Location> {
    if !cargo_toml_uri.path().ends_with("Cargo.toml") {
        return None;
    }

    if matches_accessors!(accessors, ["features", _]) {
        let feature_key = feature_key_at_accessors(document_tree, accessors)?;
        return Some(tombi_extension::Location {
            uri: cargo_toml_uri.clone(),
            range: feature_key.unquoted_range(),
        });
    }

    if !is_optional_dependency(document_tree, accessors) {
        return None;
    }

    let parent_keys = dependency_parent_accessors(accessors)
        .iter()
        .map(Accessor::as_key)
        .collect::<Option<Vec<_>>>()?;

    let (dependency_key, _) = dig_keys(document_tree, &parent_keys)?;

    Some(tombi_extension::Location {
        uri: cargo_toml_uri.clone(),
        range: dependency_key.unquoted_range(),
    })
}
