use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl};
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_accessors, dig_keys, Value};
use tombi_schema_store::{matches_accessors, Accessor};

use crate::{
    find_member_project_toml, find_workspace_pyproject_toml, get_project_name,
    goto_definition_for_member_pyproject_toml, goto_definition_for_workspace_pyproject_toml,
    load_pyproject_toml_document_tree,
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
        goto_definition_for_dependency_package(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
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
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
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

    // Check if this package is in tool.uv.sources
    if let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"]) {
        if let Some((_, Value::Table(source_table))) = sources.get_key_value(package_name) {
            if let Some((_, Value::Boolean(is_workspace))) = source_table.get_key_value("workspace")
            {
                if !is_workspace.value() {
                    return Ok(Vec::with_capacity(0));
                }
                if let Some(location) = get_workspace_dependency_definition(
                    package_name,
                    pyproject_toml_path,
                    toml_version,
                ) {
                    return Ok(vec![location]);
                } else {
                    return Ok(Vec::with_capacity(0));
                }
            }
            if let Some((_, Value::String(path))) = source_table.get_key_value("path") {
                if let Some(location) = get_path_dependency_definition(path.value(), toml_version) {
                    return Ok(vec![location]);
                } else {
                    return Ok(Vec::with_capacity(0));
                }
            }
        }
    }

    let mut locations = Vec::new();

    // Package references without version info should jump to workspace definition when available
    if requirement.version_or_url.is_none() {
        if let Some(location) = get_workspace_project_dependency_definition(
            package_name.as_ref(),
            pyproject_toml_path,
            toml_version,
        ) {
            locations.push(location);
        }
    }

    if dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
        locations.extend(get_workspace_member_dependency_definitions(
            document_tree,
            pyproject_toml_path,
            package_name,
            toml_version,
        ));
    }

    Ok(locations)
}

fn get_workspace_dependency_definition(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    // Find the workspace pyproject.toml
    let (workspace_path, _, workspace_document_tree) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;

    // Find the member project
    let (member_pyproject_toml_path, _) = find_member_project_toml(
        package_name,
        &workspace_document_tree,
        &workspace_path,
        toml_version,
    )?;

    let member_document_tree =
        load_pyproject_toml_document_tree(&member_pyproject_toml_path, toml_version)?;
    let package_name = get_project_name(&member_document_tree)?;
    let member_pyproject_toml_uri =
        tombi_uri::Uri::from_file_path(&member_pyproject_toml_path).ok()?;

    Some(tombi_extension::DefinitionLocation {
        uri: member_pyproject_toml_uri,
        range: package_name.unquoted_range(),
    })
}

pub(crate) fn get_workspace_project_dependency_definition(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    let (workspace_pyproject_toml_path, _, workspace_document_tree) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;

    let Some((_, Value::Array(workspace_dependencies))) =
        dig_keys(&workspace_document_tree, &["project", "dependencies"])
    else {
        return None;
    };

    let workspace_dependency = workspace_dependencies.iter().find_map(|value| {
        if let Value::String(dep_string) = value {
            Requirement::<VerbatimUrl>::from_str(dep_string.value())
                .ok()
                .filter(|requirement| requirement.name.as_ref() == package_name)
                .map(|_| dep_string)
        } else {
            None
        }
    })?;

    let workspace_uri = tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path).ok()?;

    Some(tombi_extension::DefinitionLocation {
        uri: workspace_uri,
        range: workspace_dependency.unquoted_range(),
    })
}

pub(crate) fn get_workspace_member_dependency_definitions(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_pyproject_toml_path: &std::path::Path,
    package_name: &str,
    toml_version: TomlVersion,
) -> Vec<tombi_extension::DefinitionLocation> {
    let member_patterns = crate::extract_member_patterns(workspace_document_tree, &[]);
    if member_patterns.is_empty() {
        return Vec::with_capacity(0);
    }

    let Some(workspace_dir_path) = workspace_pyproject_toml_path.parent() else {
        return Vec::with_capacity(0);
    };

    let exclude_patterns = crate::extract_exclude_patterns(workspace_document_tree);

    let mut locations = Vec::new();
    for (_, member_pyproject_toml_path) in
        crate::find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(member_document_tree) =
            crate::load_pyproject_toml_document_tree(&member_pyproject_toml_path, toml_version)
        else {
            continue;
        };

        collect_package_references_in_member(
            &member_document_tree,
            &member_pyproject_toml_path,
            package_name,
            &mut locations,
        );
    }

    locations
}

fn collect_package_references_in_member(
    member_document_tree: &tombi_document_tree::DocumentTree,
    member_pyproject_toml_path: &std::path::Path,
    package_name: &str,
    locations: &mut Vec<tombi_extension::DefinitionLocation>,
) {
    let mut collect_from_dependencies = |dependencies: &tombi_document_tree::Array| {
        for dependency in dependencies.iter() {
            let Value::String(dep_string) = dependency else {
                continue;
            };

            if Requirement::<VerbatimUrl>::from_str(dep_string.value())
                .ok()
                .filter(|requirement| requirement.name.as_ref() == package_name)
                .is_some()
            {
                let Ok(uri) = tombi_uri::Uri::from_file_path(member_pyproject_toml_path) else {
                    continue;
                };
                locations.push(tombi_extension::DefinitionLocation {
                    uri,
                    range: dep_string.unquoted_range(),
                });
            }
        }
    };

    if let Some((_, Value::Array(dependencies))) =
        dig_keys(member_document_tree, &["project", "dependencies"])
    {
        collect_from_dependencies(dependencies);
    }

    if let Some((_, Value::Table(optional_dependencies))) =
        dig_keys(member_document_tree, &["project", "optional-dependencies"])
    {
        for option in optional_dependencies.values() {
            if let Value::Array(dependencies) = option {
                collect_from_dependencies(dependencies);
            }
        }
    }

    if let Some((_, Value::Table(dependency_groups))) =
        dig_keys(member_document_tree, &["dependency-groups"])
    {
        for group in dependency_groups.values() {
            if let Value::Array(dependencies) = group {
                collect_from_dependencies(dependencies);
            }
        }
    }
}

pub fn get_path_dependency_definition(
    path: &str,
    toml_version: TomlVersion,
) -> Option<tombi_extension::DefinitionLocation> {
    let pyproject_toml_path = std::path::PathBuf::from(path).join("pyproject.toml");

    let member_document_tree =
        load_pyproject_toml_document_tree(&pyproject_toml_path, toml_version)?;

    let package_name = get_project_name(&member_document_tree)?;

    let member_pyproject_toml_uri = tombi_uri::Uri::from_file_path(&pyproject_toml_path).ok()?;

    Some(tombi_extension::DefinitionLocation {
        uri: member_pyproject_toml_uri,
        range: package_name.unquoted_range(),
    })
}
