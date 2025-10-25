mod code_action;
mod document_link;
mod goto_declaration;
mod goto_definition;

use std::str::FromStr;

pub use code_action::code_action;
pub use document_link::document_link;
pub use goto_declaration::goto_declaration;
pub use goto_definition::goto_definition;
use itertools::Itertools;
use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_accessors, dig_keys, TryIntoDocumentTree};
use tombi_schema_store::matches_accessors;

#[derive(Debug, Clone)]
struct DependencyRequirement<'a> {
    dependency: &'a tombi_document_tree::String,
    requirement: Requirement<VerbatimUrl>,
}

impl<'a> DependencyRequirement<'a> {
    #[inline]
    fn version_or_url(&self) -> Option<&VersionOrUrl<VerbatimUrl>> {
        self.requirement.version_or_url.as_ref()
    }
}

#[derive(Debug, Clone)]
struct PackageLocation {
    pyproject_toml_path: std::path::PathBuf,
    package_name_key_range: tombi_text::Range,
}

impl From<PackageLocation> for Option<tombi_extension::DefinitionLocation> {
    fn from(package_location: PackageLocation) -> Self {
        let Ok(uri) = tombi_uri::Uri::from_file_path(&package_location.pyproject_toml_path) else {
            return None;
        };

        Some(tombi_extension::DefinitionLocation {
            uri,
            range: package_location.package_name_key_range,
        })
    }
}

fn load_pyproject_toml(
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<(tombi_ast::Root, tombi_document_tree::DocumentTree)> {
    let toml_text = std::fs::read_to_string(pyproject_toml_path).ok()?;

    let root =
        tombi_ast::Root::cast(tombi_parser::parse(&toml_text, toml_version).into_syntax_node())?;

    // Clone the root before converting to document tree
    let root_clone = tombi_ast::Root::cast(root.syntax().clone())?;
    let document_tree = root.try_into_document_tree(toml_version).ok()?;

    Some((root_clone, document_tree))
}

fn load_pyproject_toml_document_tree(
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_document_tree::DocumentTree> {
    let (_, document_tree) = load_pyproject_toml(pyproject_toml_path, toml_version)?;
    Some(document_tree)
}

fn find_workspace_pyproject_toml(
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    let mut current_dir = pyproject_toml_path.parent()?;

    while let Some(target_dir) = current_dir.parent() {
        current_dir = target_dir;
        let workspace_pyproject_toml_path = current_dir.join("pyproject.toml");

        if workspace_pyproject_toml_path.exists() {
            let Some((root, document_tree)) =
                load_pyproject_toml(&workspace_pyproject_toml_path, toml_version)
            else {
                continue;
            };

            // Check if this pyproject.toml has a [tool.uv.workspace] section
            if tombi_document_tree::dig_keys(&document_tree, &["tool", "uv", "workspace"]).is_some()
            {
                return Some((workspace_pyproject_toml_path, root, document_tree));
            }
        }
    }

    None
}

/// Helper function to extract member patterns from workspace document tree
fn extract_member_patterns<'a>(
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

/// Helper function to extract exclude patterns from workspace document tree
fn extract_exclude_patterns(
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

/// Helper function to get project name from document tree
fn get_project_name(
    document_tree: &tombi_document_tree::DocumentTree,
) -> Option<&tombi_document_tree::String> {
    match tombi_document_tree::dig_keys(document_tree, &["project", "name"]) {
        Some((_, tombi_document_tree::Value::String(name))) => Some(name),
        _ => None,
    }
}

/// Generic function to find pyproject.toml files based on member patterns
fn find_pyproject_toml_paths<'a>(
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
            let mut pyproject_toml_paths = vec![];

            let mut member_pattern_path =
                std::path::Path::new(member_pattern.value()).to_path_buf();
            if !member_pattern_path.is_absolute() {
                member_pattern_path = workspace_dir_path.join(member_pattern_path);
            }

            // Find matching paths using glob
            let mut candidate_paths = match glob::glob(&member_pattern_path.to_string_lossy()) {
                Ok(paths) => paths,
                Err(_) => return None,
            };

            // Check if any path matches and is not excluded
            while let Some(Ok(candidate_path)) = candidate_paths.next() {
                // Skip if the path doesn't contain pyproject.toml
                let pyproject_toml_path = if candidate_path.is_dir() {
                    candidate_path.join("pyproject.toml")
                } else {
                    continue;
                };

                if !pyproject_toml_path.exists() || !pyproject_toml_path.is_file() {
                    continue;
                }

                // Check if the path is excluded
                let is_excluded = exclude_patterns.iter().any(|exclude_pattern| {
                    exclude_pattern.matches(&pyproject_toml_path.to_string_lossy())
                });

                if !is_excluded {
                    pyproject_toml_paths.push((member_pattern, pyproject_toml_path));
                }
            }

            if !pyproject_toml_paths.is_empty() {
                Some(pyproject_toml_paths)
            } else {
                None
            }
        })
        .flatten()
}

fn goto_definition_for_member_pyproject_toml(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_package: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
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

fn goto_definition_for_workspace_pyproject_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
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

/// Get the location of the workspace pyproject.toml.
///
/// ```toml
/// [project]
/// name = "example"
/// version = "0.1.0"
/// dependencies = ["other-package"]
///
/// [tool.uv.sources]
/// other-package = { workspace = true }
/// ```
fn goto_workspace_member(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_package: bool,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    assert!(
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
    if accessors.len() == 3 {
        if let Some((_, tombi_document_tree::Value::Table(table))) =
            dig_accessors(document_tree, accessors)
        {
            if !table.contains_key("workspace") {
                return Ok(None);
            }
        }
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
            load_pyproject_toml_document_tree(pyproject_toml_path, toml_version)
        else {
            return Ok(None);
        };
        let Some(package_name) = get_project_name(&member_document_tree) else {
            return Ok(None);
        };

        Ok(Some(tombi_extension::DefinitionLocation {
            uri: package_pyproject_toml_uri,
            range: package_name.unquoted_range(),
        }))
    } else {
        let Ok(workspace_pyproject_toml_uri) =
            tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path)
        else {
            return Ok(None);
        };

        Ok(Some(tombi_extension::DefinitionLocation {
            uri: workspace_pyproject_toml_uri,
            range: member_range,
        }))
    }
}

/// Get the location of the workspace members definition.
///
/// ```toml
/// [tool.uv.workspace]
/// members = ["python/tombi-beta"]
/// ```
fn goto_member_pyprojects(
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

fn find_member_project_toml(
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

        if let Some(name) = get_project_name(&package_project_toml_document_tree) {
            if name.value() == package_name {
                return Some((package_project_toml_path, member_item.unquoted_range()));
            }
        }
    }

    None
}

fn parse_requirement(dependency: &str) -> Option<Requirement<VerbatimUrl>> {
    match Requirement::<VerbatimUrl>::from_str(dependency) {
        Ok(requirement) => Some(requirement),
        Err(e) => {
            tracing::debug!(
                dependency = %dependency,
                error = %e,
                "Failed to parse PEP 508 dependency string"
            );
            None
        }
    }
}

fn parse_dependency_requirement<'a>(
    dependency: &'a tombi_document_tree::String,
) -> Option<DependencyRequirement<'a>> {
    parse_requirement(dependency.value()).map(|requirement| DependencyRequirement {
        requirement,
        dependency,
    })
}

fn collect_dependency_requirements_from_document_tree<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
) -> Vec<DependencyRequirement<'a>> {
    let mut dependency_requirements = Vec::new();

    if let Some((_, tombi_document_tree::Value::Array(dep_array))) =
        dig_keys(document_tree, &["project", "dependencies"])
    {
        dependency_requirements.extend(collect_dependency_requirements_from_values::<'a>(
            dep_array.iter(),
        ));
    }
    if let Some((_, tombi_document_tree::Value::Table(dep_group))) =
        dig_keys(document_tree, &["project", "optional-dependencies"])
    {
        for value in dep_group.values() {
            if let tombi_document_tree::Value::Array(dep_array) = value {
                dependency_requirements.extend(collect_dependency_requirements_from_values(
                    dep_array.iter(),
                ));
            }
        }
    }
    if let Some((_, tombi_document_tree::Value::Table(dep_group))) =
        dig_keys(document_tree, &["dependency-groups"])
    {
        for value in dep_group.values() {
            if let tombi_document_tree::Value::Array(dep_array) = value {
                dependency_requirements.extend(collect_dependency_requirements_from_values(
                    dep_array.iter(),
                ));
            }
        }
    }

    dependency_requirements
}

fn collect_dependency_requirements_from_values<'a>(
    dependencies: impl Iterator<Item = &'a tombi_document_tree::Value>,
) -> Vec<DependencyRequirement<'a>> {
    dependencies
        .filter_map(|value| {
            if let tombi_document_tree::Value::String(dep_str) = value {
                parse_requirement(dep_str.value()).map(|requirement| DependencyRequirement {
                    requirement,
                    dependency: dep_str,
                })
            } else {
                None
            }
        })
        .collect()
}
