use tombi_document_tree::{dig_keys, TableKind};
use tombi_schema_store::{dig_accessors, matches_accessors, Accessor, AccessorContext};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionOrCommand, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextDocumentIdentifier, TextEdit,
    WorkspaceEdit,
};

use crate::{find_workspace_cargo_toml, get_workspace_path};

pub enum CodeActionRefactorRewriteName {
    InheritFromWorkspace,
    InheritDependencyFromWorkspace,
    ConvertDependencyToTableFormat,
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
        }
    }
}

pub fn code_action(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    if !text_document.uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }
    let Some(cargo_toml_path) = text_document.uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut code_actions = Vec::new();

    if document_tree.contains_key("workspace") {
        code_actions.extend(code_actions_for_workspace_cargo_toml(
            text_document,
            document_tree,
            &cargo_toml_path,
            accessors,
            contexts,
            toml_version,
        ))
    } else {
        code_actions.extend(code_actions_for_crate_cargo_toml(
            text_document,
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
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    _cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    _toml_version: tombi_config::TomlVersion,
) -> Vec<CodeActionOrCommand> {
    let mut code_actions = Vec::new();

    if let Some(action) =
        crate_version_code_action(text_document, document_tree, accessors, contexts)
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    code_actions
}

fn code_actions_for_crate_cargo_toml(
    text_document: &TextDocumentIdentifier,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
) -> Vec<CodeActionOrCommand> {
    let mut code_actions = Vec::new();

    if let Some((_, workspace_document_tree)) = find_workspace_cargo_toml(
        crate_cargo_toml_path,
        get_workspace_path(crate_document_tree),
        toml_version,
    ) {
        // Add workspace-specific code actions here
        if let Some(action) = workspace_code_action(
            text_document,
            crate_document_tree,
            &workspace_document_tree,
            accessors,
            contexts,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(action));
        }

        if let Some(action) = use_workspace_depencency_code_action(
            text_document,
            crate_document_tree,
            &workspace_document_tree,
            accessors,
            contexts,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(action));
        }
    }

    // Add crate-specific code actions here
    if let Some(action) =
        crate_version_code_action(text_document, crate_document_tree, accessors, contexts)
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    code_actions
}

fn workspace_code_action(
    text_document: &TextDocumentIdentifier,
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
        kind: Some(tower_lsp::lsp_types::CodeActionKind::REFACTOR_REWRITE),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document.clone().uri,
                    version: None,
                },
                edits: vec![OneOf::Left(TextEdit {
                    range: (parent_key_context.range + value.symbol_range()).into(),
                    new_text: format!("{parent_key}.workspace = true"),
                })],
            }])),
            change_annotations: None,
        }),
        ..Default::default()
    })
}

fn use_workspace_depencency_code_action(
    text_document: &TextDocumentIdentifier,
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
                kind: Some(tower_lsp::lsp_types::CodeActionKind::REFACTOR_REWRITE),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document.clone().uri,
                            version: None,
                        },
                        edits: vec![OneOf::Left(TextEdit {
                            range: tombi_text::Range {
                                start: crate_key_context.range.start,
                                end: version.range().end,
                            }
                            .into(),
                            // NOTE: Convert to a workspace dependency to make it easier
                            //       to add other settings in the future.
                            new_text: format!("{} = {{ workspace = true }}", crate_name),
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
                    range: (crate_key_context.range + version.range()).into(),
                    new_text: format!("{} = {{ workspace = true }}", crate_name),
                }
            } else {
                TextEdit {
                    range: (key.range() + version.range()).into(),
                    new_text: "workspace = true".to_string(),
                }
            };

            return Some(CodeAction {
                title: CodeActionRefactorRewriteName::InheritDependencyFromWorkspace.to_string(),
                kind: Some(tower_lsp::lsp_types::CodeActionKind::REFACTOR_REWRITE),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document.clone().uri,
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
    text_document: &TextDocumentIdentifier,
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
                kind: Some(tower_lsp::lsp_types::CodeActionKind::REFACTOR_REWRITE),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document.clone().uri,
                            version: None,
                        },
                        edits: vec![
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::at(version.symbol_range().start).into(),
                                new_text: "{ version = ".to_string(),
                            }),
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::at(version.symbol_range().end).into(),
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
