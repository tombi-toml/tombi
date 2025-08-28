use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl};
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_accessors, dig_keys, Value};
use tombi_schema_store::{matches_accessors, Accessor};

use crate::{
    find_member_project_toml, find_workspace_pyproject_toml, get_project_name,
    goto_definition_for_member_pyproject_toml, goto_definition_for_workspace_pyproject_toml,
    load_pyproject_toml,
};

pub async fn goto_definition(
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
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            accessors.last() != Some(&tombi_schema_store::Accessor::Key("workspace".to_string())),
        )?
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
        // Handle dependencies in project.dependencies, project.optional-dependencies, or dependency-groups
        match goto_definition_for_dependency_package(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )? {
            Some(location) => vec![location],
            None => Vec::with_capacity(0),
        }
    } else {
        Vec::with_capacity(0)
    };

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}

fn goto_definition_for_dependency_package(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    // Get the dependency string from the current position
    let Some((_, Value::String(dependency))) = dig_accessors(document_tree, accessors) else {
        return Ok(None);
    };

    // Parse the PEP 508 requirement to extract package name
    let package_name = match Requirement::<VerbatimUrl>::from_str(dependency.value()) {
        Ok(requirement) => requirement.name,
        Err(_) => return Ok(None),
    };

    // Check if this package is in tool.uv.sources
    let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"])
    else {
        return Ok(None);
    };
    let Some((_, Value::Table(source_table))) = sources.get_key_value(package_name.as_ref()) else {
        return Ok(None);
    };
    if let Some((_, Value::Boolean(is_workspace))) = source_table.get_key_value("workspace") {
        if !is_workspace.value() {
            return Ok(None);
        }
        return Ok(get_workspace_dependency_definition(
            package_name.as_ref(),
            pyproject_toml_path,
            toml_version,
        ));
    }
    if let Some((_, Value::String(path))) = source_table.get_key_value("path") {
        return Ok(get_path_dependency_definition(path.value(), toml_version));
    }

    Ok(None)
}

fn get_workspace_dependency_definition(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    // Find the workspace pyproject.toml
    let (workspace_path, workspace_document_tree) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;

    // Find the member project
    let (member_pyproject_toml_path, _) = find_member_project_toml(
        package_name,
        &workspace_document_tree,
        &workspace_path,
        toml_version,
    )?;

    let member_document_tree = load_pyproject_toml(&member_pyproject_toml_path, toml_version)?;
    let package_name = get_project_name(&member_document_tree)?;
    let member_pyproject_toml_uri =
        tombi_uri::Uri::from_file_path(&member_pyproject_toml_path).ok()?;

    Some(tombi_extension::DefinitionLocation {
        uri: member_pyproject_toml_uri,
        range: package_name.unquoted_range(),
    })
}

pub fn get_path_dependency_definition(
    path: &str,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    let pyproject_toml_path = std::path::PathBuf::from(path).join("pyproject.toml");

    let member_document_tree = load_pyproject_toml(&pyproject_toml_path, toml_version)?;

    let package_name = get_project_name(&member_document_tree)?;

    let member_pyproject_toml_uri = tombi_uri::Uri::from_file_path(&pyproject_toml_path).ok()?;

    Some(tombi_extension::DefinitionLocation {
        uri: member_pyproject_toml_uri,
        range: package_name.unquoted_range(),
    })
}
