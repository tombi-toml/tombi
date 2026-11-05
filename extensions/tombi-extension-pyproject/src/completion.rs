use tombi_ast_syntax::{AstNode, TomlNode};
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{Value, dig_accessors, dig_keys};
use tombi_extension::{
    CompletionContent, CompletionEdit, CompletionHint, completion_directory_path,
    completion_file_path_from_uri,
};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::accessors::is_dependency_groups_include_group_accessors;

pub async fn completion(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    root: &tombi_ast_syntax::Root<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
    _toml_version: TomlVersion,
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

    Ok(completion_include_group(
        document_tree,
        root,
        offset,
        accessors,
        completion_hint,
    ))
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
    root: &tombi_ast_syntax::Root<'_>,
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
            match incomplete_include_group_string_span(root, offset, equal_span) {
                Some(span) => IncludeGroupCompletionTarget::Replace { span, prefix: None },
                None => IncludeGroupCompletionTarget::Insert { offset, hint },
            }
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

    Some(
        groups
            .keys()
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

fn incomplete_include_group_string_span(
    root: &tombi_ast_syntax::Root<'_>,
    offset: tombi_text::Offset,
    equal_span: tombi_text::Span,
) -> Option<tombi_text::Span> {
    let mut nodes = root.nodes_at_offset(offset);
    let span = nodes.find_map(|node| match node {
        TomlNode::Invalid(span) => Some(span),
        _ => None,
    })?;
    let key_value = nodes.find_map(|node| match node {
        TomlNode::KeyValue(key_value) => Some(key_value),
        _ => None,
    })?;
    if key_value.value().is_some()
        || key_value.eq()?.span() != equal_span
        || span.start < equal_span.end
        || !span.contains_inclusive(offset)
    {
        return None;
    }

    let text = root.syntax().text();
    let token = &text[span];
    if !(token.starts_with('"') || token.starts_with('\''))
        || token.starts_with("\"\"\"")
        || token.starts_with("'''")
        || token.contains(['\n', '\r'])
        || !text[tombi_text::Span::new(equal_span.end, span.start)]
            .trim()
            .is_empty()
    {
        return None;
    }

    Some(span)
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
