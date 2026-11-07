use std::collections::BTreeSet;

use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{Value, dig_accessors, dig_keys};
use tombi_extension::{
    CompletionContent, CompletionEdit, CompletionHint, completion_directory_path,
    completion_file_path_from_uri,
};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::accessors::{
    is_dependency_groups_include_group_accessors, is_dependency_name_accessors,
};
use crate::dependency::{normalize_pep503_name, parse_requirement};
use crate::manifest::{find_workspace_pyproject_toml, get_project_name};
use crate::workspace::{
    extract_exclude_patterns, extract_member_patterns, find_pyproject_toml_paths,
};

pub async fn completion(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    completion_hint: Option<CompletionHint>,
    in_comment: bool,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp_server::jsonrpc::Error> {
    if in_comment {
        return Ok(None);
    }

    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    if !features
        .map(|features| features.enabled())
        .unwrap_or_default()
        .value()
        || !features
            .and_then(|features| features.lsp())
            .and_then(|lsp| lsp.completion())
            .map(|completion| completion.enabled())
            .unwrap_or_default()
            .value()
    {
        return Ok(None);
    }

    if let Some(completions) = features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
        .and_then(|completion| completion.path())
        .map(|path| path.enabled())
        .unwrap_or_default()
        .value()
        .then(|| {
            completion_pyproject_file_path(text_document_uri, document_tree, offset, accessors)
        })
        .flatten()
    {
        return Ok(Some(completions));
    }

    if let Some(completions) =
        completion_include_group(document_tree, offset, accessors, completion_hint)
    {
        return Ok(Some(completions));
    }

    if let Some(completions) = completion_uv_declared_name(document_tree, offset, accessors) {
        return Ok(Some(completions));
    }

    Ok(completion_workspace_dependency_name(
        text_document_uri,
        document_tree,
        offset,
        accessors,
        toml_version,
    ))
}

/// Completes group and extra names in `tool.uv` that refer to tables declared in this file.
fn completion_uv_declared_name(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
) -> Option<Vec<CompletionContent>> {
    let (table_keys, detail): (&[&str], &str) =
        if matches_accessors!(accessors, ["tool", "uv", "default-groups", _])
            || matches_accessors!(accessors, ["tool", "uv", "conflicts", _, _, "group"])
        {
            (&["dependency-groups"], "Dependency group")
        } else if matches_accessors!(accessors, ["tool", "uv", "conflicts", _, _, "extra"]) {
            (&["project", "optional-dependencies"], "Extra")
        } else {
            return None;
        };

    let (_, Value::String(value)) = dig_accessors(document_tree, accessors)? else {
        return None;
    };
    if !value.span().contains_inclusive(offset) {
        return None;
    }

    let (_, Value::Table(names)) = dig_keys(document_tree, table_keys)? else {
        return None;
    };

    let completions = names
        .keys()
        .filter(|name| name.value().starts_with(value.value()))
        .map(|name| {
            let label = serde_json::Value::String(name.value().to_owned()).to_string();
            let edit = CompletionEdit::new_string_literal_while_editing(&label, value.span());
            CompletionContent::new_enum_value(
                label,
                Some(detail.to_string()),
                None,
                edit,
                None,
                None,
            )
        })
        .collect_vec();

    (!completions.is_empty()).then_some(completions)
}

/// Completes dependency names of a workspace member from the workspace root.
///
/// Candidates are the names in the workspace root's `project.dependencies`,
/// `project.optional-dependencies` and `dependency-groups`, plus the members
/// that this file references with `tool.uv.sources` and `workspace = true`.
fn completion_workspace_dependency_name(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
    toml_version: TomlVersion,
) -> Option<Vec<CompletionContent>> {
    if !is_dependency_name_accessors(accessors) {
        return None;
    }

    let (_, Value::String(value)) = dig_accessors(document_tree, accessors)? else {
        return None;
    };
    if !value.span().contains_inclusive(offset) {
        return None;
    }

    let pyproject_toml_path = text_document_uri.to_file_path().ok()?;
    let workspace_dependency_names = find_workspace_pyproject_toml(
        &pyproject_toml_path,
        toml_version,
        tombi_text::EncodingKind::default(),
        |workspace_pyproject_toml_path, _, workspace_document_tree, _| {
            // The file is the workspace root itself, so there is no workspace to complete from.
            if workspace_pyproject_toml_path == pyproject_toml_path {
                return None;
            }

            let member_patterns = extract_member_patterns(workspace_document_tree, &[]);
            let exclude_patterns = extract_exclude_patterns(workspace_document_tree);
            let workspace_dir_path = workspace_pyproject_toml_path.parent()?;
            let is_member =
                find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
                    .any(|(_, member_pyproject_toml_path)| {
                        is_same_file(&member_pyproject_toml_path, &pyproject_toml_path)
                    });
            if !is_member {
                return None;
            }

            let mut dependency_arrays = Vec::new();
            if let Some((_, Value::Array(dependencies))) =
                dig_keys(workspace_document_tree, &["project", "dependencies"])
            {
                dependency_arrays.push(dependencies);
            }
            for table_keys in [
                &["project", "optional-dependencies"][..],
                &["dependency-groups"][..],
            ] {
                if let Some((_, Value::Table(table))) =
                    dig_keys(workspace_document_tree, table_keys)
                {
                    dependency_arrays.extend(table.values().filter_map(|value| match value {
                        Value::Array(dependencies) => Some(dependencies),
                        _ => None,
                    }));
                }
            }

            Some(
                dependency_arrays
                    .into_iter()
                    .flat_map(|dependencies| dependencies.iter())
                    .filter_map(|dependency| match dependency {
                        Value::String(dependency) => parse_requirement(dependency.value()),
                        _ => None,
                    })
                    .map(|requirement| requirement.name.to_string())
                    .collect::<BTreeSet<_>>(),
            )
        },
    )
    .flatten()?;

    let workspace_member_names = match dig_keys(document_tree, &["tool", "uv", "sources"]) {
        Some((_, Value::Table(sources))) => sources
            .key_values()
            .iter()
            .filter(|(_, source)| is_workspace_source(source))
            .map(|(name, _)| name.value().to_owned())
            .collect::<BTreeSet<_>>(),
        _ => BTreeSet::new(),
    };

    let project_name =
        get_project_name(document_tree).map(|name| normalize_pep503_name(name.value()));
    let completions = workspace_dependency_names
        .into_iter()
        .chain(workspace_member_names)
        .filter(|name| Some(normalize_pep503_name(name)) != project_name)
        .unique()
        .filter(|name| name.starts_with(value.value()))
        .map(|name| {
            let label = serde_json::Value::String(name).to_string();
            let edit = CompletionEdit::new_string_literal_while_editing(&label, value.span());
            CompletionContent::new_enum_value(
                label,
                Some("Workspace dependency".to_string()),
                None,
                edit,
                None,
                None,
            )
        })
        .collect_vec();

    (!completions.is_empty()).then_some(completions)
}

fn is_same_file(path: &std::path::Path, other: &std::path::Path) -> bool {
    match (tombi_fs::canonicalize(path), tombi_fs::canonicalize(other)) {
        (Ok(path), Ok(other)) => path == other,
        _ => path == other,
    }
}

fn is_workspace_source(source: &Value<'_>) -> bool {
    let Value::Table(source) = source else {
        return false;
    };

    matches!(
        source.get_key_value("workspace"),
        Some((_, Value::Boolean(is_workspace))) if is_workspace.value()
    )
}

enum IncludeGroupCompletionTarget<'a> {
    Replace {
        span: tombi_text::Span,
        prefix: Option<&'a str>,
    },
    Insert {
        offset: tombi_text::Offset,
        hint: CompletionHint,
    },
}

impl<'a> IncludeGroupCompletionTarget<'a> {
    fn prefix(&self) -> Option<&'a str> {
        match self {
            Self::Replace { prefix, .. } => *prefix,
            Self::Insert { .. } => None,
        }
    }

    fn edit(&self, label: &str) -> Option<CompletionEdit> {
        match self {
            Self::Replace { span, .. } => {
                CompletionEdit::new_string_literal_while_editing(label, *span)
            }
            Self::Insert { offset, hint } => {
                CompletionEdit::new_literal(label, *offset, Some(*hint))
            }
        }
    }
}

fn completion_include_group(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
) -> Option<Vec<CompletionContent>> {
    if !is_dependency_groups_include_group_accessors(accessors) {
        return None;
    }

    let (_, value) = dig_accessors(document_tree, accessors)?;
    let target = match (value, completion_hint) {
        (Value::String(value), _) if value.span().contains_inclusive(offset) => {
            IncludeGroupCompletionTarget::Replace {
                span: value.span(),
                prefix: Some(value.value()),
            }
        }
        (
            Value::Incomplete { span },
            Some(
                hint @ CompletionHint::EqualTrigger {
                    span: equal_span, ..
                },
            ),
        ) if span.is_empty() && offset >= equal_span.end => {
            IncludeGroupCompletionTarget::Insert { offset, hint }
        }
        (Value::Incomplete { span }, _) if !span.is_empty() && span.contains_inclusive(offset) => {
            IncludeGroupCompletionTarget::Replace {
                span: *span,
                prefix: None,
            }
        }
        _ => return None,
    };

    let (_, Value::Table(groups)) = dig_keys(document_tree, &["dependency-groups"])? else {
        return None;
    };

    let current_group = match accessors.get(1) {
        Some(Accessor::Key(key)) => Some(key.as_str()),
        _ => None,
    };

    Some(
        groups
            .keys()
            .filter(|key| Some(key.value()) != current_group)
            .filter(|key| {
                target
                    .prefix()
                    .is_none_or(|prefix| key.value().starts_with(prefix))
            })
            .map(|key| {
                let label = serde_json::Value::String(key.value().to_owned()).to_string();
                let edit = target.edit(&label);
                CompletionContent::new_enum_value(
                    label,
                    Some("Dependency group".to_string()),
                    None,
                    edit,
                    None,
                    None,
                )
            })
            .collect(),
    )
}

fn completion_pyproject_file_path(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
) -> Option<Vec<CompletionContent>> {
    // Pyproject workspace: directory paths only (members, exclude)
    if matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _])
        || matches_accessors!(accessors, ["tool", "uv", "workspace", "exclude", _])
    {
        return completion_directory_path(text_document_uri, document_tree, offset, accessors);
    }

    // Pyproject sources: path to local package (file or directory)
    if matches_accessors!(accessors, ["tool", "uv", "sources", _, "path"]) {
        return completion_file_path_from_uri(
            text_document_uri,
            document_tree,
            offset,
            accessors,
            Some(&[]),
        );
    }

    // Pyproject standard: build-system, readme, license
    if matches_accessors!(accessors, ["build-system", "backend-path", _])
        || matches_accessors!(accessors, ["project", "readme"])
        || matches_accessors!(accessors, ["project", "readme", "file"])
        || matches_accessors!(accessors, ["project", "license", "file"])
        || matches_accessors!(accessors, ["project", "license-files", _])
    {
        return completion_file_path_from_uri(
            text_document_uri,
            document_tree,
            offset,
            accessors,
            Some(&[]),
        );
    }

    None
}
