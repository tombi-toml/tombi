mod document_link;
mod goto_declaration;
mod goto_definition;

pub use document_link::document_link;
pub use goto_declaration::goto_declaration;
pub use goto_definition::goto_definition;
use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_schema_store::{dig_accessors, matches_accessors};
use tower_lsp::lsp_types::Url;

#[derive(Debug, Clone)]
struct PackageLocation {
    pyproject_toml_path: std::path::PathBuf,
    package_name_key_range: tombi_text::Range,
}

impl From<PackageLocation> for Option<tombi_extension::DefinitionLocation> {
    fn from(package_location: PackageLocation) -> Self {
        let Ok(uri) = Url::from_file_path(&package_location.pyproject_toml_path) else {
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
) -> Option<tombi_document_tree::DocumentTree> {
    let toml_text = std::fs::read_to_string(pyproject_toml_path).ok()?;

    let root =
        tombi_ast::Root::cast(tombi_parser::parse(&toml_text, toml_version).into_syntax_node())?;

    root.try_into_document_tree(toml_version).ok()
}

fn find_workspace_pyproject_toml(
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<(std::path::PathBuf, tombi_document_tree::DocumentTree)> {
    let mut current_dir = pyproject_toml_path.parent()?;

    while let Some(target_dir) = current_dir.parent() {
        current_dir = target_dir;
        let workspace_pyproject_toml_path = current_dir.join("pyproject.toml");

        if workspace_pyproject_toml_path.exists() {
            let Some(package_pyproject_toml_document_tree) =
                load_pyproject_toml(&workspace_pyproject_toml_path, toml_version)
            else {
                continue;
            };

            // Check if this pyproject.toml has a [tool.uv.workspace] section

            if tombi_document_tree::dig_keys(
                &package_pyproject_toml_document_tree,
                &["tool", "uv", "workspace"],
            )
            .is_some()
            {
                return Some((
                    workspace_pyproject_toml_path,
                    package_pyproject_toml_document_tree,
                ));
            }
        }
    }

    None
}

fn find_member_pyproject_toml_paths<'a>(
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

    let Some((workspace_pyproject_toml_path, workspace_pyproject_toml_document_tree)) =
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
        let Ok(package_pyproject_toml_uri) = Url::from_file_path(&package_toml_path) else {
            return Ok(None);
        };

        Ok(Some(tombi_extension::DefinitionLocation {
            uri: package_pyproject_toml_uri,
            range: tombi_text::Range::default(),
        }))
    } else {
        let Ok(workspace_pyproject_toml_uri) = Url::from_file_path(&workspace_pyproject_toml_path)
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
    let member_patterns =
        if matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _]) {
            let Some((_, tombi_document_tree::Value::String(member))) =
                dig_accessors(workspace_document_tree, accessors)
            else {
                return Ok(Vec::with_capacity(0));
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
        };

    let Some(workspace_dir_path) = workspace_pyproject_toml_path.parent() else {
        return Ok(Vec::with_capacity(0));
    };

    let exclude_patterns = match tombi_document_tree::dig_keys(
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
    };

    let mut locations = Vec::new();
    for (_, pyproject_toml_path) in
        find_member_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(member_document_tree) =
            crate::load_pyproject_toml(&pyproject_toml_path, toml_version)
        else {
            continue;
        };

        let Some((_, tombi_document_tree::Value::String(package_name))) =
            tombi_document_tree::dig_keys(&member_document_tree, &["project", "name"])
        else {
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

    let member_patterns = match tombi_document_tree::dig_keys(
        workspace_pyproject_toml_document_tree,
        &[&"tool", &"uv", &"workspace", &"members"],
    ) {
        Some((_, tombi_document_tree::Value::Array(members))) => members
            .iter()
            .filter_map(|member| match member {
                tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                _ => None,
            })
            .collect_vec(),
        _ => vec![],
    };

    let exclude_patterns = match tombi_document_tree::dig_keys(
        workspace_pyproject_toml_document_tree,
        &[&"tool", &"uv", &"workspace", &"exclude"],
    ) {
        Some((_, tombi_document_tree::Value::Array(exclude))) => exclude
            .iter()
            .filter_map(|member| match member {
                tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                _ => None,
            })
            .collect_vec(),
        _ => vec![],
    };

    for (member_item, package_project_toml_path) in
        find_member_project_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(package_project_toml_document_tree) =
            load_pyproject_toml(&package_project_toml_path, toml_version)
        else {
            continue;
        };

        if let Some((_, tombi_document_tree::Value::String(name))) =
            tombi_document_tree::dig_keys(&package_project_toml_document_tree, &["project", "name"])
        {
            if name.value() == package_name {
                return Some((package_project_toml_path, member_item.unquoted_range()));
            }
        }
    }

    None
}

fn find_member_project_toml_paths<'a>(
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
            let mut project_toml_paths = vec![];

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
                let project_toml_path = if candidate_path.is_dir() {
                    candidate_path.join("pyproject.toml")
                } else {
                    continue;
                };

                if !project_toml_path.exists() || !project_toml_path.is_file() {
                    continue;
                }

                // Check if the path is excluded
                let is_excluded = exclude_patterns.iter().any(|exclude_pattern| {
                    exclude_pattern.matches(&project_toml_path.to_string_lossy())
                });

                if !is_excluded {
                    project_toml_paths.push((member_pattern, project_toml_path));
                }
            }

            if !project_toml_paths.is_empty() {
                Some(project_toml_paths)
            } else {
                None
            }
        })
        .flatten()
}
