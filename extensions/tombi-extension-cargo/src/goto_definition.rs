use crate::{
    CargoNavigationFeature, classify_cargo_navigation_feature, collect_feature_usage_locations,
    collect_feature_usage_locations_in_manifest, dependency_feature_string_context,
    feature_key_at_accessors, feature_table_string_at_accessors,
    feature_usage_target_for_feature_key, feature_usage_target_for_optional_dependency,
    goto_definition_for_crate_cargo_toml, goto_definition_for_workspace_cargo_toml,
    optional_dependency_value_at_accessors, resolve_dependency_feature_string,
    resolve_feature_table_string,
};
use tombi_config::TomlVersion;

pub async fn goto_definition(
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
    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(Default::default());
    };

    if !cargo_navigation_enabled(features, accessors) {
        return Ok(None);
    }

    if let Some(target) = feature_key_at_accessors(document_tree, accessors)
        .and_then(|_| feature_usage_target_for_feature_key(&cargo_toml_path, accessors))
    {
        let locations =
            collect_feature_usage_locations(document_tree, &cargo_toml_path, &target, toml_version)
                .await
                .into_iter()
                .filter_map(|location| location.definition_location())
                .collect::<Vec<_>>();

        if locations.is_empty() {
            return Ok(None);
        }

        return Ok(Some(locations));
    }

    if let Some(feature_string) = feature_table_string_at_accessors(document_tree, accessors)
        && let Some(location) = resolve_feature_table_string(
            document_tree,
            &cargo_toml_path,
            feature_string,
            toml_version,
        )
        && let Some(location) = location.definition_location()
    {
        return Ok(Some(vec![location]));
    }

    if let Some((feature_string, dependency_accessors)) =
        dependency_feature_string_context(document_tree, accessors)
        && let Some(location) = resolve_dependency_feature_string(
            document_tree,
            &cargo_toml_path,
            dependency_accessors.as_slice(),
            feature_string,
            toml_version,
        )
        && let Some(location) = location.definition_location()
    {
        return Ok(Some(vec![location]));
    }

    if optional_dependency_value_at_accessors(document_tree, accessors)
        .is_some_and(|optional| optional.value())
        && let Some(target) =
            feature_usage_target_for_optional_dependency(&cargo_toml_path, accessors)
    {
        // `optional = true` defines an implicit feature in the same manifest.
        // Keep goto-definition local; workspace-wide usage collection belongs to goto-declaration.
        let locations = collect_feature_usage_locations_in_manifest(
            document_tree,
            &cargo_toml_path,
            &target,
            toml_version,
        )
        .into_iter()
        .filter_map(|location| location.definition_location())
        .collect::<Vec<_>>();

        if locations.is_empty() {
            return Ok(None);
        }

        return Ok(Some(locations));
    }

    let locations = if accessors.first()
        == Some(&tombi_schema_store::Accessor::Key("workspace".to_string()))
    {
        itertools::concat([
            goto_definition_for_workspace_cargo_toml(
                document_tree,
                accessors,
                &cargo_toml_path,
                toml_version,
                true,
            )?,
            // For Root Package
            // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package
            goto_definition_for_crate_cargo_toml(
                document_tree,
                accessors,
                &cargo_toml_path,
                toml_version,
                true,
            )?,
        ])
    } else {
        goto_definition_for_crate_cargo_toml(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
            accessors.last() != Some(&tombi_schema_store::Accessor::Key("workspace".to_string())),
        )?
    };

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}

fn cargo_navigation_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_definition())
        .and_then(
            |goto_definition| match classify_cargo_navigation_feature(accessors) {
                CargoNavigationFeature::Dependency => goto_definition.dependency(),
                CargoNavigationFeature::Member => goto_definition.member(),
                CargoNavigationFeature::Path => goto_definition.path(),
            },
        )
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}
