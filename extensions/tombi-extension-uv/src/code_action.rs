use pep508_rs::{Requirement, VerbatimUrl};
use std::str::FromStr;
use tombi_ast::AstNode;
use tombi_extension::CodeActionOrCommand;
use tombi_schema_store::Accessor;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    TextDocumentEdit, TextEdit, WorkspaceEdit,
};

use crate::find_workspace_pyproject_toml;

fn parse_dependency(dep_string: &str) -> Option<Requirement<VerbatimUrl>> {
    match Requirement::<VerbatimUrl>::from_str(dep_string) {
        Ok(requirement) => Some(requirement),
        Err(e) => {
            tracing::debug!(
                dependency = %dep_string,
                error = %e,
                "Failed to parse PEP 508 dependency string"
            );
            None
        }
    }
}

fn format_dependency_without_version(requirement: &Requirement<VerbatimUrl>) -> String {
    let name = requirement.name.to_string();
    if requirement.extras.is_empty() {
        name
    } else {
        let extras: Vec<String> = requirement.extras.iter().map(|e| e.to_string()).collect();
        format!("{}[{}]", name, extras.join(","))
    }
}

pub enum CodeActionRefactorRewriteName {
    /// Use Workspace Dependency
    ///
    /// Convert a member's dependency to use the version defined in the workspace.
    ///
    /// Before:
    /// ```toml
    /// # In member's pyproject.toml
    /// [project]
    /// dependencies = ["pydantic>=2.10,<3.0"]
    /// ```
    ///
    /// After applying "Use Workspace Dependency":
    /// ```toml
    /// # In member's pyproject.toml
    /// [project]
    /// dependencies = ["pydantic"]
    /// ```
    UseWorkspaceDependency,

    /// Add to Workspace and Use Workspace Dependency
    ///
    /// Add a dependency to workspace's [project.dependencies] and convert the member's
    /// dependency to version-less format.
    ///
    /// Before:
    /// ```toml
    /// # Workspace pyproject.toml
    /// [project]
    /// dependencies = []
    ///
    /// # Member pyproject.toml
    /// [project]
    /// dependencies = ["pydantic>=2.10,<3.0"]
    /// ```
    ///
    /// After applying "Add to Workspace and Use Workspace Dependency":
    /// ```toml
    /// # Workspace pyproject.toml
    /// [project]
    /// dependencies = ["pydantic>=2.10,<3.0"]
    ///
    /// # Member pyproject.toml
    /// [project]
    /// dependencies = ["pydantic"]
    /// ```
    AddToWorkspaceAndUseWorkspaceDependency,
}

impl std::fmt::Display for CodeActionRefactorRewriteName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CodeActionRefactorRewriteName::UseWorkspaceDependency => {
                write!(f, "Use Workspace Dependency")
            }
            CodeActionRefactorRewriteName::AddToWorkspaceAndUseWorkspaceDependency => {
                write!(f, "Add to Workspace and Use Workspace Dependency")
            }
        }
    }
}

fn calculate_insertion_index(existing_package_names: &[&str], new_package_name: &str) -> usize {
    existing_package_names
        .iter()
        .position(|&existing| {
            tombi_version_sort::version_sort(new_package_name, existing) == std::cmp::Ordering::Less
        })
        .unwrap_or(existing_package_names.len())
}

/// Get AST array from document tree range
/// First finds the range in document_tree, then locates the corresponding AST node
fn get_ast_array_from_document_tree(
    root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    keys: &[&str],
) -> Option<tombi_ast::Array> {
    // Get the value from document tree to find its range
    let (_, value) = tombi_document_tree::dig_keys(document_tree, keys)?;

    let tombi_document_tree::Value::Array(doc_array) = value else {
        return None;
    };

    // Get the range of the array in the document tree
    let target_range = doc_array.range();

    // Use descendants to find the Array with matching range
    for node in root.syntax().descendants() {
        if let Some(array) = tombi_ast::Array::cast(node) {
            if array.range() == target_range {
                return Some(array);
            }
        }
    }

    None
}

/// Calculate insertion position and text for array insertion with comma handling
/// Uses tombi_ast API to properly handle commas and formatting
fn calculate_array_insertion(
    ast_array: &tombi_ast::Array,
    insertion_index: usize,
    new_element: &tombi_document_tree::String,
) -> Option<(tombi_text::Position, String)> {
    let values_with_comma: Vec<_> = ast_array.values_with_comma().collect();

    if values_with_comma.is_empty() {
        // Empty array - insert without comma
        return if let Some(dangling_comment) = ast_array
            .inner_dangling_comments()
            .last()
            .and_then(|comments| comments.last())
        {
            Some((
                dangling_comment.syntax().range().end,
                format!("\n\n{},\n", new_element),
            ))
        } else {
            Some((
                ast_array.bracket_start()?.range().end,
                new_element.to_string(),
            ))
        };
    }

    if insertion_index == 0 {
        // Insert at the beginning
        let (first_value, _) = values_with_comma.first()?;
        let insert_pos = first_value.syntax().range().start;
        let new_text = format!("{},\n", new_element.to_string());
        return Some((insert_pos, new_text));
    }

    if insertion_index >= values_with_comma.len() {
        // Insert at the end
        let (last_value, last_comma) = values_with_comma.last()?;
        if let Some(last_comma) = last_comma {
            let insert_pos = last_comma.range().end;
            let new_text = format!("\n{},", new_element.to_string());
            return Some((insert_pos, new_text));
        } else {
            let insert_pos = last_value.syntax().range().end;
            let new_text = format!(",{}", new_element.to_string());
            return Some((insert_pos, new_text));
        }
    }

    // Insert in the middle
    let (target_value, target_comma) = values_with_comma.get(insertion_index)?;
    let insert_pos = if let Some(target_comma) = target_comma {
        target_comma.range().end
    } else {
        target_value.syntax().range().end
    };
    let new_text = format!("\n{},\n", new_element.to_string());
    return Some((insert_pos, new_text));
}

fn add_workspace_dependency_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    workspace_pyproject_toml_path: &std::path::Path,
    workspace_line_index: &tombi_text::LineIndex,
    workspace_root: &tombi_ast::Root,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
) -> Option<CodeAction> {
    // Accessors should be at least: ["project", "dependencies", Index(n)]
    if accessors.len() < 3 {
        return None;
    }

    // Get the dependency string from member's document tree
    let (_, dependency_value) = tombi_document_tree::dig_accessors(document_tree, accessors)?;

    let tombi_document_tree::Value::String(dependency) = dependency_value else {
        return None;
    };

    // Parse the dependency
    let requirement = parse_dependency(dependency.value())?;

    // If no version specified, don't provide code action
    if requirement.version_or_url.is_none() {
        return None;
    }

    // Check if this dependency already exists in workspace
    let (_, workspace_deps) =
        tombi_document_tree::dig_keys(workspace_document_tree, &["project", "dependencies"])?;

    let tombi_document_tree::Value::Array(workspace_deps_array) = workspace_deps else {
        return None;
    };

    // Check if the workspace already has a dependency with the same package name
    let workspace_has_dep = workspace_deps_array.iter().any(|dep| {
        if let tombi_document_tree::Value::String(ws_dep_str) = dep {
            if let Some(ws_requirement) = parse_dependency(ws_dep_str.value()) {
                return ws_requirement.name == requirement.name;
            }
        }
        false
    });

    if workspace_has_dep {
        return None; // Already in workspace
    }

    // Generate workspace URI
    let Ok(workspace_uri) = tombi_uri::Uri::from_file_path(workspace_pyproject_toml_path) else {
        tracing::warn!(
            path = %workspace_pyproject_toml_path.display(),
            "Failed to convert workspace path to URI"
        );
        return None;
    };

    // Generate workspace edit (add dependency with version, without extras)
    let workspace_edit = generate_workspace_dependency_edit(
        workspace_line_index,
        workspace_root,
        workspace_document_tree,
        &requirement,
        dependency,
    )?;

    // Generate member edit (convert to version-less format, preserving extras)
    let member_edit = generate_member_dependency_edit(dependency, &requirement, line_index)?;

    // Build WorkspaceEdit with both file changes
    let workspace_edit_full = WorkspaceEdit {
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
        title: CodeActionRefactorRewriteName::AddToWorkspaceAndUseWorkspaceDependency.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE.clone()),
        diagnostics: None,
        edit: Some(workspace_edit_full),
        ..Default::default()
    })
}

/// Generate TextEdit for adding dependency to workspace [project.dependencies]
fn generate_workspace_dependency_edit(
    workspace_line_index: &tombi_text::LineIndex,
    workspace_root: &tombi_ast::Root,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    requirement: &Requirement<VerbatimUrl>,
    dependency: &tombi_document_tree::String,
) -> Option<TextEdit> {
    // Get workspace.dependencies array from document tree
    let (_, workspace_deps) =
        tombi_document_tree::dig_keys(workspace_document_tree, &["project", "dependencies"])?;

    let tombi_document_tree::Value::Array(deps_doc_array) = workspace_deps else {
        return None;
    };

    // Get the AST array to access comma information
    let deps_ast_array = get_ast_array_from_document_tree(
        workspace_root,
        workspace_document_tree,
        &["project", "dependencies"],
    )?;

    // Get existing package names
    let existing_package_names: Vec<String> = deps_doc_array
        .iter()
        .filter_map(|dep| {
            if let tombi_document_tree::Value::String(dep_str) = dep {
                parse_dependency(dep_str.value()).map(|req| req.name.to_string())
            } else {
                None
            }
        })
        .collect();

    // Convert to &str for calculate_insertion_index
    let existing_packages: Vec<&str> = existing_package_names.iter().map(|s| s.as_str()).collect();

    let package_name = requirement.name.to_string();

    // Calculate insertion index
    let insertion_index = calculate_insertion_index(&existing_packages, &package_name);

    // Determine insertion position and comma handling using AST
    let (insertion_range, new_text) =
        calculate_array_insertion(&deps_ast_array, insertion_index, dependency)?;

    Some(TextEdit {
        range: tombi_text::Range::at(insertion_range).into_lsp(workspace_line_index),
        new_text,
    })
}

/// Generate TextEdit for converting member dependency to version-less format
fn generate_member_dependency_edit(
    dependency: &tombi_document_tree::String,
    requirement: &Requirement<VerbatimUrl>,
    line_index: &tombi_text::LineIndex,
) -> Option<TextEdit> {
    let new_dep_str = format_dependency_without_version(requirement);

    Some(TextEdit {
        range: dependency.range().into_lsp(line_index),
        new_text: format!("\"{}\"", new_dep_str),
    })
}

fn use_workspace_dependency_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    workspace_document_tree: &tombi_document_tree::DocumentTree,
) -> Option<CodeAction> {
    // Accessors should be at least: ["project", "dependencies", Index(n)]
    if accessors.len() < 3 {
        return None;
    }

    // Get the dependency string from member's document tree
    let (_, dep_value) = tombi_document_tree::dig_accessors(document_tree, accessors)?;

    let tombi_document_tree::Value::String(dep_string) = dep_value else {
        return None;
    };

    // Parse the dependency
    let requirement = parse_dependency(dep_string.value())?;

    // If no version specified, don't provide code action
    if requirement.version_or_url.is_none() {
        return None;
    }

    // Check if this dependency exists in workspace
    let (_, workspace_deps) =
        tombi_document_tree::dig_keys(workspace_document_tree, &["project", "dependencies"])?;

    let tombi_document_tree::Value::Array(workspace_deps_array) = workspace_deps else {
        return None;
    };

    // Check if the workspace has a dependency with the same package name
    let workspace_has_dep = workspace_deps_array.iter().any(|dep| {
        if let tombi_document_tree::Value::String(ws_dep_str) = dep {
            if let Some(ws_requirement) = parse_dependency(ws_dep_str.value()) {
                return ws_requirement.name == requirement.name;
            }
        }
        false
    });

    if !workspace_has_dep {
        return None;
    }

    // Format dependency without version (preserving extras)
    let new_dep_str = format_dependency_without_version(&requirement);

    // Use the string's range for replacement
    let range = dep_string.range().into_lsp(line_index);

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::UseWorkspaceDependency.to_string(),
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
                    range,
                    new_text: format!("\"{}\"", new_dep_str),
                })],
            }])),
            change_annotations: None,
        }),
        ..Default::default()
    })
}

pub fn code_action(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    toml_version: tombi_config::TomlVersion,
    line_index: &tombi_text::LineIndex,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    // Check if the file is pyproject.toml
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    // Check if this is a workspace root (has [tool.uv.workspace] section)
    if document_tree.contains_key("tool") {
        if let Some((_, tool_value)) = tombi_document_tree::dig_keys(document_tree, &["tool"]) {
            if let tombi_document_tree::Value::Table(tool_table) = tool_value {
                if let Some((_, uv_value)) = tool_table.get_key_value("uv") {
                    if let tombi_document_tree::Value::Table(uv_table) = uv_value {
                        if uv_table.contains_key("workspace") {
                            // This is a workspace root, don't provide code actions
                            return Ok(None);
                        }
                    }
                }
            }
        }
    }

    // Check if accessors match [project.dependencies] or [project.optional-dependencies.*]
    if accessors.len() < 3 {
        return Ok(None);
    }

    let is_project_dependencies = matches!(accessors.first(), Some(Accessor::Key(key)) if key == "project")
        && matches!(accessors.get(1), Some(Accessor::Key(key)) if key == "dependencies");

    let is_optional_dependencies = matches!(accessors.first(), Some(Accessor::Key(key)) if key == "project")
        && matches!(accessors.get(1), Some(Accessor::Key(key)) if key == "optional-dependencies");

    if !is_project_dependencies && !is_optional_dependencies {
        return Ok(None);
    }

    // Try to find workspace pyproject.toml
    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        tracing::warn!(
            uri = %text_document_uri,
            "Failed to convert URI to file path"
        );
        return Ok(None);
    };

    let Some((workspace_path, workspace_root, workspace_document_tree)) =
        find_workspace_pyproject_toml(&pyproject_toml_path, toml_version)
    else {
        tracing::debug!(
            member_path = %pyproject_toml_path.display(),
            "No workspace pyproject.toml found"
        );
        return Ok(None);
    };

    // Load workspace text and create line index for workspace document
    let Ok(workspace_text) = std::fs::read_to_string(&workspace_path) else {
        tracing::warn!(
            path = %workspace_path.display(),
            "Failed to read workspace pyproject.toml"
        );
        return Ok(None);
    };
    let workspace_line_index =
        tombi_text::LineIndex::new(&workspace_text, line_index.encoding_kind);

    // Try to provide code actions
    let mut actions = Vec::new();

    // Try "Use Workspace Dependency" (when dependency exists in workspace)
    if let Some(action) = use_workspace_dependency_code_action(
        text_document_uri,
        line_index,
        document_tree,
        accessors,
        &workspace_document_tree,
    ) {
        tracing::debug!(
            action = %action.title,
            uri = %text_document_uri,
            "Providing 'Use Workspace Dependency' code action"
        );
        actions.push(CodeActionOrCommand::CodeAction(action));
    }

    // Try "Add to Workspace and Use Workspace Dependency" (when dependency doesn't exist in workspace)
    if let Some(action) = add_workspace_dependency_code_action(
        text_document_uri,
        line_index,
        document_tree,
        accessors,
        &workspace_path,
        &workspace_line_index,
        &workspace_root,
        &workspace_document_tree,
    ) {
        tracing::debug!(
            action = %action.title,
            uri = %text_document_uri,
            "Providing 'Add to Workspace and Use Workspace Dependency' code action"
        );
        actions.push(CodeActionOrCommand::CodeAction(action));
    }

    if actions.is_empty() {
        tracing::trace!(
            uri = %text_document_uri,
            "No code actions available for this context"
        );
        Ok(None)
    } else {
        Ok(Some(actions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_ast::AstNode;
    use tombi_document_tree::TryIntoDocumentTree;

    #[test]
    fn test_code_action_refactor_rewrite_name_display() {
        assert_eq!(
            CodeActionRefactorRewriteName::UseWorkspaceDependency.to_string(),
            "Use Workspace Dependency"
        );
        assert_eq!(
            CodeActionRefactorRewriteName::AddToWorkspaceAndUseWorkspaceDependency.to_string(),
            "Add to Workspace and Use Workspace Dependency"
        );
    }

    #[test]
    fn test_parse_dependency_basic_with_version() {
        let requirement = parse_dependency("pydantic>=2.10,<3.0").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        assert!(requirement.extras.is_empty());
        assert!(requirement.version_or_url.is_some());
    }

    #[test]
    fn test_parse_dependency_with_extras() {
        let requirement = parse_dependency("pydantic[email,dotenv]>=2.10").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        let extras: Vec<String> = requirement.extras.iter().map(|e| e.to_string()).collect();
        assert_eq!(extras, vec!["email", "dotenv"]);
        assert!(requirement.version_or_url.is_some());
    }

    #[test]
    fn test_parse_dependency_without_version() {
        let requirement = parse_dependency("pydantic").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        assert!(requirement.extras.is_empty());
        assert!(requirement.version_or_url.is_none());
    }

    #[test]
    fn test_parse_dependency_with_extras_no_version() {
        let requirement = parse_dependency("pydantic[email]").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        let extras: Vec<String> = requirement.extras.iter().map(|e| e.to_string()).collect();
        assert_eq!(extras, vec!["email"]);
        assert!(requirement.version_or_url.is_none());
    }

    #[test]
    fn test_parse_dependency_invalid() {
        let result = parse_dependency("invalid package name!!!");
        assert!(result.is_none());
    }

    #[test]
    fn test_format_dependency_without_extras() {
        let requirement = parse_dependency("pydantic>=2.10").unwrap();
        let formatted = format_dependency_without_version(&requirement);
        assert_eq!(formatted, "pydantic");
    }

    #[test]
    fn test_format_dependency_with_one_extra() {
        let requirement = parse_dependency("pydantic[email]>=2.10").unwrap();
        let formatted = format_dependency_without_version(&requirement);
        assert_eq!(formatted, "pydantic[email]");
    }

    #[test]
    fn test_format_dependency_with_multiple_extras() {
        let requirement = parse_dependency("pydantic[email,dotenv]>=2.10").unwrap();
        let formatted = format_dependency_without_version(&requirement);
        assert_eq!(formatted, "pydantic[email,dotenv]");
    }

    #[test]
    fn test_code_action_non_pyproject_toml_returns_none() {
        let uri = tombi_uri::Uri::from_file_path("/path/to/Cargo.toml").unwrap();
        let toml_text = r#"
[package]
name = "test"
"#;
        let root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = code_action(
            &uri,
            &document_tree,
            &[],
            tombi_config::TomlVersion::default(),
            &line_index,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_code_action_workspace_root_returns_none() {
        let uri = tombi_uri::Uri::from_file_path("/path/to/pyproject.toml").unwrap();
        let toml_text = r#"
[tool.uv.workspace]
members = ["member1"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = code_action(
            &uri,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
            ],
            tombi_config::TomlVersion::default(),
            &line_index,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_code_action_invalid_accessor_returns_none() {
        let uri = tombi_uri::Uri::from_file_path("/path/to/pyproject.toml").unwrap();
        let toml_text = r#"
[project]
name = "test"
"#;
        let root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        // Test with invalid accessor (not dependencies)
        let result = code_action(
            &uri,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("name".to_string()),
            ],
            tombi_config::TomlVersion::default(),
            &line_index,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Tests for use_workspace_dependency_code_action

    #[test]
    fn test_use_workspace_dependency_basic_case() {
        // Member pyproject.toml with versioned dependency
        let member_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let member_toml = r#"
[project]
name = "member"
dependencies = ["pydantic>=2.10,<3.0"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(member_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        // Workspace pyproject.toml with the same dependency
        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10,<3.0"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_tree = workspace_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index =
            tombi_text::LineIndex::new(member_toml, tombi_text::EncodingKind::default());

        // Call use_workspace_dependency_code_action
        let result = use_workspace_dependency_code_action(
            &member_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );

        // Should return a code action to convert "pydantic>=2.10,<3.0" to "pydantic"
        assert!(result.is_some());
        let action = result.unwrap();
        assert_eq!(action.title, "Use Workspace Dependency");
    }

    #[test]
    fn test_use_workspace_dependency_with_extras() {
        let text_document_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let toml_text = r#"
[project]
name = "member"
dependencies = ["pydantic[email,dotenv]>=2.10,<3.0"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_tree = workspace_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = use_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );

        // Should convert to "pydantic[email,dotenv]" (preserving extras)
        assert!(result.is_some());
    }

    #[test]
    fn test_use_workspace_dependency_already_versionless() {
        let member_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let toml_text = r#"
[project]
name = "member"
dependencies = ["pydantic"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_tree = workspace_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = use_workspace_dependency_code_action(
            &member_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );

        // Already version-less, should not provide code action
        assert!(result.is_none());
    }

    #[test]
    fn test_use_workspace_dependency_not_in_workspace() {
        let text_document_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let toml_text = r#"
[project]
name = "member"
dependencies = ["requests>=2.28"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_tree = workspace_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = use_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );

        // Dependency not in workspace, should not provide code action
        assert!(result.is_none());
    }

    // Tests for calculate_insertion_index

    #[test]
    fn test_calculate_insertion_index_empty_list() {
        let existing: Vec<&str> = vec![];
        let result = calculate_insertion_index(&existing, "pydantic");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_insertion_index_insert_at_beginning() {
        let existing = vec!["requests", "urllib3"];
        let result = calculate_insertion_index(&existing, "pydantic");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_insertion_index_insert_at_end() {
        let existing = vec!["pydantic", "requests"];
        let result = calculate_insertion_index(&existing, "urllib3");
        assert_eq!(result, 2);
    }

    #[test]
    fn test_calculate_insertion_index_insert_in_middle() {
        let existing = vec!["pydantic", "urllib3"];
        let result = calculate_insertion_index(&existing, "requests");
        assert_eq!(result, 1);
    }

    // Tests for add_workspace_dependency_code_action

    #[test]
    fn test_add_workspace_dependency_basic_case() {
        let member_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let workspace_uri = tombi_uri::Uri::from_file_path("/workspace/pyproject.toml").unwrap();

        let member_toml = r#"
[project]
name = "member"
dependencies = ["requests>=2.28"]
"#;
        let member_root = tombi_ast::Root::cast(
            tombi_parser::parse(member_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let member_tree = member_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_root_for_tree =
            tombi_ast::Root::cast(workspace_root.syntax().clone()).unwrap();
        let workspace_tree = workspace_root_for_tree
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let member_line_index =
            tombi_text::LineIndex::new(member_toml, tombi_text::EncodingKind::default());
        let workspace_line_index =
            tombi_text::LineIndex::new(workspace_toml, tombi_text::EncodingKind::default());

        let result = add_workspace_dependency_code_action(
            &member_uri,
            &member_line_index,
            &member_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            workspace_uri.to_file_path().unwrap().as_path(),
            &workspace_line_index,
            &workspace_root,
            &workspace_tree,
        );

        // Should return a code action to add "requests>=2.28" to workspace and convert member to "requests"
        assert!(result.is_some());
        let action = result.unwrap();
        assert_eq!(
            action.title,
            "Add to Workspace and Use Workspace Dependency"
        );
    }

    #[test]
    fn test_add_workspace_dependency_already_in_workspace() {
        let member_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let workspace_uri = tombi_uri::Uri::from_file_path("/workspace/pyproject.toml").unwrap();

        let member_toml = r#"
[project]
name = "member"
dependencies = ["pydantic>=2.10"]
"#;
        let member_root = tombi_ast::Root::cast(
            tombi_parser::parse(member_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let member_tree = member_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10,<3.0"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_root_for_tree =
            tombi_ast::Root::cast(workspace_root.syntax().clone()).unwrap();
        let workspace_tree = workspace_root_for_tree
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let member_line_index =
            tombi_text::LineIndex::new(member_toml, tombi_text::EncodingKind::default());
        let workspace_line_index =
            tombi_text::LineIndex::new(workspace_toml, tombi_text::EncodingKind::default());

        let result = add_workspace_dependency_code_action(
            &member_uri,
            &member_line_index,
            &member_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            workspace_uri.to_file_path().unwrap().as_path(),
            &workspace_line_index,
            &workspace_root,
            &workspace_tree,
        );

        // Already in workspace, should not provide code action
        assert!(result.is_none());
    }

    // Tests for exclusive provision (Task 7.3)

    #[test]
    fn test_exclusive_provision_use_when_in_workspace() {
        // When dependency exists in workspace, only "Use Workspace Dependency" should be provided
        let text_document_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let workspace_uri = tombi_uri::Uri::from_file_path("/workspace/pyproject.toml").unwrap();

        let toml_text = r#"
[project]
name = "member"
dependencies = ["pydantic>=2.10"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10,<3.0"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_root_for_tree =
            tombi_ast::Root::cast(workspace_root.syntax().clone()).unwrap();
        let workspace_tree = workspace_root_for_tree
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());
        let workspace_line_index =
            tombi_text::LineIndex::new(workspace_toml, tombi_text::EncodingKind::default());

        // "Use Workspace Dependency" should be provided
        let use_result = use_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );
        assert!(use_result.is_some());

        // "Add to Workspace" should NOT be provided
        let add_result = add_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            workspace_uri.to_file_path().unwrap().as_path(),
            &workspace_line_index,
            &workspace_root,
            &workspace_tree,
        );
        assert!(add_result.is_none());
    }

    #[test]
    fn test_exclusive_provision_add_when_not_in_workspace() {
        // When dependency doesn't exist in workspace, only "Add to Workspace" should be provided
        let text_document_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let workspace_uri = tombi_uri::Uri::from_file_path("/workspace/pyproject.toml").unwrap();

        let toml_text = r#"
[project]
name = "member"
dependencies = ["requests>=2.28"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_root_for_tree =
            tombi_ast::Root::cast(workspace_root.syntax().clone()).unwrap();
        let workspace_tree = workspace_root_for_tree
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());
        let workspace_line_index =
            tombi_text::LineIndex::new(workspace_toml, tombi_text::EncodingKind::default());

        // "Use Workspace Dependency" should NOT be provided
        let use_result = use_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );
        assert!(use_result.is_none());

        // "Add to Workspace" should be provided
        let add_result = add_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            workspace_uri.to_file_path().unwrap().as_path(),
            &workspace_line_index,
            &workspace_root,
            &workspace_tree,
        );
        assert!(add_result.is_some());
    }

    #[test]
    fn test_optional_dependencies_support() {
        // Test that code actions work for [project.optional-dependencies] too
        let text_document_uri =
            tombi_uri::Uri::from_file_path("/workspace/member/pyproject.toml").unwrap();
        let toml_text = r#"
[project]
name = "member"

[project.optional-dependencies]
dev = ["pytest>=7.0"]
"#;
        let document_root = tombi_ast::Root::cast(
            tombi_parser::parse(toml_text, tombi_config::TomlVersion::default()).into_syntax_node(),
        )
        .unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pytest>=7.0,<8.0"]
"#;
        let workspace_root = tombi_ast::Root::cast(
            tombi_parser::parse(workspace_toml, tombi_config::TomlVersion::default())
                .into_syntax_node(),
        )
        .unwrap();
        let workspace_tree = workspace_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = use_workspace_dependency_code_action(
            &text_document_uri,
            &line_index,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("optional-dependencies".to_string()),
                Accessor::Key("dev".to_string()),
                Accessor::Index(0),
            ],
            &workspace_tree,
        );

        // Should work for optional-dependencies too
        assert!(result.is_some());
    }
}
