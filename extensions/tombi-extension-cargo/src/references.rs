use crate::{
    collect_feature_usage_locations, dependency_package_name, feature_usage_target_for_feature_key,
    feature_usage_target_for_optional_dependency, get_workspace_cargo_toml_path,
    goto_workspace_member_crates, load_cargo_toml, load_workspace_cargo_toml,
    optional_dependency_value_at_accessors, workspace_dependency_usage_locations,
};
use tombi_config::TomlVersion;
use tombi_document_tree::{LikeString, Value, dig_accessors, dig_keys};
use tombi_schema_store::{Accessor, matches_accessors};

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

    if matches_accessors!(accessors, ["package", "name"]) {
        let locations = package_name_reference_locations(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
        )
        .await?;
        return Ok((!locations.is_empty()).then_some(locations));
    }

    if let Some(target) = feature_usage_target_for_feature_key(&cargo_toml_path, accessors) {
        let locations =
            collect_feature_usage_locations(document_tree, &cargo_toml_path, &target, toml_version)
                .await
                .into_iter()
                .filter_map(|location| location.get_location())
                .collect::<Vec<_>>();

        return Ok((!locations.is_empty()).then_some(locations));
    }

    if optional_dependency_value_at_accessors(document_tree, accessors).unwrap_or_default()
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

pub(crate) async fn package_name_reference_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    debug_assert!(matches_accessors!(accessors, ["package", "name"]));

    let Some((_, Value::String(package_name))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::new());
    };

    let Some((workspace_cargo_toml_path, workspace_document_tree)) = load_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
    )
    .await
    else {
        return Ok(Vec::new());
    };

    let mut locations = Vec::new();
    collect_workspace_dependency_references(
        &mut locations,
        &workspace_document_tree,
        &workspace_cargo_toml_path,
        package_name.value(),
    );

    for crate_location in goto_workspace_member_crates(
        &workspace_document_tree,
        &[],
        &workspace_cargo_toml_path,
        toml_version,
        "members",
    )? {
        let Some((_, crate_document_tree)) =
            load_cargo_toml(&crate_location.cargo_toml_path, toml_version)
        else {
            continue;
        };

        collect_member_dependency_references(
            &mut locations,
            &crate_document_tree,
            &crate_location.cargo_toml_path,
            package_name.value(),
        );
    }

    Ok(locations)
}

fn collect_workspace_dependency_references(
    locations: &mut Vec<tombi_extension::Location>,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &std::path::Path,
    package_name: &str,
) {
    let Some((_, Value::Table(workspace_dependencies))) =
        dig_keys(workspace_document_tree, &["workspace", "dependencies"])
    else {
        return;
    };

    let Ok(uri) = tombi_uri::Uri::from_file_path(workspace_cargo_toml_path) else {
        return;
    };

    for (dependency_key, dependency_value) in workspace_dependencies.key_values() {
        if dependency_package_name(dependency_key.value(), dependency_value) == package_name {
            locations.push(tombi_extension::Location {
                uri: uri.clone(),
                range: dependency_key.unquoted_range(),
            });
        }
    }
}

fn collect_member_dependency_references(
    locations: &mut Vec<tombi_extension::Location>,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    package_name: &str,
) {
    let Ok(uri) = tombi_uri::Uri::from_file_path(crate_cargo_toml_path) else {
        return;
    };

    for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, Value::Table(dependencies))) =
            dig_keys(crate_document_tree, &[dependency_kind])
        {
            collect_dependency_table_references(locations, &uri, dependencies, package_name);
        }
    }

    if let Some((_, Value::Table(targets))) = dig_keys(crate_document_tree, &["target"]) {
        for target_value in targets.values() {
            let Value::Table(target_table) = target_value else {
                continue;
            };
            for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some((_, Value::Table(dependencies))) =
                    target_table.get_key_value(dependency_kind)
                else {
                    continue;
                };
                collect_dependency_table_references(locations, &uri, dependencies, package_name);
            }
        }
    }
}

fn collect_dependency_table_references(
    locations: &mut Vec<tombi_extension::Location>,
    uri: &tombi_uri::Uri,
    dependencies: &tombi_document_tree::Table,
    package_name: &str,
) {
    for (dependency_key, dependency_value) in dependencies.key_values() {
        if dependency_package_name(dependency_key.value(), dependency_value) == package_name {
            locations.push(tombi_extension::Location {
                uri: uri.clone(),
                range: dependency_key.unquoted_range(),
            });
        }
    }
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
