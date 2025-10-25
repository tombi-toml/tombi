use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl};
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_accessors, dig_keys, Value};
use tombi_schema_store::matches_accessors;

use crate::{
    find_member_project_toml, find_workspace_pyproject_toml,
    goto_definition::{
        collect_workspace_project_dependency_definitions, get_path_dependency_definition,
    },
    goto_definition_for_member_pyproject_toml, goto_definition_for_workspace_pyproject_toml,
};

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is pyproject.toml
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(Default::default());
    }
    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(Default::default());
    };

    let locations = if matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "sources"]
    ) {
        if matches_accessors!(accessors, ["tool", "uv", "sources", _, "path"]) {
            goto_declaration_for_sources_path(document_tree, accessors, &pyproject_toml_path)?
        } else {
            goto_definition_for_member_pyproject_toml(
                document_tree,
                accessors,
                &pyproject_toml_path,
                toml_version,
                false,
            )?
        }
    } else if matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "workspace"]
    ) {
        goto_definition_for_workspace_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
    } else if matches_accessors!(accessors, ["project", "dependencies", _])
        || matches_accessors!(accessors, ["project", "optional-dependencies", _, _])
        || matches_accessors!(accessors, ["dependency-groups", _, _])
    {
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
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    if dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
        return Ok(Vec::with_capacity(0));
    }

    // Get the dependency string from the current position
    let Some((_, Value::String(dependency))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::with_capacity(0));
    };

    // Parse the PEP 508 requirement to extract package name
    let requirement = match Requirement::<VerbatimUrl>::from_str(dependency.value()) {
        Ok(requirement) => requirement,
        Err(_) => return Ok(Vec::with_capacity(0)),
    };
    let package_name = requirement.name.as_ref();

    let mut locations = Vec::new();

    if let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"]) {
        if let Some((_, Value::Table(source_table))) = sources.get_key_value(package_name) {
            if let Some((_, Value::Boolean(is_workspace))) = source_table.get_key_value("workspace")
            {
                if is_workspace.value() {
                    if let Some(location) = get_workspace_dependency_declaration(
                        package_name,
                        pyproject_toml_path,
                        toml_version,
                    ) {
                        locations.push(location);
                    }
                }
            }

            if let Some((_, Value::String(path))) = source_table.get_key_value("path") {
                if let Some(location) = get_path_dependency_declaration(pyproject_toml_path, path) {
                    locations.push(location);
                } else if let Some(location) =
                    get_path_dependency_definition(path.value(), toml_version)
                {
                    locations.push(location);
                }
            }
        }
    }

    locations.extend(collect_workspace_project_dependency_definitions(
        package_name,
        pyproject_toml_path,
        toml_version,
    ));

    Ok(locations)
}

fn get_workspace_dependency_declaration(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    // Find the workspace pyproject.toml
    let (workspace_pyproject_toml_path, _, workspace_document_tree) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;

    // Find the member project
    let (_, member_range) = find_member_project_toml(
        package_name,
        &workspace_document_tree,
        &workspace_pyproject_toml_path,
        toml_version,
    )?;

    let Ok(workspace_pyproject_toml_uri) =
        tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path)
    else {
        return None;
    };

    Some(tombi_extension::DefinitionLocation {
        uri: workspace_pyproject_toml_uri,
        range: member_range,
    })
}

fn goto_declaration_for_sources_path(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    let Some((_, Value::String(path_value))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::with_capacity(0));
    };

    if let Some(location) = get_path_dependency_declaration(pyproject_toml_path, path_value) {
        return Ok(vec![location]);
    }

    Ok(Vec::with_capacity(0))
}

fn get_path_dependency_declaration(
    pyproject_toml_path: &std::path::Path,
    path_value: &tombi_document_tree::String,
) -> Option<tombi_extension::DefinitionLocation> {
    let uri = tombi_uri::Uri::from_file_path(pyproject_toml_path).ok()?;
    Some(tombi_extension::DefinitionLocation {
        uri,
        range: path_value.unquoted_range(),
    })
}
