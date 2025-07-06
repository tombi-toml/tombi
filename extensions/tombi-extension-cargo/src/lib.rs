mod code_action;
mod completion;
mod document_link;
mod goto_declaration;
mod goto_definition;

pub use code_action::{code_action, CodeActionRefactorRewriteName};
pub use completion::completion;
pub use document_link::{document_link, DocumentLinkToolTip};
pub use goto_declaration::goto_declaration;
pub use goto_definition::goto_definition;
use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_keys, TryIntoDocumentTree, ValueImpl};
use tombi_schema_store::{dig_accessors, matches_accessors};
use tower_lsp::lsp_types::Url;

#[derive(Debug, Clone)]
struct CrateLocation {
    cargo_toml_path: std::path::PathBuf,
    package_name_key_range: tombi_text::Range,
}

impl From<CrateLocation> for Option<tombi_extension::DefinitionLocation> {
    fn from(crate_location: CrateLocation) -> Self {
        let Ok(uri) = Url::from_file_path(&crate_location.cargo_toml_path) else {
            return None;
        };

        Some(tombi_extension::DefinitionLocation {
            uri,
            range: crate_location.package_name_key_range,
        })
    }
}

fn load_cargo_toml(
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<tombi_document_tree::DocumentTree> {
    let toml_text = std::fs::read_to_string(cargo_toml_path).ok()?;

    let root =
        tombi_ast::Root::cast(tombi_parser::parse(&toml_text, toml_version).into_syntax_node())?;

    root.try_into_document_tree(toml_version).ok()
}

fn find_workspace_cargo_toml(
    cargo_toml_path: &std::path::Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(std::path::PathBuf, tombi_document_tree::DocumentTree)> {
    let mut current_dir = cargo_toml_path.parent()?;

    if let Some(workspace_path) = workspace_path {
        let mut workspace_cargo_toml_path =
            std::path::PathBuf::from(workspace_path).join("Cargo.toml");
        if !workspace_cargo_toml_path.is_absolute() {
            if let Some(joined_path) = cargo_toml_path
                .parent()
                .map(|parent| parent.join(&workspace_cargo_toml_path))
            {
                workspace_cargo_toml_path = joined_path;
            }
        }
        if let Ok(canonicalized_path) = std::fs::canonicalize(&workspace_cargo_toml_path) {
            let document_tree = load_cargo_toml(&canonicalized_path, toml_version)?;
            if document_tree.contains_key("workspace") {
                return Some((canonicalized_path, document_tree));
            };
        }
        return None;
    }

    while let Some(target_dir) = current_dir.parent() {
        current_dir = target_dir;
        let workspace_cargo_toml_path = current_dir.join("Cargo.toml");

        if workspace_cargo_toml_path.exists() {
            let Some(document_tree) = load_cargo_toml(&workspace_cargo_toml_path, toml_version)
            else {
                continue;
            };

            if document_tree.contains_key("workspace") {
                return Some((workspace_cargo_toml_path, document_tree));
            };
        }
    }

    None
}

fn find_path_crate_cargo_toml(
    cargo_toml_path: &std::path::Path,
    crate_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<(std::path::PathBuf, tombi_document_tree::DocumentTree)> {
    let mut crate_path = crate_path.to_path_buf();
    if !crate_path.is_absolute() {
        if let Some(workspace_dir) = cargo_toml_path.parent() {
            crate_path = workspace_dir.join(crate_path);
        }
    }

    let Ok(crate_path) = crate_path.canonicalize() else {
        return None;
    };

    let subcrate_cargo_toml_path = crate_path.join("Cargo.toml");
    if !subcrate_cargo_toml_path.exists() {
        return None;
    }

    let document_tree = load_cargo_toml(&subcrate_cargo_toml_path, toml_version)?;

    Some((subcrate_cargo_toml_path, document_tree))
}

/// Get the location of the workspace Cargo.toml.
///
/// ```toml
/// [project]
/// name = "my_project"
/// version = { workspace = true }
///
/// [dependencies]
/// tombi-ast = { workspace = true }
/// ```
fn goto_workspace(
    accessors: &[tombi_schema_store::Accessor],
    crate_cargo_toml_path: &std::path::Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    assert!(matches!(
        accessors.last(),
        Some(tombi_schema_store::Accessor::Key(key)) if key == "workspace"
    ));

    let Some((workspace_cargo_toml_path, workspace_cargo_toml_document_tree)) =
        find_workspace_cargo_toml(crate_cargo_toml_path, workspace_path, toml_version)
    else {
        return Ok(None);
    };

    let keys = {
        let mut sanitized_keys = if let tombi_schema_store::Accessor::Key(key) = &accessors[0] {
            vec![sanitize_dependency_key(key)]
        } else {
            return Ok(None);
        };
        sanitized_keys.extend(accessors[1..].iter().filter_map(|accessor| {
            if let tombi_schema_store::Accessor::Key(key) = accessor {
                Some(key.as_str())
            } else {
                None
            }
        }));
        sanitized_keys
    };

    let Some((key, value)) = tombi_document_tree::dig_keys(
        &workspace_cargo_toml_document_tree,
        &std::iter::once("workspace")
            .chain(keys[..keys.len() - 1].iter().copied())
            .collect_vec(),
    ) else {
        return Ok(None);
    };

    if jump_to_subcrate
        && matches!(
            keys.first(),
            Some(key) if *key == "dependencies" || *key == "dev-dependencies" || *key == "build-dependencies"
        )
    {
        if let tombi_document_tree::Value::Table(table) = value {
            if let Some(tombi_document_tree::Value::String(subcrate_path)) = table.get("path") {
                if let Some((subcrate_cargo_toml_path, subcrate_document_tree)) =
                    find_path_crate_cargo_toml(
                        &workspace_cargo_toml_path,
                        std::path::Path::new(subcrate_path.value()),
                        toml_version,
                    )
                {
                    if let Some((_, tombi_document_tree::Value::String(package_name))) =
                        tombi_document_tree::dig_keys(&subcrate_document_tree, &["package", "name"])
                    {
                        return Ok(Some(tombi_extension::DefinitionLocation {
                            uri: Url::from_file_path(subcrate_cargo_toml_path).unwrap(),
                            range: package_name.unquoted_range(),
                        }));
                    }
                }
            }
        }
    }

    let Ok(workspace_cargo_toml_uri) = Url::from_file_path(&workspace_cargo_toml_path) else {
        return Ok(None);
    };

    Ok(Some(tombi_extension::DefinitionLocation {
        uri: workspace_cargo_toml_uri,
        range: key.unquoted_range(),
    }))
}

/// Get the location of the crate path in the workspace.
///
/// ```toml
/// [workspace.dependencies]
/// tombi-ast█ = { path = "crates/tombi-ast" }
/// ```
fn goto_dependency_crates(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    assert!(
        matches_accessors!(accessors, ["workspace", "dependencies", _])
            || matches_accessors!(accessors, ["dependencies", _])
            || matches_accessors!(accessors, ["dev-dependencies", _])
            || matches_accessors!(accessors, ["build-dependencies", _])
    );

    let Some((tombi_schema_store::Accessor::Key(crate_name), crate_value)) =
        tombi_schema_store::dig_accessors(workspace_document_tree, accessors)
    else {
        return Ok(Vec::with_capacity(0));
    };

    let is_workspace_cargo_toml =
        matches_accessors!(accessors[..accessors.len().min(1)], ["workspace"]);
    let mut locations = Vec::new();
    if let tombi_document_tree::Value::Table(table) = crate_value {
        if let Some(tombi_document_tree::Value::String(subcrate_path)) = table.get("path") {
            if let Some((subcrate_cargo_toml_path, subcrate_document_tree)) =
                find_path_crate_cargo_toml(
                    workspace_cargo_toml_path,
                    std::path::Path::new(subcrate_path.value()),
                    toml_version,
                )
            {
                if let Some((_, tombi_document_tree::Value::String(package_name))) =
                    tombi_document_tree::dig_keys(&subcrate_document_tree, &["package", "name"])
                {
                    locations.push(tombi_extension::DefinitionLocation {
                        uri: Url::from_file_path(subcrate_cargo_toml_path).unwrap(),
                        range: package_name.unquoted_range(),
                    });
                }
            }
        } else if let Some(tombi_document_tree::Value::Boolean(has_workspace)) =
            table.get("workspace")
        {
            if has_workspace.value() {
                let mut accessors = accessors.iter().map(Clone::clone).collect_vec();
                accessors.push(tombi_schema_store::Accessor::Key("workspace".to_string()));
                if is_workspace_cargo_toml {
                    locations.extend(goto_definition_for_workspace_cargo_toml(
                        workspace_document_tree,
                        &accessors,
                        workspace_cargo_toml_path,
                        toml_version,
                        jump_to_subcrate,
                    )?);
                } else {
                    locations.extend(goto_definition_for_crate_cargo_toml(
                        workspace_document_tree,
                        &accessors,
                        workspace_cargo_toml_path,
                        toml_version,
                        jump_to_subcrate,
                    )?);
                }
            }
        }
    }
    if is_workspace_cargo_toml {
        for crate_location in goto_workspace_member_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            "members",
        )? {
            let Some(crate_document_tree) =
                load_cargo_toml(&crate_location.cargo_toml_path, toml_version)
            else {
                continue;
            };

            for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some((crate_key, _)) = tombi_document_tree::dig_keys(
                    &crate_document_tree,
                    &[dependency_key, crate_name],
                ) {
                    if let Some(mut definition_location) =
                        Option::<tombi_extension::DefinitionLocation>::from(crate_location.clone())
                    {
                        definition_location.range = crate_key.unquoted_range();
                        locations.push(definition_location);
                    }
                }
            }
        }
    }

    Ok(locations)
}

/// Get the location of the crate path in the workspace.
///
/// ```toml
/// [workspace.dependencies]
/// tombi-ast = { path█ = "crates/tombi-ast" }
/// ```
fn goto_crate_package(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    assert!(
        matches_accessors!(accessors, ["workspace", "dependencies", _, "path"])
            || matches_accessors!(accessors, ["dependencies", _, "path"])
            || matches_accessors!(accessors, ["dev-dependencies", _, "path"])
            || matches_accessors!(accessors, ["build-dependencies", _, "path"])
    );

    let Some((_, value)) = tombi_schema_store::dig_accessors(workspace_document_tree, accessors)
    else {
        return Ok(None);
    };

    if value.value_type() == tombi_document_tree::ValueType::String {
        let subcrate_path = match value {
            tombi_document_tree::Value::String(path) => path,
            _ => unreachable!(),
        };

        if let Some((subcrate_cargo_toml_path, subcrate_document_tree)) = find_path_crate_cargo_toml(
            workspace_cargo_toml_path,
            std::path::Path::new(subcrate_path.value()),
            toml_version,
        ) {
            if let Some((_, tombi_document_tree::Value::String(package_name))) =
                tombi_document_tree::dig_keys(&subcrate_document_tree, &["package", "name"])
            {
                return Ok(Some(tombi_extension::DefinitionLocation {
                    uri: Url::from_file_path(subcrate_cargo_toml_path).unwrap(),
                    range: package_name.unquoted_range(),
                }));
            }
        }
    }

    Ok(None)
}

/// Sanitize the dependency key to be "dependencies" if it is "dev-dependencies" or "build-dependencies".
///
/// This is because the dependency key is always "dependencies" in the workspace Cargo.toml.
#[inline]
fn sanitize_dependency_key(key: &str) -> &str {
    if matches!(key, "dev-dependencies" | "build-dependencies") {
        "dependencies"
    } else {
        key
    }
}

fn find_package_cargo_toml_paths<'a>(
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
            let mut cargo_toml_paths = vec![];

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
                // Skip if the path doesn't contain Cargo.toml
                let cargo_toml_path = if candidate_path.is_dir() {
                    candidate_path.join("Cargo.toml")
                } else {
                    continue;
                };

                if !cargo_toml_path.exists() || !cargo_toml_path.is_file() {
                    continue;
                }

                // Check if the path is excluded
                let is_excluded = exclude_patterns.iter().any(|exclude_pattern| {
                    exclude_pattern.matches(&cargo_toml_path.to_string_lossy())
                });

                if !is_excluded {
                    cargo_toml_paths.push((member_pattern, cargo_toml_path));
                }
            }

            if !cargo_toml_paths.is_empty() {
                Some(cargo_toml_paths)
            } else {
                None
            }
        })
        .flatten()
}

fn goto_definition_for_workspace_cargo_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["workspace", "dependencies", _]) {
        goto_dependency_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            jump_to_subcrate,
        )
    } else if matches_accessors!(accessors, ["workspace", "dependencies", _, "path"]) {
        match goto_crate_package(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
        )? {
            Some(location) => Ok(vec![location]),
            None => Ok(Vec::with_capacity(0)),
        }
    } else if matches_accessors!(accessors, ["workspace", "members"])
        || matches_accessors!(accessors, ["workspace", "members", _])
    {
        goto_workspace_member_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            "members",
        )
        .map(|locations| locations.into_iter().filter_map(Into::into).collect_vec())
    } else if matches_accessors!(accessors, ["workspace", "default-members"])
        || matches_accessors!(accessors, ["workspace", "default-members", _])
    {
        goto_workspace_member_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            "default-members",
        )
        .map(|locations| locations.into_iter().filter_map(Into::into).collect_vec())
    } else {
        Ok(Vec::with_capacity(0))
    }
}

fn goto_definition_for_crate_cargo_toml(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    let location = if matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
    {
        return goto_dependency_crates(
            document_tree,
            accessors,
            cargo_toml_path,
            toml_version,
            jump_to_subcrate,
        );
    } else if accessors.last() == Some(&tombi_schema_store::Accessor::Key("workspace".to_string()))
    {
        goto_workspace(
            accessors,
            cargo_toml_path,
            get_workspace_path(document_tree),
            toml_version,
            jump_to_subcrate,
        )
    } else if matches_accessors!(accessors, ["dependencies", _, "path"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "path"])
        || matches_accessors!(accessors, ["build-dependencies", _, "path"])
    {
        goto_crate_package(document_tree, accessors, cargo_toml_path, toml_version)
    } else {
        Ok(None)
    }?;

    match location {
        Some(location) => Ok(vec![location]),
        None => Ok(Vec::with_capacity(0)),
    }
}

fn goto_workspace_member_crates(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    members_key: &'static str,
) -> Result<Vec<CrateLocation>, tower_lsp::jsonrpc::Error> {
    let member_patterns = if matches_accessors!(accessors, ["workspace", members_key, _]) {
        let Some((_, tombi_document_tree::Value::String(member))) =
            dig_accessors(workspace_document_tree, accessors)
        else {
            return Ok(Vec::with_capacity(0));
        };
        vec![member]
    } else {
        match tombi_document_tree::dig_keys(workspace_document_tree, &["workspace", members_key]) {
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

    let Some(workspace_dir_path) = workspace_cargo_toml_path.parent() else {
        return Ok(Vec::with_capacity(0));
    };

    let exclude_patterns =
        match tombi_document_tree::dig_keys(workspace_document_tree, &["workspace", "exclude"]) {
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
    for (_, cargo_toml_path) in
        find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(member_document_tree) = load_cargo_toml(&cargo_toml_path, toml_version) else {
            continue;
        };

        let Some((_, tombi_document_tree::Value::String(package_name))) =
            tombi_document_tree::dig_keys(&member_document_tree, &["package", "name"])
        else {
            continue;
        };

        locations.push(CrateLocation {
            cargo_toml_path,
            package_name_key_range: package_name.unquoted_range(),
        });
    }

    Ok(locations)
}

/// Get the workspace path from Cargo.toml
///
/// See: https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field
#[inline]
fn get_workspace_path(document_tree: &tombi_document_tree::DocumentTree) -> Option<&str> {
    dig_keys(document_tree, &["package", "workspace"]).and_then(|(_, workspace)| {
        if let tombi_document_tree::Value::String(workspace_path) = workspace {
            Some(workspace_path.value())
        } else {
            None
        }
    })
}
