use itertools::Itertools;
use std::path::Path;
use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_schema_store::Accessor;

use crate::{
    DependencyRequirement, PyprojectNavigationFeature,
    accessors::{
        is_dependency_group_name_accessors, is_dependency_groups_include_group_accessors,
        is_uv_sources_accessors,
    },
    classify_pyproject_navigation_feature, collect_dependency_requirements_from_document_tree,
    find_dependency_group_key, find_member_project_toml, find_workspace_pyproject_toml,
    get_project_name, goto_definition_for_member_pyproject_toml,
    goto_definition_for_workspace_pyproject_toml, has_uv_sources_accessors,
    is_dependency_name_accessors, is_project_name_accessors, is_pyproject_path_accessors,
    is_uv_source_path_accessors, is_uv_source_workspace_accessors, is_uv_workspace_accessors,
    load_pyproject_toml_document_tree, parse_requirement, resolve_member_pyproject_toml_path,
    resolve_relative_path_uri,
};

pub async fn goto_definition(
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

    if !pyproject_goto_definition_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = if is_project_name_accessors(accessors) {
        goto_definition_for_project_name(document_tree, &text_document_uri)
    } else if is_dependency_group_name_accessors(accessors) {
        goto_definition_for_dependency_group_name(document_tree, accessors, &text_document_uri)
    } else if is_uv_source_path_accessors(accessors) {
        goto_definition_for_relative_package(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )
    } else if has_uv_sources_accessors(accessors) {
        let jump_to_package = is_uv_sources_accessors(accessors)
            || !source_is_workspace_managed(document_tree, accessors);
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            jump_to_package,
        )?
    } else if is_pyproject_path_accessors(accessors) {
        goto_definition_for_relative_file(document_tree, accessors, &pyproject_toml_path)
    } else if is_uv_workspace_accessors(accessors) {
        goto_definition_for_workspace_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
    } else if is_dependency_groups_include_group_accessors(accessors) {
        goto_definition_for_include_group(document_tree, accessors, &pyproject_toml_path)?
    } else if is_dependency_name_accessors(accessors) {
        goto_definition_for_dependency_package(
            document_tree,
            accessors,
            &text_document_uri,
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

fn goto_definition_for_project_name(
    document_tree: &tombi_document_tree::DocumentTree,
    text_document_uri: &tombi_uri::Uri,
) -> Vec<tombi_extension::Location> {
    let Some((_, Value::String(project_name))) = dig_keys(document_tree, &["project", "name"])
    else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: project_name.unquoted_range(),
    }]
}

fn goto_definition_for_dependency_group_name(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
) -> Vec<tombi_extension::Location> {
    let Some(group_name) = accessors.get(1).and_then(Accessor::as_key) else {
        return Vec::with_capacity(0);
    };
    let Some((key, _)) = dig_keys(document_tree, &["dependency-groups", group_name]) else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: key.unquoted_range(),
    }]
}

#[inline]
fn is_workspace_root_pyproject(pyproject_toml_path: &Path, toml_version: TomlVersion) -> bool {
    find_workspace_pyproject_toml(pyproject_toml_path, toml_version).is_some_and(
        |(workspace_pyproject_toml_path, _, _)| {
            workspace_pyproject_toml_path == pyproject_toml_path
        },
    )
}

fn should_stay_on_dependency_string(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> bool {
    let Some((_, Value::String(dep_str))) = dig_accessors(document_tree, accessors) else {
        return false;
    };
    let Some(requirement) = parse_requirement(dep_str.value()) else {
        return false;
    };
    let package_name = requirement.name.as_ref();

    if is_workspace_root_pyproject(pyproject_toml_path, toml_version) {
        return collect_workspace_project_dependency_definitions(
            package_name,
            pyproject_toml_path,
            toml_version,
        )
        .is_empty();
    }

    if let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"])
        && sources.contains_key(package_name)
    {
        return false;
    }

    if !collect_workspace_project_dependency_definitions(
        package_name,
        pyproject_toml_path,
        toml_version,
    )
    .is_empty()
    {
        return false;
    }

    get_workspace_member_package_definition(package_name, pyproject_toml_path, toml_version)
        .is_none()
}

fn source_is_workspace_managed(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
) -> bool {
    let source_accessors = if is_uv_source_workspace_accessors(accessors) {
        &accessors[..accessors.len().saturating_sub(1)]
    } else {
        accessors
    };

    matches!(
        dig_accessors(document_tree, source_accessors),
        Some((_, Value::Table(table)))
            if matches!(
                table.get("workspace"),
                Some(Value::Boolean(is_workspace)) if is_workspace.value()
            )
    )
}

fn goto_definition_for_relative_package(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Vec<tombi_extension::Location> {
    let Some((_, Value::String(path_value))) = dig_accessors(document_tree, accessors) else {
        return Vec::with_capacity(0);
    };

    get_path_dependency_definition(pyproject_toml_path, path_value.value(), toml_version)
        .into_iter()
        .collect()
}

fn goto_definition_for_relative_file(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    pyproject_toml_path: &Path,
) -> Vec<tombi_extension::Location> {
    let Some((_, Value::String(path_value))) = dig_accessors(document_tree, accessors) else {
        return Vec::with_capacity(0);
    };

    let Some(uri) = resolve_relative_path_uri(pyproject_toml_path, Path::new(path_value.value()))
    else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri,
        range: tombi_text::Range::default(),
    }]
}

#[inline]
fn pyproject_goto_definition_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_definition())
        .and_then(
            |goto_definition| match classify_pyproject_navigation_feature(accessors) {
                PyprojectNavigationFeature::Dependency => goto_definition.dependency(),
                PyprojectNavigationFeature::Member => goto_definition.member(),
                PyprojectNavigationFeature::Path => goto_definition.path(),
            },
        )
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}

fn goto_definition_for_dependency_package(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    // Get the dependency string from the current position
    let Some((_, Value::String(dep_str))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::with_capacity(0));
    };

    // Parse the PEP 508 requirement to extract package name
    let Some(requirement) = parse_requirement(dep_str.value()) else {
        return Ok(Vec::with_capacity(0));
    };
    let package_name = requirement.name.as_ref();

    // Check if this package is in tool.uv.sources
    if let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"])
        && let Some((_, Value::Table(source_table))) = sources.get_key_value(package_name)
    {
        if let Some((_, Value::Boolean(is_workspace))) = source_table.get_key_value("workspace") {
            if !is_workspace.value() {
                return Ok(Vec::with_capacity(0));
            }
            let definition_accessors = [
                Accessor::Key("tool".to_string()),
                Accessor::Key("uv".to_string()),
                Accessor::Key("sources".to_string()),
                Accessor::Key(package_name.to_string()),
                Accessor::Key("workspace".to_string()),
            ];
            if let Some(location) = goto_definition_for_member_pyproject_toml(
                document_tree,
                &definition_accessors,
                pyproject_toml_path,
                toml_version,
                true,
            )?
            .into_iter()
            .next()
            {
                return Ok(vec![location]);
            } else {
                return Ok(Vec::with_capacity(0));
            }
        }
        if let Some((_, Value::String(path))) = source_table.get_key_value("path") {
            if let Some(location) =
                get_path_dependency_definition(pyproject_toml_path, path.value(), toml_version)
            {
                return Ok(vec![location]);
            } else {
                return Ok(Vec::with_capacity(0));
            }
        }
    }

    if requirement.version_or_url.is_none() {
        let workspace_dependency_definitions = collect_workspace_project_dependency_definitions(
            package_name,
            pyproject_toml_path,
            toml_version,
        );
        if !workspace_dependency_definitions.is_empty() {
            return Ok(workspace_dependency_definitions);
        }
    }

    if let Some(location) =
        get_workspace_member_package_definition(package_name, pyproject_toml_path, toml_version)
    {
        return Ok(vec![location]);
    }

    Ok(goto_definition_for_dependency_string(
        document_tree,
        accessors,
        text_document_uri,
        pyproject_toml_path,
        toml_version,
    ))
}

fn goto_definition_for_dependency_string(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    text_document_uri: &tombi_uri::Uri,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Vec<tombi_extension::Location> {
    if !should_stay_on_dependency_string(
        document_tree,
        accessors,
        pyproject_toml_path,
        toml_version,
    ) {
        return Vec::with_capacity(0);
    }

    let Some((_, Value::String(dependency))) = dig_accessors(document_tree, accessors) else {
        return Vec::with_capacity(0);
    };

    vec![tombi_extension::Location {
        uri: text_document_uri.clone(),
        range: dependency.unquoted_range(),
    }]
}

fn goto_definition_for_include_group(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    pyproject_toml_path: &std::path::Path,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    let Some((_, Value::String(include_group))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::with_capacity(0));
    };

    let Some(group_key) = find_dependency_group_key(document_tree, include_group.value()) else {
        return Ok(Vec::with_capacity(0));
    };

    let Ok(uri) = tombi_uri::Uri::from_file_path(pyproject_toml_path) else {
        return Ok(Vec::with_capacity(0));
    };

    Ok(vec![tombi_extension::Location {
        uri,
        range: group_key.unquoted_range(),
    }])
}

pub(crate) fn collect_workspace_project_dependency_definitions(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Vec<tombi_extension::Location> {
    let Some((workspace_pyproject_toml_path, _, workspace_document_tree)) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)
    else {
        return Vec::with_capacity(0);
    };

    if workspace_pyproject_toml_path == pyproject_toml_path {
        return Vec::with_capacity(0);
    }

    let Ok(workspace_uri) = tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path) else {
        return Vec::with_capacity(0);
    };

    collect_dependency_requirements_from_document_tree(&workspace_document_tree)
        .iter()
        .filter_map(
            |DependencyRequirement {
                 requirement,
                 dependency,
             }| {
                if requirement.name.as_ref() == package_name {
                    Some(tombi_extension::Location {
                        uri: workspace_uri.clone(),
                        range: dependency.unquoted_range(),
                    })
                } else {
                    None
                }
            },
        )
        .collect_vec()
}

pub(crate) fn get_workspace_member_package_definition(
    package_name: &str,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_extension::Location> {
    let (workspace_pyproject_toml_path, _, workspace_document_tree) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;
    let (member_pyproject_toml_path, _) = find_member_project_toml(
        package_name,
        &workspace_document_tree,
        &workspace_pyproject_toml_path,
        toml_version,
    )?;

    let member_document_tree =
        load_pyproject_toml_document_tree(&member_pyproject_toml_path, toml_version)?;
    let package_name = get_project_name(&member_document_tree)?;
    let member_uri = tombi_uri::Uri::from_file_path(&member_pyproject_toml_path).ok()?;

    Some(tombi_extension::Location {
        uri: member_uri,
        range: package_name.unquoted_range(),
    })
}

pub(crate) fn get_workspace_member_dependency_definitions(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_pyproject_toml_path: &std::path::Path,
    package_name: &str,
    toml_version: TomlVersion,
) -> Vec<tombi_extension::Location> {
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

        for DependencyRequirement {
            requirement,
            dependency,
        } in collect_dependency_requirements_from_document_tree(&member_document_tree)
        {
            if requirement.name.as_ref() == package_name {
                let Ok(uri) = tombi_uri::Uri::from_file_path(&member_pyproject_toml_path) else {
                    continue;
                };
                locations.push(tombi_extension::Location {
                    uri,
                    range: dependency.unquoted_range(),
                });
            }
        }
    }

    locations
}

pub fn get_path_dependency_definition(
    pyproject_toml_path: &std::path::Path,
    path: &str,
    toml_version: TomlVersion,
) -> Option<tombi_extension::Location> {
    let pyproject_toml_path = resolve_member_pyproject_toml_path(pyproject_toml_path, path)?;

    let member_document_tree =
        load_pyproject_toml_document_tree(&pyproject_toml_path, toml_version)?;

    let package_name = get_project_name(&member_document_tree)?;

    let member_pyproject_toml_uri = tombi_uri::Uri::from_file_path(&pyproject_toml_path).ok()?;

    Some(tombi_extension::Location {
        uri: member_pyproject_toml_uri,
        range: package_name.unquoted_range(),
    })
}
