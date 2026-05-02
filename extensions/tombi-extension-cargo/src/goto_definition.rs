use crate::{
    CargoNavigationFeature, classify_cargo_navigation_feature, dependency_feature_string_context,
    dependency_parent_accessors, feature_table_string_at_accessors, find_workspace_cargo_toml,
    get_workspace_cargo_toml_path, goto_definition_for_workspace_cargo_toml,
    goto_workspace_managed_dependency_locations, is_dependency_accessor, is_feature_key_accessor,
    is_optional_dependency_accessor, is_package_name_accessor, is_workspace_definition_accessor,
    is_workspace_dependency_accessor, is_workspace_flag_accessor,
    is_workspace_managed_dependency_accessor, resolve_dependency_feature_string,
    resolve_feature_table_string,
};
use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_keys};
use tombi_schema_store::{Accessor, matches_accessors};

pub async fn goto_definition(
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
    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(Default::default());
    };

    if !cargo_goto_definition_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = if is_package_name_accessor(accessors) {
        goto_definition_for_package_name(document_tree, accessors, &text_document_uri)
    } else if is_feature_key_accessor(accessors) {
        goto_definition_for_feature_key(document_tree, accessors, &text_document_uri)
    } else if is_optional_dependency_accessor(accessors) {
        goto_definition_for_optional_dependency(document_tree, accessors, &text_document_uri)
    } else if let Some(feature_string) = feature_table_string_at_accessors(document_tree, accessors)
        && let Some(location) = resolve_feature_table_string(
            document_tree,
            &cargo_toml_path,
            feature_string,
            toml_version,
        )
        && let Some(location) = location.get_location()
    {
        vec![location]
    } else if is_workspace_definition_accessor(accessors) {
        goto_workspace_definition_locations(
            document_tree,
            accessors,
            &text_document_uri,
            &cargo_toml_path,
            toml_version,
        )?
    } else if let Some((feature_string, dependency_accessors)) =
        dependency_feature_string_context(document_tree, accessors)
        && let Some(location) = resolve_dependency_feature_string(
            document_tree,
            &cargo_toml_path,
            dependency_accessors.as_slice(),
            feature_string,
            toml_version,
        )
        && let Some(location) = location.get_location()
    {
        vec![location]
    } else if is_workspace_flag_accessor(accessors) {
        goto_workspace_managed_dependency_locations(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
            false,
        )?
    } else if is_workspace_managed_dependency_accessor(document_tree, accessors) {
        goto_workspace_dependency_locations(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
        )?
    } else {
        goto_dependency_definition_locations(
            document_tree,
            accessors,
            &cargo_toml_path,
            toml_version,
        )?
    };

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}

fn goto_definition_for_package_name(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
) -> Vec<tombi_extension::Location> {
    debug_assert!(matches_accessors!(accessors, ["package", "name"]));

    let Some((_, Value::String(package_name))) = dig_keys(document_tree, &["package", "name"])
    else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: package_name.unquoted_range(),
    }]
}

fn goto_definition_for_feature_key(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
) -> Vec<tombi_extension::Location> {
    debug_assert!(matches_accessors!(accessors, ["features", _]));

    let Some(feature_name) = accessors.get(1).and_then(Accessor::as_key) else {
        return Vec::with_capacity(0);
    };

    let Some((key, _)) = dig_keys(document_tree, &["features", feature_name]) else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: key.unquoted_range(),
    }]
}

fn goto_definition_for_optional_dependency(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
) -> Vec<tombi_extension::Location> {
    let dependency_accessors = dependency_parent_accessors(accessors);
    let Some(dependency_names) = dependency_accessors
        .iter()
        .map(Accessor::as_key)
        .collect::<Option<Vec<_>>>()
    else {
        return Vec::with_capacity(0);
    };

    let Some((dependency_key, _)) = dig_keys(document_tree, &dependency_names) else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: dependency_key.unquoted_range(),
    }]
}

fn goto_workspace_definition_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    let locations = goto_definition_for_workspace_cargo_toml(
        document_tree,
        accessors,
        cargo_toml_path,
        toml_version,
        true,
    )?;

    if !locations.is_empty() || !is_workspace_dependency_accessor(accessors) {
        return Ok(locations);
    }

    let Some(dependency_name) = accessors.get(2).and_then(Accessor::as_key) else {
        return Ok(Vec::new());
    };
    let Some((key, _)) = dig_keys(
        document_tree,
        &["workspace", "dependencies", dependency_name],
    ) else {
        return Ok(Vec::new());
    };

    Ok(vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: key.unquoted_range(),
    }])
}

fn goto_workspace_dependency_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    let resolved_target_locations = goto_dependency_definition_locations(
        document_tree,
        accessors,
        cargo_toml_path,
        toml_version,
    )?;

    if resolved_target_locations.is_empty() {
        return goto_workspace_managed_dependency_locations(
            document_tree,
            accessors,
            cargo_toml_path,
            toml_version,
            false,
        );
    }

    Ok(resolved_target_locations)
}

fn goto_dependency_definition_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    let mut locations = goto_workspace_managed_dependency_locations(
        document_tree,
        accessors,
        cargo_toml_path,
        toml_version,
        true,
    )?;

    if !is_dependency_accessor(accessors)
        && !is_workspace_managed_dependency_accessor(document_tree, accessors)
    {
        return Ok(locations);
    }

    let workspace_cargo_toml_path = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
    )
    .map(|(workspace_path, _, _)| workspace_path);

    locations.retain(|location| {
        let Ok(location_path) = location.uri.to_file_path() else {
            return true;
        };
        workspace_cargo_toml_path
            .as_ref()
            .is_none_or(|workspace_path| location_path != *workspace_path)
    });

    Ok(locations)
}

fn cargo_goto_definition_enabled(
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
