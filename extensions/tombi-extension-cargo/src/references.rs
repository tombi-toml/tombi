use crate::{
    collect_feature_usage_locations, feature_key_at_accessors,
    feature_usage_target_for_feature_key, feature_usage_target_for_optional_dependency,
    optional_dependency_value_at_accessors, workspace_dependency_usage_locations,
};
use tombi_config::TomlVersion;
use tombi_schema_store::matches_accessors;

pub async fn references(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }
    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    if !cargo_references_enabled(features) {
        return Ok(None);
    }

    if let Some(target) = feature_key_at_accessors(document_tree, accessors)
        .and_then(|_| feature_usage_target_for_feature_key(&cargo_toml_path, accessors))
    {
        let locations =
            collect_feature_usage_locations(document_tree, &cargo_toml_path, &target, toml_version)
                .await
                .into_iter()
                .filter_map(|location| location.get_location())
                .collect::<Vec<_>>();

        return Ok((!locations.is_empty()).then_some(locations));
    }

    if optional_dependency_value_at_accessors(document_tree, accessors)
        .is_some_and(|optional| optional.value())
        && let Some(target) =
            feature_usage_target_for_optional_dependency(&cargo_toml_path, accessors)
    {
        let locations =
            collect_feature_usage_locations(document_tree, &cargo_toml_path, &target, toml_version)
                .await
                .into_iter()
                .filter_map(|location| location.get_location())
                .collect::<Vec<_>>();

        return Ok((!locations.is_empty()).then_some(locations));
    }

    if matches_accessors!(accessors, ["workspace", "dependencies", _]) {
        let locations = workspace_dependency_usage_locations(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
        )?;
        return Ok((!locations.is_empty()).then_some(locations));
    }

    Ok(None)
}

fn cargo_references_enabled(features: Option<&tombi_config::CargoExtensionFeatures>) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.references())
        .and_then(|references| references.dependency())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}
