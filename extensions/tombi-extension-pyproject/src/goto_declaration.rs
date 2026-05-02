use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_schema_store::matches_accessors;

use crate::{
    PyprojectNavigationFeature, classify_pyproject_navigation_feature,
    collect_workspace_project_dependency_definitions, goto_definition_for_member_pyproject_toml,
    goto_definition_for_workspace_pyproject_toml, is_dependency_name_accessors,
    is_uv_source_workspace_accessors, is_uv_workspace_accessors, parse_requirement,
};

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is pyproject.toml
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(Default::default());
    }
    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(Default::default());
    };

    if !pyproject_goto_declaration_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = if is_uv_source_workspace_accessors(accessors) {
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            false,
        )?
    } else if is_uv_workspace_accessors(accessors) {
        goto_definition_for_workspace_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
    } else if is_dependency_name_accessors(accessors) {
        goto_declaration_for_dependency_package(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
    } else {
        Vec::with_capacity(0)
    };

    if locations.is_empty() {
        return Ok(Default::default());
    }

    Ok(Some(locations))
}

fn goto_declaration_for_dependency_package(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    let Some((_, Value::String(dep_str))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::with_capacity(0));
    };
    let Some(requirement) = parse_requirement(dep_str.value()) else {
        return Ok(Vec::with_capacity(0));
    };
    let package_name = requirement.name.as_ref();

    let mut locations = dependency_source_declaration_locations(
        document_tree,
        pyproject_toml_path,
        package_name,
        toml_version,
    )?;

    if locations.is_empty() && requirement.version_or_url.is_none() {
        locations.extend(collect_workspace_project_dependency_definitions(
            package_name,
            pyproject_toml_path,
            toml_version,
        ));
    }

    Ok(locations)
}

fn dependency_source_declaration_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    pyproject_toml_path: &std::path::Path,
    package_name: &str,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"])
    else {
        return Ok(Vec::with_capacity(0));
    };
    let Some((source_key, Value::Table(source_table))) = sources.get_key_value(package_name) else {
        return Ok(Vec::with_capacity(0));
    };

    if let Some((_, Value::Boolean(is_workspace))) = source_table.get_key_value("workspace")
        && is_workspace.value()
    {
        let accessors = [
            tombi_schema_store::Accessor::Key("tool".to_string()),
            tombi_schema_store::Accessor::Key("uv".to_string()),
            tombi_schema_store::Accessor::Key("sources".to_string()),
            tombi_schema_store::Accessor::Key(package_name.to_string()),
            tombi_schema_store::Accessor::Key("workspace".to_string()),
        ];
        return goto_definition_for_member_pyproject_toml(
            document_tree,
            &accessors,
            pyproject_toml_path,
            toml_version,
            false,
        );
    }

    let Ok(uri) = tombi_uri::Uri::from_file_path(pyproject_toml_path) else {
        return Ok(Vec::with_capacity(0));
    };

    Ok(vec![tombi_extension::Location {
        uri,
        range: source_key.unquoted_range(),
    }])
}

fn pyproject_goto_declaration_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_declaration())
        .and_then(
            |goto_declaration| match classify_pyproject_navigation_feature(accessors) {
                PyprojectNavigationFeature::Dependency => goto_declaration.dependency(),
                PyprojectNavigationFeature::Member => goto_declaration.member(),
                PyprojectNavigationFeature::Path => None,
            },
        )
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}

pub fn get_current_declaration(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_uri: &tombi_uri::Uri,
) -> Option<tombi_extension::Location> {
    if !pyproject_toml_uri.path().ends_with("pyproject.toml") {
        return None;
    }

    if !matches_accessors!(accessors, ["dependency-groups", _]) {
        return None;
    }

    let (group_key, _) = dig_keys(
        document_tree,
        &["dependency-groups", accessors.get(1)?.as_key()?],
    )?;

    Some(tombi_extension::Location {
        uri: pyproject_toml_uri.clone(),
        range: group_key.unquoted_range(),
    })
}
