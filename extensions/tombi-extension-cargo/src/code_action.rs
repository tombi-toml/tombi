use tombi_document_tree_syntax::{TableKind, Value, dig_accessors, dig_keys};
use tombi_extension::{
    CodeAction, CodeActionDisabled, CodeActionKind, CodeActionOrCommand, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use tombi_schema_store::{Accessor, AccessorContext, matches_accessors};

use crate::{
    dependency_parent_accessors, fetch_crates_io_crate, find_workspace_cargo_toml,
    get_workspace_cargo_toml_path, is_any_dependency_accessor, load_cargo_toml_with_root,
};

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
    /// serde.workspace = true
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
    /// dependency to workspace inheritance format.
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
    /// serde.workspace = true
    /// ```
    AddToWorkspaceAndInheritDependency,

    /// Update Dependency to Latest Version
    ///
    /// Before
    ///
    /// ```toml
    /// [dependencies]
    /// serde = "1.0"
    /// ```
    ///
    /// After applying "Update Dependency to Latest Version"
    ///
    /// ```toml
    /// [dependencies]
    /// serde = "1.0.228"
    /// ```
    UpdateDependencyToLatestVersion,
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
            CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion => {
                write!(f, "Update Dependency to Latest Version")
            }
        }
    }
}

pub async fn code_action(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    root: &tombi_ast_syntax::Root<'_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.code_action())
        .map(|code_action| code_action.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }
    let Some(cargo_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut code_actions = Vec::new();

    if document_tree.contains_key("workspace") {
        code_actions.extend(
            code_actions_for_workspace_cargo_toml(
                text_document_uri,
                converter,
                document_tree,
                accessors,
                offline,
                cache_options,
                features,
            )
            .await?,
        )
    } else {
        code_actions.extend(
            code_actions_for_crate_cargo_toml(
                text_document_uri,
                converter,
                root,
                document_tree,
                &cargo_toml_path,
                accessors,
                contexts,
                toml_version,
                offline,
                cache_options,
                features,
            )
            .await?,
        );
    }

    Ok((!code_actions.is_empty()).then_some(code_actions))
}

async fn code_actions_for_workspace_cargo_toml(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<CodeActionOrCommand>, tower_lsp::jsonrpc::Error> {
    let mut code_actions = Vec::new();

    let code_action_features = features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.code_action());

    if code_action_features
        .as_ref()
        .and_then(|code_action| code_action.convert_dependency_to_table_format())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
        && let Some(action) = convert_dependency_to_table_format_code_action(
            text_document_uri,
            converter,
            document_tree,
            accessors,
        )
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    if code_action_features
        .as_ref()
        .and_then(|code_action| code_action.update_dependency_to_latest_version())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
        && let Some(action) = update_dependency_to_latest_version_code_action(
            text_document_uri,
            converter,
            document_tree,
            accessors,
            offline,
            cache_options,
        )
        .await?
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    Ok(code_actions)
}

async fn code_actions_for_crate_cargo_toml(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    _root: &tombi_ast_syntax::Root<'_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<CodeActionOrCommand>, tower_lsp::jsonrpc::Error> {
    let mut code_actions = Vec::new();

    let code_action_features = features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.code_action());

    if let Some(workspace_cargo_toml_path) = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
        |workspace_cargo_toml_path, _, _| workspace_cargo_toml_path.to_path_buf(),
    ) && let Some(workspace_code_actions) = load_cargo_toml_with_root(
        &workspace_cargo_toml_path,
        toml_version,
        |workspace_root, workspace_document_tree, workspace_line_index| {
            let workspace_converter =
                tombi_extension::SpanConverter::new(workspace_line_index, converter.encoding());
            let mut code_actions = Vec::new();

            // Add workspace-specific code actions here
            if code_action_features
                .as_ref()
                .and_then(|code_action| code_action.inherit_from_workspace())
                .map(|feature| feature.enabled())
                .unwrap_or_default()
                .value()
                && let Some(action) = inherit_from_workspace_code_action(
                    text_document_uri,
                    converter,
                    document_tree,
                    accessors,
                    contexts,
                    workspace_document_tree,
                )
            {
                code_actions.push(CodeActionOrCommand::CodeAction(action));
            }

            if code_action_features
                .as_ref()
                .and_then(|code_action| code_action.add_to_workspace_and_inherit_dependency())
                .map(|feature| feature.enabled())
                .unwrap_or_default()
                .value()
                && let Some(action) = add_to_workspace_and_inherit_dependency_code_action(
                    text_document_uri,
                    converter,
                    document_tree,
                    accessors,
                    contexts,
                    &workspace_cargo_toml_path,
                    workspace_converter,
                    &workspace_root,
                    workspace_document_tree,
                )
            {
                code_actions.push(CodeActionOrCommand::CodeAction(action));
            }

            if code_action_features
                .as_ref()
                .and_then(|code_action| code_action.inherit_dependency_from_workspace())
                .map(|feature| feature.enabled())
                .unwrap_or_default()
                .value()
                && let Some(action) = inherit_dependency_from_workspace_code_action(
                    text_document_uri,
                    converter,
                    document_tree,
                    cargo_toml_path,
                    accessors,
                    contexts,
                    &workspace_cargo_toml_path,
                    workspace_document_tree,
                    toml_version,
                )
            {
                code_actions.push(CodeActionOrCommand::CodeAction(action));
            }

            code_actions
        },
    ) {
        code_actions.extend(workspace_code_actions);
    }

    // Add crate-specific code actions here
    if code_action_features
        .as_ref()
        .and_then(|code_action| code_action.convert_dependency_to_table_format())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
        && let Some(action) = convert_dependency_to_table_format_code_action(
            text_document_uri,
            converter,
            document_tree,
            accessors,
        )
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    if code_action_features
        .as_ref()
        .and_then(|code_action| code_action.update_dependency_to_latest_version())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
        && let Some(action) = update_dependency_to_latest_version_code_action(
            text_document_uri,
            converter,
            document_tree,
            accessors,
            offline,
            cache_options,
        )
        .await?
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    Ok(code_actions)
}

async fn update_dependency_to_latest_version_code_action(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<CodeAction>, tower_lsp::jsonrpc::Error> {
    let dependency_accessors = if is_any_dependency_accessor(accessors) {
        accessors
    } else if matches!(accessors.last(), Some(Accessor::Key(key)) if key == "version")
        && let parent_accessors = dependency_parent_accessors(accessors)
        && is_any_dependency_accessor(parent_accessors)
    {
        parent_accessors
    } else {
        return Ok(None);
    };

    let Some((Accessor::Key(dependency_key), dependency_value)) =
        dig_accessors(document_tree, dependency_accessors)
    else {
        return Ok(None);
    };

    let (crate_name, version) = match dependency_value {
        tombi_document_tree_syntax::Value::String(version) => (dependency_key.as_str(), version),
        tombi_document_tree_syntax::Value::Table(table)
            if !(table.get("path").is_some()
                || table.get("git").is_some()
                || matches!(
                    table.get("workspace"),
                    Some(Value::Boolean(workspace)) if workspace.value()
                )) =>
        {
            match table.get("version") {
                Some(tombi_document_tree_syntax::Value::String(version)) => {
                    let crate_name = match table.get("package") {
                        Some(tombi_document_tree_syntax::Value::String(package)) => package.value(),
                        _ => dependency_key.as_str(),
                    };
                    (crate_name, version)
                }
                _ => return Ok(None),
            }
        }
        _ => return Ok(None),
    };

    let Some(latest_version) = fetch_crates_io_crate(crate_name, offline, cache_options)
        .await?
        .and_then(|response| response.crate_info.max_version)
    else {
        return Ok(None);
    };

    Ok(Some(CodeAction {
        title: CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned(),
                    version: None,
                },
                edits: vec![OneOf::Left(converter.text_edit(TextEdit {
                    span: version.span(),
                    new_text: format!("\"{latest_version}\""),
                }))],
            }])),
        }),
        disabled: (latest_version == version.value()).then(|| CodeActionDisabled {
            reason: "Already at latest version".to_string(),
        }),
    }))
}

/// Convert a package field to inherit from workspace configuration.
///
/// Before
///
/// ```toml
/// [package]
/// version = "1.0.0"
/// ```
///
/// After applying "Convert Package Field to Inherit from Workspace"
///
/// ```toml
/// [package]
/// version.workspace = true
/// ```
///
fn inherit_from_workspace_code_action(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
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

    let (_, value) = dig_accessors(document_tree, &accessors[..2])?;
    dig_keys(
        workspace_document_tree,
        &["workspace", "package", parent_key.as_str()],
    )?;

    if let tombi_document_tree_syntax::Value::Table(table) = value
        && table.get("workspace").is_some()
    {
        return None; // Workspace already exists
    };

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::InheritFromWorkspace.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned(),
                    version: None,
                },
                edits: vec![OneOf::Left(converter.text_edit(TextEdit {
                    span: (parent_key_context.span + value.symbol_span()),
                    new_text: format!("{parent_key}.workspace = true"),
                }))],
            }])),
        }),
        ..Default::default()
    })
}

fn inherit_dependency_from_workspace_code_action(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    crate_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    _crate_cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    _workspace_cargo_toml_path: &std::path::Path,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    _toml_version: tombi_config::TomlVersion,
) -> Option<CodeAction> {
    if accessors.len() < 2 {
        return None;
    }

    let is_target_dependency = accessors.len() >= 4
        && (matches_accessors!(accessors[..4], ["target", _, "dependencies", _])
            || matches_accessors!(accessors[..4], ["target", _, "dev-dependencies", _])
            || matches_accessors!(accessors[..4], ["target", _, "build-dependencies", _]));

    if !(matches_accessors!(accessors[..2], ["dependencies", _])
        || matches_accessors!(accessors[..2], ["dev-dependencies", _])
        || matches_accessors!(accessors[..2], ["build-dependencies", _])
        || is_target_dependency)
    {
        return None; // Not a dependency accessor
    }

    let offset = if is_target_dependency { 2 } else { 0 };

    let Some((Accessor::Key(crate_name), value)) =
        dig_accessors(crate_document_tree, &accessors[..2 + offset])
    else {
        return None; // Not a string value
    };
    let AccessorContext::Key(crate_key_context) = &contexts[1 + offset] else {
        return None;
    };
    dig_keys(
        workspace_document_tree,
        &["workspace", "dependencies", crate_name],
    )?;

    match value {
        tombi_document_tree_syntax::Value::String(version) => {
            return Some(CodeAction {
                title: CodeActionRefactorRewriteName::InheritDependencyFromWorkspace.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document_uri.to_owned(),
                            version: None,
                        },
                        edits: vec![OneOf::Left(converter.text_edit(TextEdit {
                            span: tombi_text::Span::new(
                                crate_key_context.span.start,
                                version.span().end,
                            ),
                            new_text: format!("{crate_name}.workspace = true"),
                        }))],
                    }])),
                }),
                ..Default::default()
            });
        }
        tombi_document_tree_syntax::Value::Table(table) => {
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

            let edits = if matches!(table.kind(), TableKind::InlineTable { .. }) {
                vec![OneOf::Left(converter.text_edit(TextEdit {
                    span: (crate_key_context.span + table.symbol_span()),
                    new_text: render_inherited_dependency_inline_table(crate_name, table),
                }))]
            } else {
                vec![OneOf::Left(converter.text_edit(TextEdit {
                    span: (key.span() + version.span()),
                    new_text: "workspace = true".to_string(),
                }))]
            };

            return Some(CodeAction {
                title: CodeActionRefactorRewriteName::InheritDependencyFromWorkspace.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document_uri.to_owned(),
                            version: None,
                        },
                        edits,
                    }])),
                }),
                ..Default::default()
            });
        }
        _ => {}
    }

    None
}
fn render_inherited_dependency_inline_table(
    crate_name: &str,
    dependency_table: &tombi_document_tree_syntax::Table<'_>,
) -> String {
    let mut entries = vec!["workspace = true".to_string()];

    for (key, value) in dependency_table.key_values() {
        match key.value() {
            "version" | "workspace" => {}
            _ => entries.push(format!("{} = {}", key.value(), value)),
        }
    }

    format!("{crate_name} = {{ {} }}", entries.join(", "))
}

/// Convert a dependency version to a table format.
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
///
fn convert_dependency_to_table_format_code_action(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
) -> Option<CodeAction> {
    if (crate::is_any_dependency_accessor(accessors))
        && let Some((_, tombi_document_tree_syntax::Value::String(version))) =
            dig_accessors(document_tree, accessors)
    {
        return Some(CodeAction {
            title: CodeActionRefactorRewriteName::ConvertDependencyToTableFormat.to_string(),
            kind: Some(CodeActionKind::REFACTOR_REWRITE),
            edit: Some(WorkspaceEdit {
                changes: None,
                document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: text_document_uri.to_owned(),
                        version: None,
                    },
                    edits: vec![
                        OneOf::Left(converter.text_edit(TextEdit {
                            span: tombi_text::Span::empty(version.span().start),
                            new_text: "{ version = ".to_string(),
                        })),
                        OneOf::Left(converter.text_edit(TextEdit {
                            span: tombi_text::Span::empty(version.span().end),
                            new_text: " }".to_string(),
                        })),
                    ],
                }])),
            }),
            ..Default::default()
        });
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

/// Get AST InlineTable from document tree span
/// First finds the span in document_tree, then locates the corresponding AST node
fn get_ast_inline_table_from_document_tree<'t>(
    root: &tombi_ast_syntax::Root<'t>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    keys: &[&str],
) -> Option<tombi_ast_syntax::InlineTable<'t>> {
    // Get the value from document tree to find its span
    let (_, value) = tombi_document_tree_syntax::dig_keys(document_tree, keys)?;

    let tombi_document_tree_syntax::Value::Table(doc_table) = value else {
        return None;
    };

    // Get the span of the inline table in the document tree
    let target_span = doc_table.span();

    root.inline_table_at_span(target_span)
}

/// Calculate insertion position and text for inline table insertion with comma handling
/// Uses tombi_ast_syntax API to properly handle commas and formatting
fn calculate_inline_table_insertion(
    ast_inline_table: &tombi_ast_syntax::InlineTable<'_>,
    insertion_index: usize,
    new_entry_text: &str,
) -> Option<(tombi_text::Offset, String)> {
    use tombi_ast_syntax::AstNode;

    let key_values_with_comma: Vec<_> = ast_inline_table.key_values_with_comma().collect();

    if key_values_with_comma.is_empty() {
        // Empty inline table - insert after opening brace
        // { } -> { serde = "1.0" }
        return if let Some(dangling_comment) = ast_inline_table
            .dangling_comment_groups()
            .last()
            .and_then(|group| group.comments().last())
        {
            Some((
                dangling_comment.syntax().span().end,
                format!("\n\n{},\n", new_entry_text),
            ))
        } else {
            Some((
                ast_inline_table.brace_start()?.span().end,
                new_entry_text.to_string(),
            ))
        };
    }

    if insertion_index == 0 {
        // Insert at the beginning
        // { tokio = "1.0" } -> { serde = "1.0", tokio = "1.0" }
        let (first_key_value, _) = key_values_with_comma.first()?;
        let insert_offset = first_key_value.syntax().span().start;
        let new_text = format!("{},\n", new_entry_text);
        return Some((insert_offset, new_text));
    }

    if insertion_index >= key_values_with_comma.len() {
        // Insert at the end
        // { serde = "1.0" } -> { serde = "1.0", tokio = "1.0" }
        let (last_key_value, last_comma) = key_values_with_comma.last()?;
        if let Some(last_comma) = last_comma {
            let insert_offset = last_comma.span().end;
            let new_text = format!("\n{}, ", new_entry_text);
            return Some((insert_offset, new_text));
        } else {
            let insert_offset = last_key_value.syntax().span().end;
            let new_text = format!(", {}", new_entry_text);
            return Some((insert_offset, new_text));
        }
    }

    // Insert in the middle
    // { serde = "1.0", tracing = "0.1" } -> { serde = "1.0", tokio = "1.0", tracing = "0.1" }
    let (target_key_value, target_comma) = key_values_with_comma.get(insertion_index)?;
    let insert_offset = if let Some(target_comma) = target_comma {
        target_comma.span().end
    } else {
        target_key_value.syntax().span().end
    };
    let new_text = format!("\n{},\n", new_entry_text);
    Some((insert_offset, new_text))
}

/// Add a dependency to workspace.dependencies and convert member's dependency
/// to workspace inheritance format.
///
/// This code action is provided when:
/// - The cursor is on a dependency in member Cargo.toml
/// - The dependency is not yet registered in workspace.dependencies
/// - The dependency is not already using workspace inheritance
///
/// Before
///
/// ```toml
/// [dependencies]
/// serde = "1.0"
/// ```
///
/// After applying "Add to Workspace and Inherit Dependency"
///
/// ```toml
/// [workspace.dependencies]
/// serde = "1.0"
///
/// [dependencies]
/// serde.workspace = true
/// ```
///
fn add_to_workspace_and_inherit_dependency_code_action(
    text_document_uri: &tombi_uri::Uri,
    converter: tombi_extension::SpanConverter<'_, '_>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    workspace_cargo_toml_path: &std::path::Path,
    workspace_converter: tombi_extension::SpanConverter<'_, '_>,
    workspace_root: &tombi_ast_syntax::Root<'_>,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
) -> Option<CodeAction> {
    // Check if accessors match dependency patterns
    if accessors.len() < 2 {
        return None;
    }

    let is_target_dependency = accessors.len() >= 4
        && (matches_accessors!(accessors[..4], ["target", _, "dependencies", _])
            || matches_accessors!(accessors[..4], ["target", _, "dev-dependencies", _])
            || matches_accessors!(accessors[..4], ["target", _, "build-dependencies", _]));

    if !(matches_accessors!(accessors[..2], ["dependencies", _])
        || matches_accessors!(accessors[..2], ["dev-dependencies", _])
        || matches_accessors!(accessors[..2], ["build-dependencies", _])
        || is_target_dependency)
    {
        return None;
    }

    let offset = if is_target_dependency { 2 } else { 0 };

    // Extract crate name and value from member Cargo.toml
    let Some((Accessor::Key(crate_name), crate_value)) =
        dig_accessors(document_tree, &accessors[..2 + offset])
    else {
        return None;
    };

    // Check if already using workspace = true
    if let tombi_document_tree_syntax::Value::Table(table) = crate_value {
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
    let workspace_edit = generate_workspace_dependencies_edit(
        workspace_root,
        workspace_document_tree,
        crate_name,
        crate_value,
    )?;

    // Generate member edit to convert to workspace inheritance.
    let member_edit =
        generate_member_workspace_true_edit(crate_name, crate_value, &contexts[1 + offset])?;

    // Build WorkspaceEdit with both file changes
    let workspace_edit = WorkspaceEdit {
        changes: None,
        document_changes: Some(DocumentChanges::Edits(vec![
            TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: workspace_uri,
                    version: None,
                },
                edits: vec![OneOf::Left(workspace_converter.text_edit(workspace_edit))],
            },
            TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned(),
                    version: None,
                },
                edits: vec![OneOf::Left(converter.text_edit(member_edit))],
            },
        ])),
    };

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::AddToWorkspaceAndInheritDependency.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(workspace_edit),
        ..Default::default()
    })
}

/// Generate TextEdit for adding dependency to workspace.dependencies
fn generate_workspace_dependencies_edit(
    workspace_root: &tombi_ast_syntax::Root<'_>,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    crate_name: &str,
    crate_value: &tombi_document_tree_syntax::Value<'_>,
) -> Option<TextEdit> {
    // Get or prepare workspace.dependencies section
    let workspace_deps = dig_keys(workspace_document_tree, &["workspace", "dependencies"]);

    let Some((_, deps_table)) = workspace_deps else {
        // NOTE: `workspace.dependencies` section doesn't exist, need to create it
        //       For now, return None - this will be handled in a future enhancement
        return None;
    };

    let tombi_document_tree_syntax::Value::Table(table) = deps_table else {
        return None;
    };

    // Get existing crate names and calculate insertion index
    let existing_crates: Vec<&str> = table.keys().map(|key| key.value()).collect();
    let insertion_index = calculate_insertion_index(&existing_crates, crate_name);
    let crate_value_text = crate_value.to_string();

    // Find insertion position in the actual table
    let (insertion_span, new_text) = if table.kind() == TableKind::Table {
        let insertion_span = if insertion_index == 0 {
            if table.is_empty() {
                tombi_text::Span::empty(table.span().end)
            } else {
                let span = table.keys().next().unwrap().span();
                tombi_text::Span::empty(span.start)
            }
        } else if insertion_index >= existing_crates.len() {
            // Insert at the end of the table
            let span = table.span();
            tombi_text::Span::empty(span.end)
        } else {
            // Insert before the crate at insertion_index
            if let Some((target_key, _)) = table.get_key_value(existing_crates[insertion_index]) {
                let span = target_key.span();
                tombi_text::Span::empty(span.start)
            } else {
                let span = table.span();
                tombi_text::Span::empty(span.end)
            }
        };

        (
            insertion_span,
            format!("{crate_name} = {crate_value_text}\n"),
        )
    } else if matches!(table.kind(), TableKind::InlineTable { .. }) {
        // Handle InlineTable case using AST for accurate comma handling
        let ast_inline_table = get_ast_inline_table_from_document_tree(
            workspace_root,
            workspace_document_tree,
            &["workspace", "dependencies"],
        )?;

        let new_entry_text = format!("{crate_name} = {crate_value_text}");
        let (insertion_offset, new_text) =
            calculate_inline_table_insertion(&ast_inline_table, insertion_index, &new_entry_text)?;

        (tombi_text::Span::empty(insertion_offset), new_text)
    } else {
        return None;
    };

    Some(TextEdit {
        span: insertion_span,
        new_text,
    })
}

/// Generate TextEdit for converting member dependency to workspace inheritance.
fn generate_member_workspace_true_edit(
    crate_name: &str,
    crate_value: &tombi_document_tree_syntax::Value<'_>,
    accessor_context: &AccessorContext,
) -> Option<TextEdit> {
    let AccessorContext::Key(crate_key_context) = accessor_context else {
        return None;
    };

    match crate_value {
        tombi_document_tree_syntax::Value::String(_) => Some(TextEdit {
            span: (crate_key_context.span + crate_value.symbol_span()),
            new_text: format!("{crate_name}.workspace = true"),
        }),
        tombi_document_tree_syntax::Value::Table(table)
            if matches!(table.kind(), TableKind::InlineTable { .. }) =>
        {
            Some(TextEdit {
                span: (crate_key_context.span + table.symbol_span()),
                new_text: render_inherited_dependency_inline_table(crate_name, table),
            })
        }
        tombi_document_tree_syntax::Value::Table(table) => {
            let (key, version) = table.get_key_value("version")?;

            Some(TextEdit {
                span: (key.span() + version.span()),
                new_text: "workspace = true".to_string(),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_ast_syntax::AstNode as _;
    use tombi_document_tree_syntax::{TryIntoDocumentTree, dig_keys};
    use tombi_schema_store::{AccessorContext, AccessorKeyKind, KeyContext};
    use tombi_text::{Offset, Span};

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

    fn with_dependency_value(
        source: &str,
        crate_name: &str,
        f: impl FnOnce(&str, &tombi_document_tree_syntax::Value<'_>, &AccessorContext),
    ) {
        let source = source.trim();
        let parsed = tombi_parser::parse(source);
        let root = parsed.root();
        let decoded = root.decode_strings(tombi_config::TomlVersion::default());
        let document_tree = root
            .try_into_document_tree(tombi_config::TomlVersion::default(), &decoded)
            .expect("expected document tree");
        let (_, value) =
            dig_keys(&document_tree, &["dependencies", crate_name]).expect("expected dependency");
        let start = source.find(crate_name).expect("expected crate name");
        let end = start + crate_name.len();
        let accessor_context = AccessorContext::Key(KeyContext {
            kind: AccessorKeyKind::KeyValue,
            span: Span::new(Offset::of(&source[..start]), Offset::of(&source[..end])),
        });

        f(source, value, &accessor_context);
    }

    fn apply_text_edit(source: &str, edit: &TextEdit) -> String {
        let mut text = source.to_string();
        text.replace_range(std::ops::Range::<usize>::from(edit.span), &edit.new_text);
        text
    }

    #[test]
    fn generate_member_workspace_true_edit_preserves_inline_table_keys() {
        with_dependency_value(
            r#"[dependencies]
serde = { version = "1.0", features = ["derive"] }"#,
            "serde",
            |source, value, accessor_context| {
                let edit = generate_member_workspace_true_edit("serde", value, accessor_context)
                    .expect("expected edit");

                assert_eq!(
                    apply_text_edit(source, &edit),
                    r#"[dependencies]
serde = { workspace = true, features = ["derive"] }"#
                );
            },
        );
    }

    #[test]
    fn generate_member_workspace_true_edit_preserves_inline_table_comment() {
        with_dependency_value(
            r#"[dependencies]
serde = { version = "1.0" } # comment"#,
            "serde",
            |source, value, accessor_context| {
                let edit = generate_member_workspace_true_edit("serde", value, accessor_context)
                    .expect("expected edit");

                assert_eq!(
                    apply_text_edit(source, &edit),
                    r#"[dependencies]
serde = { workspace = true } # comment"#
                );
            },
        );
    }

    #[test]
    fn generate_member_workspace_true_edit_replaces_dotted_version_key() {
        with_dependency_value(
            r#"[dependencies]
serde.version = "1.0"
serde.features = ["derive"]"#,
            "serde",
            |source, value, accessor_context| {
                let edit = generate_member_workspace_true_edit("serde", value, accessor_context)
                    .expect("expected edit");

                assert_eq!(
                    apply_text_edit(source, &edit),
                    r#"[dependencies]
serde.workspace = true
serde.features = ["derive"]"#
                );
            },
        );
    }

    #[test]
    fn generate_member_workspace_true_edit_returns_none_for_non_inline_table_without_version() {
        with_dependency_value(
            "[dependencies]\nserde.features = [\"derive\"]",
            "serde",
            |_, value, accessor_context| {
                let edit = generate_member_workspace_true_edit("serde", value, accessor_context);

                assert!(edit.is_none());
            },
        );
    }
}
