use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_document_tree::dig_accessors;
use tombi_schema_store::matches_accessors;

use crate::{
    PackageLocation, find_workspace_pyproject_toml, get_project_name,
    load_pyproject_toml_document_tree,
};

pub(crate) fn extract_member_patterns<'a>(
    workspace_document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [tombi_schema_store::Accessor],
) -> Vec<&'a tombi_document_tree::String> {
    if matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _]) {
        let Some((_, tombi_document_tree::Value::String(member))) =
            dig_accessors(workspace_document_tree, accessors)
        else {
            return vec![];
        };
        vec![member]
    } else {
        match tombi_document_tree::dig_keys(
            workspace_document_tree,
            &["tool", "uv", "workspace", "members"],
        ) {
            Some((_, tombi_document_tree::Value::Array(members))) => members
                .iter()
                .filter_map(|member| match member {
                    tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                    _ => None,
                })
                .collect_vec(),
            _ => vec![],
        }
    }
}

pub(crate) fn extract_exclude_patterns(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
) -> Vec<&tombi_document_tree::String> {
    match tombi_document_tree::dig_keys(
        workspace_document_tree,
        &["tool", "uv", "workspace", "exclude"],
    ) {
        Some((_, tombi_document_tree::Value::Array(exclude))) => exclude
            .iter()
            .filter_map(|member| match member {
                tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                _ => None,
            })
            .collect_vec(),
        _ => Vec::with_capacity(0),
    }
}

pub(crate) fn find_pyproject_toml_paths<'a>(
    member_patterns: &'a [&'a tombi_document_tree::String],
    exclude_patterns: &'a [&'a tombi_document_tree::String],
    workspace_dir_path: &'a std::path::Path,
) -> impl Iterator<Item = (&'a tombi_document_tree::String, std::path::PathBuf)> + 'a {
    let exclude_patterns = exclude_patterns
        .iter()
        .filter_map(|pattern| glob::Pattern::new(pattern.value()).ok())
        .collect_vec();

    member_patterns
        .iter()
        .filter_map(move |&member_pattern| {
            let mut manifest_paths = vec![];

            let mut member_pattern_path =
                std::path::Path::new(member_pattern.value()).to_path_buf();
            if !member_pattern_path.is_absolute() {
                member_pattern_path = workspace_dir_path.join(member_pattern_path);
            }

            let mut candidate_paths = match glob::glob(&member_pattern_path.to_string_lossy()) {
                Ok(paths) => paths,
                Err(_) => return None,
            };

            while let Some(Ok(candidate_path)) = candidate_paths.next() {
                if !candidate_path.is_dir() {
                    continue;
                }

                let manifest_path = candidate_path.join("pyproject.toml");
                if !manifest_path.is_file() {
                    continue;
                }

                let is_excluded = exclude_patterns.iter().any(|exclude_pattern| {
                    exclude_pattern.matches(&manifest_path.to_string_lossy())
                });

                if !is_excluded {
                    manifest_paths.push((member_pattern, manifest_path));
                }
            }

            (!manifest_paths.is_empty()).then_some(manifest_paths)
        })
        .flatten()
}

pub(crate) fn goto_definition_for_member_pyproject_toml(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_package: bool,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["tool", "uv", "sources", _])
        || matches_accessors!(accessors, ["tool", "uv", "sources", _, "workspace"])
    {
        match goto_workspace_member(
            document_tree,
            accessors,
            pyproject_toml_path,
            toml_version,
            jump_to_package,
        )? {
            Some(location) => Ok(vec![location]),
            None => Ok(Vec::with_capacity(0)),
        }
    } else {
        Ok(Vec::with_capacity(0))
    }
}

pub(crate) fn goto_definition_for_workspace_pyproject_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["tool", "uv", "workspace", "members"])
        || matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _])
    {
        goto_member_pyprojects(
            workspace_document_tree,
            accessors,
            workspace_pyproject_toml_path,
            toml_version,
        )
        .map(|locations| locations.into_iter().filter_map(Into::into).collect_vec())
    } else {
        Ok(Vec::with_capacity(0))
    }
}

pub(crate) fn goto_workspace_member(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_package: bool,
) -> Result<Option<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    debug_assert!(
        matches_accessors!(accessors, ["tool", "uv", "sources", _])
            || matches_accessors!(accessors, ["tool", "uv", "sources", _, "workspace"])
    );

    let Some((workspace_pyproject_toml_path, _, workspace_pyproject_toml_document_tree)) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)
    else {
        return Ok(None);
    };

    let package_name = if let tombi_schema_store::Accessor::Key(key) = &accessors[3] {
        key
    } else {
        return Ok(None);
    };
    if accessors.len() == 4
        && let Some((_, tombi_document_tree::Value::Table(table))) =
            dig_accessors(document_tree, &accessors[..4])
        && !table.contains_key("workspace")
    {
        return Ok(None);
    }

    let Some((package_toml_path, member_range)) = find_member_project_toml(
        package_name,
        &workspace_pyproject_toml_document_tree,
        &workspace_pyproject_toml_path,
        toml_version,
    ) else {
        return Ok(None);
    };

    if jump_to_package {
        let Ok(package_pyproject_toml_uri) = tombi_uri::Uri::from_file_path(&package_toml_path)
        else {
            return Ok(None);
        };
        let Some(member_document_tree) =
            load_pyproject_toml_document_tree(&package_toml_path, toml_version)
        else {
            return Ok(None);
        };
        let Some(package_name) = get_project_name(&member_document_tree) else {
            return Ok(None);
        };

        Ok(Some(tombi_extension::Location {
            uri: package_pyproject_toml_uri,
            range: package_name.unquoted_range(),
        }))
    } else {
        let Ok(workspace_pyproject_toml_uri) =
            tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path)
        else {
            return Ok(None);
        };

        Ok(Some(tombi_extension::Location {
            uri: workspace_pyproject_toml_uri,
            range: member_range,
        }))
    }
}

pub(crate) fn goto_member_pyprojects(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<PackageLocation>, tower_lsp::jsonrpc::Error> {
    let member_patterns = extract_member_patterns(workspace_document_tree, accessors);
    if member_patterns.is_empty() {
        return Ok(Vec::with_capacity(0));
    }

    let Some(workspace_dir_path) = workspace_pyproject_toml_path.parent() else {
        return Ok(Vec::with_capacity(0));
    };

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);

    let mut locations = Vec::new();
    for (_, pyproject_toml_path) in
        find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(member_document_tree) =
            load_pyproject_toml_document_tree(&pyproject_toml_path, toml_version)
        else {
            continue;
        };

        let Some(package_name) = get_project_name(&member_document_tree) else {
            continue;
        };

        locations.push(PackageLocation {
            pyproject_toml_path,
            package_name_key_range: package_name.unquoted_range(),
        });
    }

    Ok(locations)
}

pub(crate) fn find_member_project_toml(
    package_name: &str,
    workspace_pyproject_toml_document_tree: &tombi_document_tree::DocumentTree,
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<(std::path::PathBuf, tombi_text::Range)> {
    let workspace_dir_path = workspace_pyproject_toml_path.parent()?;

    let member_patterns = extract_member_patterns(workspace_pyproject_toml_document_tree, &[]);
    let exclude_patterns = extract_exclude_patterns(workspace_pyproject_toml_document_tree);

    for (member_item, package_project_toml_path) in
        find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(package_project_toml_document_tree) =
            load_pyproject_toml_document_tree(&package_project_toml_path, toml_version)
        else {
            continue;
        };

        if let Some(name) = get_project_name(&package_project_toml_document_tree)
            && name.value() == package_name
        {
            return Some((package_project_toml_path, member_item.unquoted_range()));
        }
    }

    None
}
