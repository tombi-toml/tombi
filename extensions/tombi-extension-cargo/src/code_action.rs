use tombi_document_tree::{TableKind, Value, dig_accessors, dig_keys};
use tombi_extension::CodeActionOrCommand;
use tombi_schema_store::{Accessor, AccessorContext, matches_accessors};
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    TextDocumentEdit, TextEdit, WorkspaceEdit,
};

use crate::{cargo_lock::load_cargo_lock, dependency_package_name, find_path_crate_cargo_toml, find_workspace_cargo_toml, get_workspace_path};

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
    line_index: &tombi_text::LineIndex,
    root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !cargo_code_action_root_enabled(features) {
        return Ok(None);
    }
    let Some(cargo_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut code_actions = Vec::new();

    if document_tree.contains_key("workspace") {
        code_actions.extend(code_actions_for_workspace_cargo_toml(
            text_document_uri,
            line_index,
            document_tree,
            accessors,
            features,
        ))
    } else {
        code_actions.extend(code_actions_for_crate_cargo_toml(
            text_document_uri,
            line_index,
            root,
            document_tree,
            &cargo_toml_path,
            accessors,
            contexts,
            toml_version,
            features,
        ));
    }

    Ok((!code_actions.is_empty()).then_some(code_actions))
}

fn cargo_code_action_root_enabled(features: Option<&tombi_config::CargoExtensionFeatures>) -> bool {
    features.map_or(
        true,
        tombi_config::CargoExtensionFeatures::code_action_enabled,
    )
}

fn code_actions_for_workspace_cargo_toml(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Vec<CodeActionOrCommand> {
    let mut code_actions = Vec::new();

    if features.map_or(
        true,
        tombi_config::CargoExtensionFeatures::convert_dependency_to_table_format_code_action_enabled,
    ) && let Some(action) =
        crate_version_code_action(text_document_uri, line_index, document_tree, accessors)
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    code_actions
}

fn code_actions_for_crate_cargo_toml(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    toml_version: tombi_config::TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Vec<CodeActionOrCommand> {
    let mut code_actions = Vec::new();

    if let Some((workspace_cargo_toml_path, workspace_root, workspace_document_tree)) =
        find_workspace_cargo_toml(
            cargo_toml_path,
            get_workspace_path(document_tree),
            toml_version,
        )
    {
        // Load workspace text and create line index for workspace document
        let Ok(workspace_text) = std::fs::read_to_string(&workspace_cargo_toml_path) else {
            return code_actions;
        };
        let workspace_line_index =
            tombi_text::LineIndex::new(&workspace_text, line_index.encoding_kind);

        // Add workspace-specific code actions here
        if features.map_or(
            true,
            tombi_config::CargoExtensionFeatures::inherit_from_workspace_code_action_enabled,
        ) && let Some(action) = workspace_code_action(
            text_document_uri,
            line_index,
            document_tree,
            accessors,
            contexts,
            &workspace_document_tree,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(action));
        }

        if features.map_or(
            true,
            tombi_config::CargoExtensionFeatures::add_to_workspace_and_inherit_dependency_code_action_enabled,
        ) && let Some(action) = add_workspace_dependency_code_action(
            text_document_uri,
            line_index,
            document_tree,
            accessors,
            contexts,
            &workspace_cargo_toml_path,
            &workspace_line_index,
            &workspace_root,
            &workspace_document_tree,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(action));
        }

        if features.map_or(
        true,
        tombi_config::CargoExtensionFeatures::inherit_dependency_from_workspace_code_action_enabled,
    ) && let Some(action) = use_workspace_dependency_code_action(
        text_document_uri,
        line_index,
        root,
        document_tree,
        cargo_toml_path,
        accessors,
            contexts,
            &workspace_cargo_toml_path,
            &workspace_document_tree,
            toml_version,
        ) {
            code_actions.push(CodeActionOrCommand::CodeAction(action));
        }
    }

    // Add crate-specific code actions here
    if features.map_or(
        true,
        tombi_config::CargoExtensionFeatures::convert_dependency_to_table_format_code_action_enabled,
    ) && let Some(action) =
        crate_version_code_action(text_document_uri, line_index, document_tree, accessors)
    {
        code_actions.push(CodeActionOrCommand::CodeAction(action));
    }

    code_actions
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
fn workspace_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    workspace_document_tree: &tombi_document_tree::DocumentTree,
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

    if let tombi_document_tree::Value::Table(table) = value
        && table.get("workspace").is_some()
    {
        return None; // Workspace already exists
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
                    range: (parent_key_context.range + value.symbol_range()).into_lsp(line_index),
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
    line_index: &tombi_text::LineIndex,
    root: &tombi_ast::Root,
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    workspace_cargo_toml_path: &std::path::Path,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    toml_version: tombi_config::TomlVersion,
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
    if dig_keys(
        workspace_document_tree,
        &["workspace", "dependencies", crate_name],
    )
    .is_none()
    {
        return None;
    }

    match value {
        tombi_document_tree::Value::String(version) => {
            let default_feature_additions = inherited_default_feature_additions(
                crate_document_tree,
                crate_cargo_toml_path,
                crate_name,
                None,
                workspace_cargo_toml_path,
                workspace_document_tree,
                toml_version,
            );

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
                            }
                            .into_lsp(line_index),
                            // NOTE: Convert to a workspace dependency to make it easier
                            //       to add other settings in the future.
                            new_text: render_inherited_dependency_string(crate_name, &default_feature_additions),
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

            let Some((key, version)) = table.get_key_value("version") else {
                return None; // No version to inherit
            };

            let default_feature_additions = inherited_default_feature_additions(
                crate_document_tree,
                crate_cargo_toml_path,
                crate_name,
                Some(table),
                workspace_cargo_toml_path,
                workspace_document_tree,
                toml_version,
            );

            let edits = if matches!(
                table.kind(),
                TableKind::KeyValue | TableKind::InlineTable { .. }
            ) {
                vec![OneOf::Left(TextEdit {
                    range: (crate_key_context.range + table.range()).into_lsp(line_index),
                    new_text: render_inherited_dependency_inline_table(
                        crate_name,
                        table,
                        &default_feature_additions,
                    ),
                })]
            } else {
                let mut edits = vec![OneOf::Left(TextEdit {
                    range: (key.range() + version.range()).into_lsp(line_index),
                    new_text: render_inherited_dependency_table_version(
                        table.get("features").and_then(|value| match value {
                            Value::Array(_) => Some(()),
                            _ => None,
                        }),
                        &default_feature_additions,
                    ),
                })];

                if !default_feature_additions.is_empty()
                    && let Some(Value::Array(features)) = table.get("features")
                {
                    edits.push(OneOf::Left(append_feature_array_edit(
                        line_index,
                        root,
                        features,
                        &default_feature_additions,
                        toml_version,
                    )?));
                }

                edits
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
                        edits,
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

fn inherited_default_feature_additions(
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    crate_name: &str,
    dependency_table: Option<&tombi_document_tree::Table>,
    workspace_cargo_toml_path: &std::path::Path,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    toml_version: tombi_config::TomlVersion,
) -> Vec<String> {
    if dependency_table.is_some_and(dependency_table_default_features_disabled)
    {
        return Vec::new();
    }

    let Some((_, workspace_dependency_value)) = dig_keys(
        workspace_document_tree,
        &["workspace", "dependencies", crate_name],
    ) else {
        return Vec::new();
    };
    let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
        return Vec::new();
    };

    if workspace_dependency_table
        .get("default-features")
        .and_then(|value| match value {
            Value::Boolean(boolean) => Some(boolean.value()),
            _ => None,
        })
        != Some(false)
    {
        return Vec::new();
    }

    let Some(resolved_dependency_version) = resolved_dependency_version(
        crate_document_tree,
        crate_cargo_toml_path,
        crate_name,
        workspace_dependency_value,
        workspace_document_tree,
        toml_version,
    ) else {
        return Vec::new();
    };

    let Some(default_features) = workspace_dependency_default_features(
        crate_name,
        workspace_dependency_table,
        workspace_cargo_toml_path,
        workspace_document_tree,
        toml_version,
        &resolved_dependency_version,
    ) else {
        return Vec::new();
    };

    let existing_features = dependency_table
        .and_then(|table| table.get("features"))
        .and_then(|value| match value {
            Value::Array(features) => Some(
                features
                    .values()
                    .iter()
                    .filter_map(|feature| match feature {
                        Value::String(feature) => Some(feature.value().to_string()),
                        _ => None,
                    })
                    .collect::<std::collections::BTreeSet<_>>(),
            ),
            _ => None,
        })
        .unwrap_or_default();

    default_features
        .into_iter()
        .filter(|feature| !existing_features.contains(feature.as_str()))
        .collect()
}

fn resolved_dependency_version(
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    crate_name: &str,
    workspace_dependency_value: &Value,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    toml_version: tombi_config::TomlVersion,
) -> Option<String> {
    let cargo_lock = load_cargo_lock(crate_cargo_toml_path, toml_version)?;
    let package_name = current_package_name(crate_document_tree)?;
    let package_version = package_version(
        crate_document_tree,
        crate_cargo_toml_path,
        Some(workspace_document_tree),
        toml_version,
    )?;
    let dependency_name = dependency_package_name(crate_name, workspace_dependency_value);

    cargo_lock.dependency_version_for_package(package_name, &package_version, dependency_name)
}

fn current_package_name(document_tree: &tombi_document_tree::DocumentTree) -> Option<&str> {
    let (_, Value::String(package_name)) = dig_keys(document_tree, &["package", "name"])? else {
        return None;
    };

    Some(package_name.value())
}

fn package_version(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    workspace_document_tree: Option<&tombi_document_tree::DocumentTree>,
    toml_version: tombi_config::TomlVersion,
) -> Option<String> {
    let (_, package_version) = dig_keys(document_tree, &["package", "version"])?;

    match package_version {
        Value::String(version) => Some(version.value().to_string()),
        Value::Table(table) => {
            let Some(Value::Boolean(workspace)) = table.get("workspace") else {
                return None;
            };
            if !workspace.value() {
                return None;
            }

            let workspace_document_tree = if let Some(workspace_document_tree) = workspace_document_tree
            {
                workspace_document_tree
            } else {
                let (_, _, workspace_document_tree) = find_workspace_cargo_toml(
                    cargo_toml_path,
                    get_workspace_path(document_tree),
                    toml_version,
                )?;
                return dig_keys(&workspace_document_tree, &["workspace", "package", "version"])
                    .and_then(|(_, value)| match value {
                        Value::String(version) => Some(version.value().to_string()),
                        _ => None,
                    });
            };
            let (_, Value::String(version)) = dig_keys(
                workspace_document_tree,
                &["workspace", "package", "version"],
            )?
            else {
                return None;
            };

            Some(version.value().to_string())
        }
        _ => None,
    }
}

fn workspace_dependency_default_features(
    crate_name: &str,
    workspace_dependency_table: &tombi_document_tree::Table,
    workspace_cargo_toml_path: &std::path::Path,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    toml_version: tombi_config::TomlVersion,
    resolved_dependency_version: &str,
) -> Option<Vec<String>> {
    let Some(Value::String(path)) = workspace_dependency_table.get("path") else {
        return None;
    };

    let (dependency_cargo_toml_path, _, dependency_document_tree) = find_path_crate_cargo_toml(
        workspace_cargo_toml_path,
        std::path::Path::new(path.value()),
        toml_version,
    )?;
    let dependency_value = Value::Table(workspace_dependency_table.clone());

    if current_package_name(&dependency_document_tree)?
        != dependency_package_name(crate_name, &dependency_value)
    {
        return None;
    }

    if package_version(
        &dependency_document_tree,
        &dependency_cargo_toml_path,
        Some(workspace_document_tree),
        toml_version,
    )? != resolved_dependency_version
    {
        return None;
    }

    let (_, Value::Array(default_features)) =
        dig_keys(&dependency_document_tree, &["features", "default"])?
    else {
        return None;
    };

    let default_features = default_features
        .values()
        .iter()
        .filter_map(|value| match value {
            Value::String(feature) => Some(feature.value().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    (!default_features.is_empty()).then_some(default_features)
}

fn render_inherited_dependency_inline_table(
    crate_name: &str,
    dependency_table: &tombi_document_tree::Table,
    default_feature_additions: &[String],
) -> String {
    let mut entries = vec!["workspace = true".to_string()];
    let mut has_features = false;

    for (key, value) in dependency_table.key_values() {
        match key.value.as_str() {
            "version" | "workspace" => {}
            "features" => {
                has_features = true;
                let Value::Array(features) = value else {
                    entries.push(format!("{} = {}", key.value, value));
                    continue;
                };
                entries.push(format!(
                    "{} = {}",
                    key.value,
                    render_feature_array(features, default_feature_additions)
                ));
            }
            _ => entries.push(format!("{} = {}", key.value, value)),
        }
    }

    if !has_features && !default_feature_additions.is_empty() {
        entries.push(format!(
            "features = {}",
            render_feature_values(default_feature_additions)
        ));
    }

    format!("{crate_name} = {{ {} }}", entries.join(", "))
}

fn render_inherited_dependency_string(
    crate_name: &str,
    default_feature_additions: &[String],
) -> String {
    if default_feature_additions.is_empty() {
        format!("{crate_name} = {{ workspace = true }}")
    } else {
        format!(
            "{crate_name} = {{ workspace = true, features = {} }}",
            render_feature_values(default_feature_additions)
        )
    }
}

fn render_inherited_dependency_table_version(
    existing_features: Option<()>,
    default_feature_additions: &[String],
) -> String {
    if existing_features.is_none() && !default_feature_additions.is_empty() {
        format!(
            "workspace = true\nfeatures = {}",
            render_feature_values(default_feature_additions)
        )
    } else {
        "workspace = true".to_string()
    }
}

fn render_feature_array(
    features: &tombi_document_tree::Array,
    default_feature_additions: &[String],
) -> String {
    let mut values = features
        .values()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();

    values.extend(
        default_feature_additions
            .iter()
            .map(|feature| format!("{feature:?}")),
    );

    format!("[{}]", values.join(", "))
}

fn render_feature_values(default_feature_additions: &[String]) -> String {
    format!(
        "[{}]",
        default_feature_additions
            .iter()
            .map(|feature| format!("{feature:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn append_feature_array_edit(
    line_index: &tombi_text::LineIndex,
    root: &tombi_ast::Root,
    features: &tombi_document_tree::Array,
    default_feature_additions: &[String],
    toml_version: tombi_config::TomlVersion,
) -> Option<TextEdit> {
    let ast_array = get_ast_array_from_range(root, features.range())?;
    let (insert_pos, new_text) =
        calculate_feature_array_append(&ast_array, default_feature_additions, toml_version)?;

    Some(TextEdit {
        range: tombi_text::Range::at(insert_pos).into_lsp(line_index),
        new_text,
    })
}

fn get_ast_array_from_range(
    root: &tombi_ast::Root,
    target_range: tombi_text::Range,
) -> Option<tombi_ast::Array> {
    use tombi_ast::AstNode;

    root.syntax().descendants().find_map(|node| {
        let array = tombi_ast::Array::cast(node)?;
        (array.range() == target_range).then_some(array)
    })
}

fn calculate_feature_array_append(
    ast_array: &tombi_ast::Array,
    default_feature_additions: &[String],
    toml_version: tombi_config::TomlVersion,
) -> Option<(tombi_text::Position, String)> {
    let values_with_comma = ast_array.values_with_comma().collect::<Vec<_>>();
    let rendered_additions = default_feature_additions
        .iter()
        .map(|feature| format!("{feature:?}"))
        .collect::<Vec<_>>();

    if values_with_comma.is_empty() {
        return Some((
            ast_array.bracket_start()?.range().end,
            rendered_additions.join(", "),
        ));
    }

    let (last_value, last_comma) = values_with_comma.last()?;
    let insert_pos = last_comma
        .as_ref()
        .map_or_else(|| last_value.range().end, |comma| comma.range().end);

    if ast_array.should_be_multiline(toml_version) {
        let indent = " ".repeat(last_value.range().start.column as usize);
        let additions = rendered_additions
            .iter()
            .map(|feature| format!("\n{indent}{feature},"))
            .collect::<String>();
        let prefix = if last_comma.is_some() { "" } else { "," };
        return Some((insert_pos, format!("{prefix}{additions}")));
    }

    let prefix = if last_comma.is_some() { " " } else { ", " };
    Some((insert_pos, format!("{prefix}{}", rendered_additions.join(", "))))
}

fn dependency_table_default_features_disabled(table: &tombi_document_tree::Table) -> bool {
    table.get("default-features").is_some_and(|value| match value {
        Value::Boolean(boolean) => !boolean.value(),
        _ => false,
    })
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
fn crate_version_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
) -> Option<CodeAction> {
    if (matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
        || matches_accessors!(accessors, ["workspace", "dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _]))
        && let Some((_, tombi_document_tree::Value::String(version))) =
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
                            range: tombi_text::Range::at(version.symbol_range().start)
                                .into_lsp(line_index),
                            new_text: "{ version = ".to_string(),
                        }),
                        OneOf::Left(TextEdit {
                            range: tombi_text::Range::at(version.symbol_range().end)
                                .into_lsp(line_index),
                            new_text: " }".to_string(),
                        }),
                    ],
                }])),
                change_annotations: None,
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

/// Get AST InlineTable from document tree range
/// First finds the range in document_tree, then locates the corresponding AST node
fn get_ast_inline_table_from_document_tree(
    root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    keys: &[&str],
) -> Option<tombi_ast::InlineTable> {
    use tombi_ast::AstNode;

    // Get the value from document tree to find its range
    let (_, value) = tombi_document_tree::dig_keys(document_tree, keys)?;

    let tombi_document_tree::Value::Table(doc_table) = value else {
        return None;
    };

    // Get the range of the inline table in the document tree
    let target_range = doc_table.range();

    // Use descendants to find the InlineTable with matching range
    for node in root.syntax().descendants() {
        if let Some(inline_table) = tombi_ast::InlineTable::cast(node)
            && inline_table.range() == target_range
        {
            return Some(inline_table);
        }
    }

    None
}

/// Calculate insertion position and text for inline table insertion with comma handling
/// Uses tombi_ast API to properly handle commas and formatting
fn calculate_inline_table_insertion(
    ast_inline_table: &tombi_ast::InlineTable,
    insertion_index: usize,
    new_entry_text: &str,
) -> Option<(tombi_text::Position, String)> {
    use tombi_ast::AstNode;

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
                dangling_comment.syntax().range().end,
                format!("\n\n{},\n", new_entry_text),
            ))
        } else {
            Some((
                ast_inline_table.brace_start()?.range().end,
                new_entry_text.to_string(),
            ))
        };
    }

    if insertion_index == 0 {
        // Insert at the beginning
        // { tokio = "1.0" } -> { serde = "1.0", tokio = "1.0" }
        let (first_key_value, _) = key_values_with_comma.first()?;
        let insert_pos = first_key_value.syntax().range().start;
        let new_text = format!("{},\n", new_entry_text);
        return Some((insert_pos, new_text));
    }

    if insertion_index >= key_values_with_comma.len() {
        // Insert at the end
        // { serde = "1.0" } -> { serde = "1.0", tokio = "1.0" }
        let (last_key_value, last_comma) = key_values_with_comma.last()?;
        if let Some(last_comma) = last_comma {
            let insert_pos = last_comma.range().end;
            let new_text = format!("\n{}, ", new_entry_text);
            return Some((insert_pos, new_text));
        } else {
            let insert_pos = last_key_value.syntax().range().end;
            let new_text = format!(", {}", new_entry_text);
            return Some((insert_pos, new_text));
        }
    }

    // Insert in the middle
    // { serde = "1.0", tracing = "0.1" } -> { serde = "1.0", tokio = "1.0", tracing = "0.1" }
    let (target_key_value, target_comma) = key_values_with_comma.get(insertion_index)?;
    let insert_pos = if let Some(target_comma) = target_comma {
        target_comma.range().end
    } else {
        target_key_value.syntax().range().end
    };
    let new_text = format!("\n{},\n", new_entry_text);
    Some((insert_pos, new_text))
}

/// Add a dependency to workspace.dependencies and convert member's dependency
/// to workspace = true format.
///
/// This code action is provided when:
/// - The cursor is on a dependency in member Cargo.toml
/// - The dependency is not yet registered in workspace.dependencies
/// - The dependency is not already using workspace = true
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
/// serde = { workspace = true }
/// ```
///
fn add_workspace_dependency_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
    workspace_cargo_toml_path: &std::path::Path,
    workspace_line_index: &tombi_text::LineIndex,
    workspace_root: &tombi_ast::Root,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
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
    let workspace_edit = generate_workspace_dependencies_edit(
        workspace_line_index,
        workspace_root,
        workspace_document_tree,
        crate_name,
        crate_value,
    )?;

    // Generate member edit to convert to workspace = true
    let member_edit = generate_member_workspace_true_edit(
        line_index,
        crate_name,
        crate_value,
        &contexts[1 + offset],
    )?;

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
    workspace_line_index: &tombi_text::LineIndex,
    workspace_root: &tombi_ast::Root,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    crate_name: &str,
    crate_value: &tombi_document_tree::Value,
) -> Option<TextEdit> {
    // Get or prepare workspace.dependencies section
    let workspace_deps = dig_keys(workspace_document_tree, &["workspace", "dependencies"]);

    let Some((_, deps_table)) = workspace_deps else {
        // NOTE: `workspace.dependencies` section doesn't exist, need to create it
        //       For now, return None - this will be handled in a future enhancement
        return None;
    };

    let tombi_document_tree::Value::Table(table) = deps_table else {
        return None;
    };

    // Get existing crate names and calculate insertion index
    let existing_crates: Vec<&str> = table.keys().map(|key| key.value.as_str()).collect();
    let insertion_index = calculate_insertion_index(&existing_crates, crate_name);
    let crate_value_text = crate_value.to_string();

    // Find insertion position in the actual table
    let (insertion_range, new_text) = if table.kind() == TableKind::Table {
        let insertion_range = if insertion_index == 0 {
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
            if let Some((target_key, _)) = table.get_key_value(existing_crates[insertion_index]) {
                let range = target_key.range();
                tombi_text::Range::at(range.start)
            } else {
                let range = table.range();
                tombi_text::Range::at(range.end)
            }
        };

        (
            insertion_range,
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
        let (insertion_pos, new_text) =
            calculate_inline_table_insertion(&ast_inline_table, insertion_index, &new_entry_text)?;

        (tombi_text::Range::at(insertion_pos), new_text)
    } else {
        return None;
    };

    Some(TextEdit {
        range: insertion_range.into_lsp(workspace_line_index),
        new_text,
    })
}

/// Generate TextEdit for converting member dependency to workspace = true
fn generate_member_workspace_true_edit(
    line_index: &tombi_text::LineIndex,
    crate_name: &str,
    crate_value: &tombi_document_tree::Value,
    accessor_context: &AccessorContext,
) -> Option<TextEdit> {
    let AccessorContext::Key(crate_key_context) = accessor_context else {
        return None;
    };

    Some(TextEdit {
        range: (crate_key_context.range + crate_value.range()).into_lsp(line_index),
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
