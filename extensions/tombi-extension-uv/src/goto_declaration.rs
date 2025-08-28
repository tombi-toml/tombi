use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl};
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_accessors, dig_keys, Value};
use tombi_schema_store::matches_accessors;

use crate::{
    find_member_project_toml, find_workspace_pyproject_toml,
    goto_definition::get_path_dependency_definition, goto_definition_for_member_pyproject_toml,
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
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            false,
        )?
    } else if matches_accessors!(accessors, ["project", "dependencies", _])
        || matches_accessors!(accessors, ["project", "optional-dependencies", _, _])
        || matches_accessors!(accessors, ["dependency-groups", _, _])
    {
        match goto_declaration_for_dependency_package(
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
        return Ok(Default::default());
    }

    Ok(Some(locations))
}

fn goto_declaration_for_dependency_package(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
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
        return Ok(get_workspace_dependency_declaration(
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

fn get_workspace_dependency_declaration(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    // Find the workspace pyproject.toml
    let (workspace_pyproject_toml_path, workspace_document_tree) =
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
