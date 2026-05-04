use pep508_rs::{
    Requirement, VerbatimUrl, VersionOrUrl,
    pep440_rs::{Version, VersionSpecifier},
};
use tombi_ast::AstNode;
use tombi_document_tree::dig_keys;
use tombi_extension::CodeActionOrCommand;
use tombi_schema_store::Accessor;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionDisabled, CodeActionKind, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit, WorkspaceEdit,
};

use crate::{
    DependencyRequirement, collect_dependency_requirements_from_document_tree, fetch_pypi_project,
    find_workspace_pyproject_toml, get_dependency_accessors, parse_dependency_requirement,
    parse_requirement,
};

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

    /// Update Dependency to Latest Version
    ///
    /// Before:
    /// ```toml
    /// [project]
    /// dependencies = ["requests>=2.0"]
    /// ```
    ///
    /// After applying "Update Dependency to Latest Version":
    /// ```toml
    /// [project]
    /// dependencies = ["requests==2.33.1"]
    /// ```
    UpdateDependencyToLatestVersion,
}

impl CodeActionRefactorRewriteName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UseWorkspaceDependency => "Use Workspace Dependency",
            Self::AddToWorkspaceAndUseWorkspaceDependency => {
                "Add to Workspace and Use Workspace Dependency"
            }
            Self::UpdateDependencyToLatestVersion => "Update Dependency to Latest Version",
        }
    }
}

impl std::fmt::Display for CodeActionRefactorRewriteName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub async fn code_action(
    text_document_uri: &tombi_uri::Uri,
    _root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    toml_version: tombi_config::TomlVersion,
    line_index: &tombi_text::LineIndex,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    // Check if the file is pyproject.toml
    if !text_document_uri.path().ends_with("pyproject.toml") {
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

    let Some(dependency_accessors) = get_dependency_accessors(accessors) else {
        return Ok(None);
    };

    let mut actions = Vec::new();

    // Try to find workspace pyproject.toml
    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        log::warn!(
            "Failed to convert URI to file path: {:?}",
            text_document_uri
        );
        return Ok(None);
    };

    if features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.code_action())
        .and_then(|code_action| code_action.update_dependency_to_latest_version())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
        && let Some(action) = update_dependency_to_latest_version_code_action(
            text_document_uri,
            line_index,
            document_tree,
            dependency_accessors,
            offline,
            cache_options,
        )
        .await?
    {
        actions.push(CodeActionOrCommand::CodeAction(action));
    }

    if dig_keys(document_tree, &["tool", "uv", "workspace"]).is_none() {
        let Some((workspace_path, workspace_root, workspace_document_tree)) =
            find_workspace_pyproject_toml(&pyproject_toml_path, toml_version)
        else {
            log::debug!(
                "No workspace pyproject.toml found: {:?}",
                pyproject_toml_path.display()
            );
            return Ok(None);
        };

        // Load workspace text and create line index for workspace document
        let Ok(workspace_text) = std::fs::read_to_string(&workspace_path) else {
            log::warn!(
                "Failed to read workspace pyproject.toml: {:?}",
                workspace_path.display()
            );
            return Ok(None);
        };
        let workspace_line_index =
            tombi_text::LineIndex::new(&workspace_text, line_index.encoding_kind);

        // Try "Use Workspace Dependency" (when dependency exists in workspace)
        if features
            .and_then(|features| features.lsp())
            .and_then(|lsp| lsp.code_action())
            .and_then(|code_action| code_action.use_workspace_dependency())
            .map(|use_workspace_dependency| use_workspace_dependency.enabled())
            .unwrap_or_default()
            .value()
            && let Some(action) = use_workspace_dependency_code_action(
                text_document_uri,
                line_index,
                document_tree,
                dependency_accessors,
                &workspace_document_tree,
            )
        {
            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        // Try "Add to Workspace and Use Workspace Dependency" (when dependency doesn't exist in workspace)
        if features
            .and_then(|features| features.lsp())
            .and_then(|lsp| lsp.code_action())
            .and_then(|code_action| code_action.add_to_workspace_and_use_workspace_dependency())
            .map(|add_to_workspace_and_use_workspace_dependency| {
                add_to_workspace_and_use_workspace_dependency.enabled()
            })
            .unwrap_or_default()
            .value()
            && let Some(action) = add_workspace_dependency_code_action(
                text_document_uri,
                line_index,
                document_tree,
                dependency_accessors,
                &workspace_path,
                &workspace_line_index,
                &workspace_root,
                &workspace_document_tree,
            )
        {
            log::debug!(
                "Providing 'Add to Workspace and Use Workspace Dependency' code action: action={:?}, uri={:?}",
                action.title,
                text_document_uri
            );
            actions.push(CodeActionOrCommand::CodeAction(action));
        }
    }

    Ok((!actions.is_empty()).then_some(actions))
}

async fn update_dependency_to_latest_version_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<CodeAction>, tower_lsp::jsonrpc::Error> {
    let Some((_, dependency_value)) = tombi_document_tree::dig_accessors(document_tree, accessors)
    else {
        return Ok(None);
    };

    let tombi_document_tree::Value::String(dep_str) = dependency_value else {
        return Ok(None);
    };

    let Some(dependency_requirement) = parse_dependency_requirement(dep_str) else {
        return Ok(None);
    };

    let Some(VersionOrUrl::VersionSpecifier(_)) = dependency_requirement.version_or_url() else {
        return Ok(None);
    };

    let Some(latest_version) = fetch_pypi_project(
        dependency_requirement.requirement.name.as_ref(),
        offline,
        cache_options,
    )
    .await?
    .and_then(|response| response.info.version) else {
        return Ok(None);
    };

    let Some(version_range) = find_version_specifier_range(dep_str.value()) else {
        return Ok(None);
    };
    let new_version_specifier = format_exact_pinned_dependency(&latest_version);
    let current_version_specifier =
        &dep_str.value()[version_range.start.column as usize..version_range.end.column as usize];

    Ok(Some(CodeAction {
        title: CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion.to_string(),
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
                    range: offset_range(dep_str.unquoted_range(), version_range)
                        .into_lsp(line_index),
                    new_text: new_version_specifier,
                })],
            }])),
            change_annotations: None,
        }),
        disabled: (latest_version == current_version_specifier).then(|| CodeActionDisabled {
            reason: "Already at latest version".to_string(),
        }),
        ..Default::default()
    }))
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

fn format_exact_pinned_dependency(latest_version: &str) -> String {
    let Ok(version) = latest_version.parse::<Version>() else {
        return format!("=={latest_version}");
    };

    VersionOrUrl::<VerbatimUrl>::VersionSpecifier(VersionSpecifier::equals_version(version).into())
        .to_string()
}

fn find_version_specifier_range(dependency: &str) -> Option<tombi_text::Range> {
    let marker_start = dependency.find(';').unwrap_or(dependency.len());
    let dependency_without_marker = &dependency[..marker_start];
    let mut cursor = 0;

    while let Some(ch) = dependency_without_marker[cursor..].chars().next() {
        if ch.is_whitespace() || matches!(ch, '[' | '@' | '<' | '>' | '=' | '!' | '~') {
            break;
        }
        cursor += ch.len_utf8();
    }

    if cursor == 0 {
        return None;
    }

    while let Some(ch) = dependency_without_marker[cursor..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        cursor += ch.len_utf8();
    }

    if dependency_without_marker[cursor..].starts_with('[') {
        let extras_relative_end = dependency_without_marker[cursor..].find(']')?;
        cursor += extras_relative_end + 1;
    }

    while let Some(ch) = dependency_without_marker[cursor..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        cursor += ch.len_utf8();
    }

    if dependency_without_marker[cursor..].starts_with('@') {
        return None;
    }

    if !matches!(
        dependency_without_marker[cursor..].chars().next(),
        Some('<' | '>' | '=' | '!' | '~')
    ) {
        return None;
    }

    let version_start = cursor;
    let version_end = dependency_without_marker
        .trim_end_matches(char::is_whitespace)
        .len();

    if version_start >= version_end {
        return None;
    }

    Some(tombi_text::Range::new(
        tombi_text::Position::new(0, version_start as u32),
        tombi_text::Position::new(0, version_end as u32),
    ))
}

fn offset_range(base: tombi_text::Range, relative: tombi_text::Range) -> tombi_text::Range {
    tombi_text::Range::new(
        base.start + tombi_text::RelativePosition::from(relative.start),
        base.start + tombi_text::RelativePosition::from(relative.end),
    )
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
    accessors: &[Accessor],
) -> Option<tombi_ast::Array> {
    // Get the value from document tree to find its range
    let (_, value) = tombi_document_tree::dig_accessors(document_tree, accessors)?;

    let tombi_document_tree::Value::Array(doc_array) = value else {
        return None;
    };

    // Get the range of the array in the document tree
    let target_range = doc_array.range();

    // Use descendants to find the Array with matching range
    for node in root.syntax().descendants() {
        if let Some(array) = tombi_ast::Array::cast(node)
            && array.range() == target_range
        {
            return Some(array);
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
    let values_with_comma: Vec<_> = ast_array.value_or_key_values_with_comma().collect();

    if values_with_comma.is_empty() {
        // Empty array - insert without comma
        return if let Some(dangling_comment) = ast_array
            .dangling_comment_groups()
            .last()
            .and_then(|group| group.comments().last())
        {
            Some((
                dangling_comment.syntax().range().end,
                format!("\n\n{},\n", new_element),
            ))
        } else {
            Some((
                ast_array.bracket_start()?.range().end,
                format!("{}", new_element),
            ))
        };
    }

    if insertion_index == 0 {
        // Insert at the beginning
        let (first_value, _) = values_with_comma.first()?;
        let insert_pos = first_value.syntax().range().start;
        let new_text = format!("{},\n", new_element);
        return Some((insert_pos, new_text));
    }

    if insertion_index >= values_with_comma.len() {
        // Insert at the end
        let (last_value, last_comma) = values_with_comma.last()?;
        if let Some(last_comma) = last_comma {
            let insert_pos = last_comma.range().end;
            let new_text = format!("\n{}, ", new_element);
            return Some((insert_pos, new_text));
        } else {
            let insert_pos = last_value.syntax().range().end;
            let new_text = format!(", {}", new_element);
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
    let new_text = format!("\n{},\n", new_element);
    Some((insert_pos, new_text))
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
    // Get the dependency string from member's document tree
    let (_, dependency_value) = tombi_document_tree::dig_accessors(document_tree, accessors)?;

    let tombi_document_tree::Value::String(dep_str) = dependency_value else {
        return None;
    };

    // Parse the dependency
    let dependency_requirement = parse_dependency_requirement(dep_str)?;

    // If no version specified, don't provide code action
    dependency_requirement.version_or_url()?;

    // Check if this dependency already exists in workspace
    let workspace_dependencies =
        collect_dependency_requirements_from_document_tree(workspace_document_tree);
    if workspace_dependencies.iter().any(
        |DependencyRequirement {
             requirement: workspace_requirement,
             ..
         }| { dependency_requirement.requirement.name == workspace_requirement.name },
    ) {
        return None;
    }

    // Generate workspace URI
    let Ok(workspace_uri) = tombi_uri::Uri::from_file_path(workspace_pyproject_toml_path) else {
        log::warn!(
            "Failed to convert workspace path to URI: {:?}",
            workspace_pyproject_toml_path.display()
        );
        return None;
    };

    // Generate workspace edit (add dependency with version, without extras)
    let workspace_edit = generate_workspace_dependency_edit(
        accessors,
        workspace_line_index,
        workspace_root,
        workspace_document_tree,
        &dependency_requirement,
    )?;

    // Generate member edit (convert to version-less format, preserving extras)
    let member_edit = generate_member_dependency_edit(&dependency_requirement, line_index)?;

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
    accessors: &[Accessor],
    workspace_line_index: &tombi_text::LineIndex,
    workspace_root: &tombi_ast::Root,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    dependency_requirement: &DependencyRequirement,
) -> Option<TextEdit> {
    // Get workspace.dependencies array from document tree
    let (workspace_accessors, workspace_deps) = match tombi_document_tree::dig_accessors(
        workspace_document_tree,
        &accessors[..accessors.len() - 1],
    ) {
        Some((_, value)) => (accessors[..accessors.len() - 1].to_vec(), value),
        None => (
            vec![
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
            ],
            tombi_document_tree::dig_keys(workspace_document_tree, &["project", "dependencies"])?.1,
        ),
    };

    let tombi_document_tree::Value::Array(deps_doc_array) = workspace_deps else {
        return None;
    };

    // Get the AST array to access comma information
    let deps_ast_array = get_ast_array_from_document_tree(
        workspace_root,
        workspace_document_tree,
        &workspace_accessors,
    )?;

    // Get existing package names
    let existing_package_names: Vec<String> = deps_doc_array
        .iter()
        .filter_map(|dep| {
            if let tombi_document_tree::Value::String(dep_str) = dep {
                parse_requirement(dep_str.value()).map(|req| req.name.to_string())
            } else {
                None
            }
        })
        .collect();

    // Convert to &str for calculate_insertion_index
    let existing_packages: Vec<&str> = existing_package_names.iter().map(|s| s.as_str()).collect();

    let package_name = dependency_requirement.requirement.name.to_string();

    // Calculate insertion index
    let insertion_index = calculate_insertion_index(&existing_packages, &package_name);

    // Determine insertion position and comma handling using AST
    let (insertion_range, new_text) = calculate_array_insertion(
        &deps_ast_array,
        insertion_index,
        dependency_requirement.dependency,
    )?;

    Some(TextEdit {
        range: tombi_text::Range::at(insertion_range).into_lsp(workspace_line_index),
        new_text,
    })
}

/// Generate TextEdit for converting member dependency to version-less format
fn generate_member_dependency_edit(
    DependencyRequirement {
        requirement,
        dependency,
    }: &DependencyRequirement,
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

    let tombi_document_tree::Value::String(dep_str) = dep_value else {
        return None;
    };

    // Parse the dependency
    let requirement = parse_requirement(dep_str.value())?;

    // If no version specified, don't provide code action
    requirement.version_or_url.as_ref()?;

    let workspace_dependency_requirements =
        collect_dependency_requirements_from_document_tree(workspace_document_tree);
    let DependencyRequirement {
        requirement: workspace_requirement,
        ..
    } = workspace_dependency_requirements.iter().find(
        |DependencyRequirement {
             requirement: workspace_requirement,
             ..
         }| workspace_requirement.name == requirement.name,
    )?;

    // Format dependency without version (preserving extras)
    let new_dep_str = format_dependency_without_version(workspace_requirement);

    // Use the string's range for replacement
    let range = dep_str.range().into_lsp(line_index);

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
        let requirement = parse_requirement("pydantic>=2.10,<3.0").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        assert!(requirement.extras.is_empty());
        assert!(requirement.version_or_url.is_some());
    }

    #[test]
    fn test_parse_dependency_with_extras() {
        let requirement = parse_requirement("pydantic[email,dotenv]>=2.10").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        let extras: Vec<String> = requirement.extras.iter().map(|e| e.to_string()).collect();
        assert_eq!(extras, vec!["email", "dotenv"]);
        assert!(requirement.version_or_url.is_some());
    }

    #[test]
    fn test_parse_dependency_without_version() {
        let requirement = parse_requirement("pydantic").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        assert!(requirement.extras.is_empty());
        assert!(requirement.version_or_url.is_none());
    }

    #[test]
    fn test_parse_dependency_with_extras_no_version() {
        let requirement = parse_requirement("pydantic[email]").unwrap();
        assert_eq!(requirement.name.to_string(), "pydantic");
        let extras: Vec<String> = requirement.extras.iter().map(|e| e.to_string()).collect();
        assert_eq!(extras, vec!["email"]);
        assert!(requirement.version_or_url.is_none());
    }

    #[test]
    fn test_parse_dependency_invalid() {
        let result = parse_requirement("invalid package name!!!");
        assert!(result.is_none());
    }

    #[test]
    fn test_format_dependency_without_extras() {
        let requirement = parse_requirement("pydantic>=2.10").unwrap();
        let formatted = format_dependency_without_version(&requirement);
        assert_eq!(formatted, "pydantic");
    }

    #[test]
    fn test_format_dependency_with_one_extra() {
        let requirement = parse_requirement("pydantic[email]>=2.10").unwrap();
        let formatted = format_dependency_without_version(&requirement);
        assert_eq!(formatted, "pydantic[email]");
    }

    #[test]
    fn test_format_dependency_with_multiple_extras() {
        let requirement = parse_requirement("pydantic[email,dotenv]>=2.10").unwrap();
        let formatted = format_dependency_without_version(&requirement);
        assert_eq!(formatted, "pydantic[email,dotenv]");
    }

    #[test]
    fn test_find_version_specifier_range_with_marker() {
        let dependency = "requests>=2.0; python_version < '3.13'";
        let range = find_version_specifier_range(dependency).unwrap();
        assert_eq!(
            &dependency[range.start.column as usize..range.end.column as usize],
            ">=2.0"
        );
    }

    #[test]
    fn test_find_version_specifier_range_with_extras() {
        let dependency = "requests[security] >= 2.0, < 3";
        let range = find_version_specifier_range(dependency).unwrap();
        assert_eq!(
            &dependency[range.start.column as usize..range.end.column as usize],
            ">= 2.0, < 3"
        );
    }

    #[test]
    fn test_find_version_specifier_range_returns_none_for_url() {
        let dependency = "requests @ https://example.com/requests.whl";
        assert!(find_version_specifier_range(dependency).is_none());
    }

    #[tokio::test]
    async fn test_code_action_non_pyproject_toml_returns_none() {
        let uri = tombi_uri::Uri::from_file_path("/path/to/Cargo.toml").unwrap();
        let toml_text = r#"
[package]
name = "test"
"#;
        let root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = root
            .clone()
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = code_action(
            &uri,
            &root,
            &document_tree,
            &[],
            tombi_config::TomlVersion::default(),
            &line_index,
            None,
            true,
            None,
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_code_action_workspace_root_skips_workspace_actions() {
        let uri = tombi_uri::Uri::from_file_path("/path/to/pyproject.toml").unwrap();
        let toml_text = r#"
[tool.uv.workspace]
members = ["member1"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = root
            .clone()
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        let result = code_action(
            &uri,
            &root,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Index(0),
            ],
            tombi_config::TomlVersion::default(),
            &line_index,
            None,
            true,
            None,
        )
        .await;

        assert!(result.is_ok());
        let titles = result
            .unwrap()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|action| match action {
                CodeActionOrCommand::CodeAction(action) => Some(action.title),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            !titles
                .iter()
                .any(|title| title == "Use Workspace Dependency")
        );
        assert!(
            !titles
                .iter()
                .any(|title| title == "Add to Workspace and Use Workspace Dependency")
        );
    }

    #[tokio::test]
    async fn test_code_action_invalid_accessor_returns_none() {
        let uri = tombi_uri::Uri::from_file_path("/path/to/pyproject.toml").unwrap();
        let toml_text = r#"
[project]
name = "test"
"#;
        let root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = root
            .clone()
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();
        let line_index = tombi_text::LineIndex::new(toml_text, tombi_text::EncodingKind::default());

        // Test with invalid accessor (not dependencies)
        let result = code_action(
            &uri,
            &root,
            &document_tree,
            &[
                Accessor::Key("project".to_string()),
                Accessor::Key("name".to_string()),
            ],
            tombi_config::TomlVersion::default(),
            &line_index,
            None,
            true,
            None,
        )
        .await;

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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(member_toml).into_syntax_node()).unwrap();
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
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let member_root =
            tombi_ast::Root::cast(tombi_parser::parse(member_toml).into_syntax_node()).unwrap();
        let member_tree = member_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let member_root =
            tombi_ast::Root::cast(tombi_parser::parse(member_toml).into_syntax_node()).unwrap();
        let member_tree = member_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10,<3.0"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10,<3.0"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pydantic>=2.10"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
        let document_root =
            tombi_ast::Root::cast(tombi_parser::parse(toml_text).into_syntax_node()).unwrap();
        let document_tree = document_root
            .try_into_document_tree(tombi_config::TomlVersion::default())
            .unwrap();

        let workspace_toml = r#"
[tool.uv.workspace]
members = ["member"]

[project]
dependencies = ["pytest>=7.0,<8.0"]
"#;
        let workspace_root =
            tombi_ast::Root::cast(tombi_parser::parse(workspace_toml).into_syntax_node()).unwrap();
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
