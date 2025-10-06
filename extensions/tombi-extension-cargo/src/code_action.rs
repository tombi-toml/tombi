use tombi_document_tree::{dig_accessors, dig_keys, TableKind};
use tombi_extension::{
    CodeAction, CodeActionOrCommand, DocumentChanges, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use tombi_schema_store::{matches_accessors, Accessor, AccessorContext};
use tower_lsp::lsp_types::{CodeActionKind, OneOf, OptionalVersionedTextDocumentIdentifier};

use crate::{find_workspace_cargo_toml, get_workspace_path};

pub enum CodeActionRefactorRewriteName {
    /// Inherit from Workspace
    ///
    /// If you are using a crate that depends on the workspace, inherit the workspace's crate.
    ///
    /// Before
    ///
    /// ```toml
    /// # In your member crate's Cargo.toml
    /// [package]
    /// version = "1.0.0"
    /// ```
    ///
    /// After applying "Inherit from Workspace"
    ///
    /// ```toml
    /// # In your member crate's Cargo.toml
    /// [package]
    /// version.workspace = true
    /// ```
    InheritFromWorkspace,

    /// Inherit Dependency from Workspace
    ///
    /// If you are using a crate that depends on the workspace, inherit the workspace's crate.
    ///
    /// Before
    ///
    /// ```toml
    /// # In your member crate's Cargo.toml
    /// [dependencies]
    /// serde = "1.0"
    /// ```
    ///
    /// After applying "Inherit Dependency from Workspace"
    ///
    /// ```toml
    /// # In your member crate's Cargo.toml
    /// [dependencies]
    /// serde = { workspace = true }
    /// ```
    InheritDependencyFromWorkspace,

    /// Convert Dependency to Table Format
    ///
    /// Before
    ///
    /// ```toml
    /// [dependencies]
    /// serde = "1.0"
    /// ```
    ///
    /// After applying "Convert Dependency to Table Format"
    ///
    /// ```toml
    /// [dependencies]
    /// serde = { version = "1.0" }
    /// ```
    ConvertDependencyToTableFormat,

    /// Add to Workspace and Inherit Dependency
    ///
    /// Adds a dependency to [workspace.dependencies] and converts the member's
    /// dependency to `workspace = true` format.
    ///
    /// Before
    ///
    /// ```toml
    /// # Workspace Cargo.toml
    /// [workspace.dependencies]
    /// # (serde not present)
    ///
    /// # Member Cargo.toml
    /// [dependencies]
    /// serde = "1.0"
    /// ```
    ///
    /// After applying "Add to Workspace and Inherit Dependency"
    ///
    /// ```toml
    /// # Workspace Cargo.toml
    /// [workspace.dependencies]
    /// serde = "1.0"
    ///
    /// # Member Cargo.toml
    /// [dependencies]
    /// serde = { workspace = true }
    /// ```
    AddToWorkspaceAndInheritDependency,
}

impl std::fmt::Display for CodeActionRefactorRewriteName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CodeActionRefactorRewriteName::InheritFromWorkspace => {
                write!(f, "Inherit from Workspace")
            }
            CodeActionRefactorRewriteName::InheritDependencyFromWorkspace => {
                write!(f, "Inherit Dependency from Workspace")
            }
            CodeActionRefactorRewriteName::ConvertDependencyToTableFormat => {
                write!(f, "Convert Dependency to Table Format")
            }
            CodeActionRefactorRewriteName::AddToWorkspaceAndInheritDependency => {
                write!(f, "Add to Workspace and Inherit Dependency")
            }
        }
    }
}

pub fn code_action(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }
    let Some(cargo_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut code_actions = Vec::new();

    if document_tree.contains_key("workspace") {
        code_actions.extend(code_actions_for_workspace_cargo_toml(
            text_document_uri,
            document_tree,
            &cargo_toml_path,
            accessors,
            contexts,
            toml_version,
        ))
    } else {
        code_actions.extend(code_actions_for_crate_cargo_toml(
            text_document_uri,
            document_tree,
            &cargo_toml_path,
            accessors,
            contexts,
            toml_version,
        ));
    }

    Ok(if code_actions.is_empty() {
        None
    } else {
        Some(code_actions)
    })
}

fn code_actions_for_workspace_cargo_toml(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    _cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    _toml_version: tombi_config::TomlVersion,
) -> Vec<CodeActionOrCommand> {
    let mut code_actions = Vec::new();

    if let Some(action) =
        crate_version_code_action(text_document_uri, document_tree, accessors, contexts)
    {
        code_actions.push(CodeActionOrCommand::CodeAction(Box::new(action)));
    }

    code_actions
}

fn code_actions_for_crate_cargo_toml(
    text_document_uri: &tombi_uri::Uri,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
) -> Vec<CodeActionOrCommand> {
    let mut code_actions = Vec::new();

    if let Some((workspace_cargo_toml_path, workspace_document_tree)) = find_workspace_cargo_toml(
        crate_cargo_toml_path,
        get_workspace_path(crate_document_tree),
        toml_version,
    ) {
        // Add workspace-specific code actions here
        if let Some(action) = workspace_code_action(
            text_document_uri,
            crate_document_tree,
            &workspace_document_tree,
            accessors,
            contexts,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(Box::new(action)));
        }

        if let Some(action) = add_workspace_dependency_code_action(
            text_document_uri,
            crate_document_tree,
            &workspace_document_tree,
            &workspace_cargo_toml_path,
            accessors,
            contexts,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(Box::new(action)));
        }

        if let Some(action) = use_workspace_dependency_code_action(
            text_document_uri,
            crate_document_tree,
            &workspace_document_tree,
            accessors,
            contexts,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(Box::new(action)));
        }
    }

    // Add crate-specific code actions here
    if let Some(action) =
        crate_version_code_action(text_document_uri, crate_document_tree, accessors, contexts)
    {
        code_actions.push(CodeActionOrCommand::CodeAction(Box::new(action)));
    }

    code_actions
}

fn workspace_code_action(
    text_document_uri: &tombi_uri::Uri,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
) -> Option<CodeAction> {
    if accessors.len() < 2 {
        return None;
    }

    if !matches!(accessors.first(), Some(a) if a == &"package") {
        return None;
    }

    let (Accessor::Key(parent_key), AccessorContext::Key(parent_key_context)) =
        (&accessors[1], &contexts[1])
    else {
        return None;
    };

    if ![
        "authors",
        "categories",
        "description",
        "documentation",
        "edition",
        "exclude",
        "homepage",
        "include",
        "keywords",
        "license-file",
        "license",
        "publish",
        "readme",
        "repository",
        "rust-version",
        "version",
    ]
    .contains(&parent_key.as_str())
    {
        return None;
    }

    let (_, value) = dig_accessors(crate_document_tree, &accessors[..2])?;
    dig_keys(
        workspace_document_tree,
        &["workspace", "package", parent_key.as_str()],
    )?;

    if let tombi_document_tree::Value::Table(table) = value {
        if table.get("workspace").is_some() {
            return None; // Workspace already exists
        }
    };

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::InheritFromWorkspace.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE.clone()),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned().into(),
                    version: None,
                },
                edits: vec![OneOf::Left(TextEdit {
                    range: parent_key_context.range + value.symbol_range(),
                    new_text: format!("{parent_key}.workspace = true"),
                })],
            }])),
            change_annotations: None,
        }),
        ..Default::default()
    })
}

fn use_workspace_dependency_code_action(
    text_document_uri: &tombi_uri::Uri,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
) -> Option<CodeAction> {
    if accessors.len() < 2 {
        return None;
    }

    if !(matches_accessors!(accessors[..2], ["dependencies", _])
        || matches_accessors!(accessors[..2], ["dev-dependencies", _])
        || matches_accessors!(accessors[..2], ["build-dependencies", _]))
    {
        return None; // Not a dependency accessor
    }

    let Some((Accessor::Key(crate_name), value)) =
        dig_accessors(crate_document_tree, &accessors[..2])
    else {
        return None; // Not a string value
    };
    let AccessorContext::Key(crate_key_context) = &contexts[1] else {
        return None;
    };

    match value {
        tombi_document_tree::Value::String(version) => {
            dig_keys(
                workspace_document_tree,
                &["workspace", "dependencies", crate_name],
            )?;
            return Some(CodeAction {
                title: CodeActionRefactorRewriteName::InheritDependencyFromWorkspace.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE.clone()),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document_uri.to_owned().into(),
                            version: None,
                        },
                        edits: vec![OneOf::Left(TextEdit {
                            range: tombi_text::Range {
                                start: crate_key_context.range.start,
                                end: version.range().end,
                            },
                            // NOTE: Convert to a workspace dependency to make it easier
                            //       to add other settings in the future.
                            new_text: format!("{crate_name} = {{ workspace = true }}"),
                        })],
                    }])),
                    change_annotations: None,
                }),
                ..Default::default()
            });
        }
        tombi_document_tree::Value::Table(table) => {
            if table.get("workspace").is_some() {
                return None; // Already a workspace dependency
            }

            dig_keys(
                workspace_document_tree,
                &["workspace", "dependencies", crate_name],
            )?;

            let Some((key, version)) = table.get_key_value("version") else {
                return None; // No version to inherit
            };

            let text_edit = if table.kind() == TableKind::KeyValue {
                TextEdit {
                    range: crate_key_context.range + version.range(),
                    new_text: format!("{crate_name} = {{ workspace = true }}"),
                }
            } else {
                TextEdit {
                    range: key.range() + version.range(),
                    new_text: "workspace = true".to_string(),
                }
            };

            return Some(CodeAction {
                title: CodeActionRefactorRewriteName::InheritDependencyFromWorkspace.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE.clone()),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document_uri.to_owned().into(),
                            version: None,
                        },
                        edits: vec![OneOf::Left(text_edit)],
                    }])),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        _ => {}
    }

    None
}

fn crate_version_code_action(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    _contexts: &[AccessorContext],
) -> Option<CodeAction> {
    if matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
        || matches_accessors!(accessors, ["workspace", "dependencies", _])
    {
        if let Some((_, tombi_document_tree::Value::String(version))) =
            dig_accessors(document_tree, accessors)
        {
            return Some(CodeAction {
                title: CodeActionRefactorRewriteName::ConvertDependencyToTableFormat.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE.clone()),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document_uri.to_owned().into(),
                            version: None,
                        },
                        edits: vec![
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::at(version.symbol_range().start),
                                new_text: "{ version = ".to_string(),
                            }),
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::at(version.symbol_range().end),
                                new_text: " }".to_string(),
                            }),
                        ],
                    }])),
                    change_annotations: None,
                }),
                ..Default::default()
            });
        }
    }
    None
}

/// Calculate the insertion index for a new crate in workspace.dependencies
/// based on version-sort rules.
///
/// Returns the index where the new crate should be inserted to maintain
/// version-sort order.
fn calculate_insertion_index(existing_crate_names: &[&str], new_crate_name: &str) -> usize {
    existing_crate_names
        .iter()
        .position(|&existing| {
            tombi_version_sort::version_sort(new_crate_name, existing) == std::cmp::Ordering::Less
        })
        .unwrap_or(existing_crate_names.len())
}

/// Add a dependency to workspace.dependencies and convert member's dependency
/// to workspace = true format.
///
/// This code action is provided when:
/// - The cursor is on a dependency in member Cargo.toml
/// - The dependency is not yet registered in workspace.dependencies
/// - The dependency is not already using workspace = true
fn add_workspace_dependency_code_action(
    text_document_uri: &tombi_uri::Uri,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
) -> Option<CodeAction> {
    // Check if accessors match dependency patterns
    if accessors.len() < 2 {
        return None;
    }

    if !(matches_accessors!(accessors[..2], ["dependencies", _])
        || matches_accessors!(accessors[..2], ["dev-dependencies", _])
        || matches_accessors!(accessors[..2], ["build-dependencies", _]))
    {
        return None;
    }

    // Extract crate name and value from member Cargo.toml
    let Some((Accessor::Key(crate_name), crate_value)) =
        dig_accessors(crate_document_tree, &accessors[..2])
    else {
        return None;
    };

    // Check if already using workspace = true
    if let tombi_document_tree::Value::Table(table) = crate_value {
        if table.get("workspace").is_some() {
            return None; // Already using workspace inheritance
        }

        if table.get("path").is_some() {
            return None; // Already using path dependency
        }
    }

    // Check if crate already exists in workspace.dependencies
    if dig_keys(
        workspace_document_tree,
        &["workspace", "dependencies", crate_name],
    )
    .is_some()
    {
        return None; // Already in workspace.dependencies
    }

    // Generate workspace Cargo.toml URI
    let Ok(workspace_uri) = tombi_uri::Uri::from_file_path(workspace_cargo_toml_path) else {
        return None;
    };

    // Generate workspace edit for workspace.dependencies
    let workspace_edit =
        generate_workspace_dependencies_edit(workspace_document_tree, crate_name, crate_value)?;

    // Generate member edit to convert to workspace = true
    let member_edit = generate_member_workspace_true_edit(&contexts[1], crate_name, crate_value)?;

    // Build WorkspaceEdit with both file changes
    let workspace_edit = WorkspaceEdit {
        changes: None,
        document_changes: Some(DocumentChanges::Edits(vec![
            TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: workspace_uri.into(),
                    version: None,
                },
                edits: vec![OneOf::Left(workspace_edit)],
            },
            TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned().into(),
                    version: None,
                },
                edits: vec![OneOf::Left(member_edit)],
            },
        ])),
        change_annotations: None,
    };

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::AddToWorkspaceAndInheritDependency.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE.clone()),
        diagnostics: None,
        edit: Some(workspace_edit),
        ..Default::default()
    })
}

/// Generate TextEdit for adding dependency to workspace.dependencies
fn generate_workspace_dependencies_edit(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    crate_name: &str,
    crate_value: &tombi_document_tree::Value,
) -> Option<TextEdit> {
    // Get or prepare workspace.dependencies section
    let workspace_deps = dig_keys(workspace_document_tree, &["workspace", "dependencies"]);

    // Extract dependency value to copy to workspace
    //
    // NOTE: By convention, dependencies are not written as inline tables,
    //       so this logic assumes the value is represented as a Table.
    //
    let dependency_text = format!("{} = {}\n", crate_name, crate_value.to_string());

    // Determine insertion position
    let insertion_range = if let Some((_, deps_table)) = workspace_deps {
        if let tombi_document_tree::Value::Table(table) = deps_table {
            // Get existing crate names and calculate insertion index
            let existing_crates: Vec<&str> = table.keys().map(|key| key.value.as_str()).collect();
            let insertion_index = calculate_insertion_index(&existing_crates, crate_name);

            // Find insertion position in the actual table
            if insertion_index == 0 {
                if table.is_empty() {
                    tombi_text::Range::at(table.range().end)
                } else {
                    let range = table.keys().next().unwrap().range();
                    tombi_text::Range::at(range.start)
                }
            } else if insertion_index >= existing_crates.len() {
                // Insert at the end of the table
                let range = table.range();
                tombi_text::Range::at(range.end)
            } else {
                // Insert before the crate at insertion_index
                if let Some((target_key, _)) = table.get_key_value(existing_crates[insertion_index])
                {
                    let range = target_key.range();
                    tombi_text::Range::at(range.start)
                } else {
                    let range = table.range();
                    tombi_text::Range::at(range.end)
                }
            }
        } else {
            return None;
        }
    } else {
        // workspace.dependencies section doesn't exist, need to create it
        // For now, return None - this will be handled in a future enhancement
        return None;
    };

    Some(TextEdit {
        range: insertion_range,
        new_text: dependency_text,
    })
}

/// Generate TextEdit for converting member dependency to workspace = true
fn generate_member_workspace_true_edit(
    accessor_context: &AccessorContext,
    crate_name: &str,
    crate_value: &tombi_document_tree::Value,
) -> Option<TextEdit> {
    let AccessorContext::Key(crate_key_context) = accessor_context else {
        return None;
    };

    Some(TextEdit {
        range: crate_key_context.range + crate_value.range(),
        new_text: format!("{} = {{ workspace = true }}", crate_name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_action_refactor_rewrite_name_display() {
        assert_eq!(
            CodeActionRefactorRewriteName::AddToWorkspaceAndInheritDependency.to_string(),
            "Add to Workspace and Inherit Dependency"
        );
    }

    #[test]
    fn test_calculate_insertion_index_empty_list() {
        let existing: Vec<&str> = vec![];
        let result = calculate_insertion_index(&existing, "serde");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_insertion_index_insert_at_beginning() {
        let existing = vec!["tokio", "tracing"];
        let result = calculate_insertion_index(&existing, "serde");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_insertion_index_insert_at_end() {
        let existing = vec!["serde", "tokio"];
        let result = calculate_insertion_index(&existing, "tracing");
        assert_eq!(result, 2);
    }

    #[test]
    fn test_calculate_insertion_index_insert_in_middle() {
        let existing = vec!["serde", "tracing"];
        let result = calculate_insertion_index(&existing, "tokio");
        assert_eq!(result, 1);
    }

    #[test]
    fn test_calculate_insertion_index_with_underscores() {
        let existing = vec!["serde", "tokio"];
        let result = calculate_insertion_index(&existing, "serde_json");
        assert_eq!(result, 1);
    }

    #[test]
    fn test_calculate_insertion_index_with_hyphens() {
        let existing = vec!["serde", "tower"];
        let result = calculate_insertion_index(&existing, "tower-lsp");
        assert_eq!(result, 2);
    }

    #[test]
    fn test_calculate_insertion_index_with_numbers() {
        let existing = vec!["tokio", "tracing"];
        let result = calculate_insertion_index(&existing, "tokio1");
        assert_eq!(result, 1);
    }
}
