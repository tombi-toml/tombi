use std::collections::{BTreeMap, BTreeSet};

use crate::dependency::{
    collect_all_dependency_requirements_from_document_tree, normalize_pep503_name,
    parse_requirement,
};
use tombi_ast_syntax::{AstNode, TomlNode};
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{DocumentTree, Value, dig_accessors, dig_keys};
use tombi_extension::{CompletionContent, CompletionEdit, CompletionHint};
use tombi_schema_store::{Accessor, matches_accessors};
use tombi_text::{Offset, Span};

struct PyprojectStringSlot {
    span: Option<Span>,
    prefix: String,
    present: BTreeSet<String>,
    offset: Offset,
    hint: Option<CompletionHint>,
}

impl PyprojectStringSlot {
    fn at(
        document: &DocumentTree<'_>,
        root: &tombi_ast_syntax::Root<'_>,
        offset: Offset,
        accessors: &[Accessor],
        hint: Option<CompletionHint>,
    ) -> Option<Self> {
        let array_accessors = if matches!(accessors.last(), Some(Accessor::Index(_))) {
            &accessors[..accessors.len() - 1]
        } else {
            accessors
        };
        let array = dig_accessors(document, array_accessors).and_then(|(_, value)| match value {
            Value::Array(array) if offset > array.span().start && offset <= array.span().end => {
                Some(array)
            }
            _ => None,
        });
        let value = dig_accessors(document, accessors).map(|(_, value)| value);
        let invalid = root.nodes_at_offset(offset).find_map(|node| match node {
            TomlNode::Invalid(span) => Some(span),
            _ => None,
        });
        let invalid = invalid.filter(|span| {
            !(span.start == offset
                && matches!(
                    hint,
                    Some(CompletionHint::Comma {
                        trailing_comma: Some(_),
                        ..
                    })
                ))
        });
        let span = if let Some(Value::String(value)) =
            value.filter(|value| value.span().contains_inclusive(offset))
        {
            Some(value.span())
        } else if let Some(span) = invalid {
            let text = &root.syntax().text()[span];
            if !text.starts_with(['"', '\''])
                || text.starts_with("\"\"\"")
                || text.starts_with("'''")
                || text.contains(['\n', '\r'])
            {
                return None;
            }
            Some(span)
        } else if array.is_some() {
            None
        } else if matches!(value, Some(Value::Incomplete { .. }))
            && matches!(hint, Some(CompletionHint::EqualTrigger { .. }))
        {
            None
        } else {
            return None;
        };
        let hint = if span.is_none()
            && let Some(array) = array
        {
            Some(CompletionHint::new_array_item(
                array.values().iter().map(Value::span),
                root.syntax().text(),
                offset,
                hint,
            ))
        } else {
            hint
        };
        let prefix = span
            .map(|span| {
                let text = root.syntax().text();
                let raw = &text[Span::new(span.start, offset.min(span.end))];
                let raw = raw
                    .strip_prefix('"')
                    .or_else(|| raw.strip_prefix('\''))
                    .unwrap_or(raw);
                let raw = raw
                    .strip_suffix('"')
                    .or_else(|| raw.strip_suffix('\''))
                    .unwrap_or(raw);
                serde_json::from_str::<String>(&format!("\"{raw}\""))
                    .unwrap_or_else(|_| raw.to_owned())
            })
            .unwrap_or_default();
        let present = match dig_accessors(document, array_accessors) {
            Some((_, Value::Array(array))) => array
                .values()
                .iter()
                .filter_map(|value| match value {
                    Value::String(value) if Some(value.span()) != span => {
                        Some(value.value().to_owned())
                    }
                    _ => None,
                })
                .collect(),
            _ => BTreeSet::new(),
        };
        Some(Self {
            span,
            prefix,
            present,
            offset,
            hint,
        })
    }

    fn edit(&self, label: &str) -> Option<CompletionEdit> {
        if let Some(span) = self.span {
            CompletionEdit::new_string_literal_while_editing(label, span)
        } else {
            CompletionEdit::new_literal(label, self.offset, self.hint)
        }
    }
}

fn table_keys(document: &DocumentTree<'_>, keys: &[&str]) -> BTreeSet<String> {
    match dig_keys(document, keys) {
        Some((_, Value::Table(table))) => table.keys().map(|key| key.value().to_owned()).collect(),
        _ => BTreeSet::new(),
    }
}

enum PyprojectCompletionSite {
    DefaultGroup,
    DependencyGroupKey,
    Conflict(&'static str),
    SourcePackageKey,
    WorkspaceProjectDependency,
}

fn package_names(document: &DocumentTree<'_>) -> BTreeSet<String> {
    let Some((_, Value::Array(array))) = dig_keys(document, &["project", "dependencies"]) else {
        return BTreeSet::new();
    };
    unique_packages(array.values().iter().filter_map(|value| match value {
        Value::String(value) => {
            parse_requirement(value.value()).map(|requirement| requirement.name.to_string())
        }
        _ => None,
    }))
}

fn unique_packages(names: impl IntoIterator<Item = String>) -> BTreeSet<String> {
    names
        .into_iter()
        .map(|name| (normalize_pep503_name(&name), name))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

pub(super) fn complete(
    uri: &tombi_uri::Uri,
    document: &DocumentTree<'_>,
    root: &tombi_ast_syntax::Root<'_>,
    offset: Offset,
    accessors: &[Accessor],
    version: TomlVersion,
    hint: Option<CompletionHint>,
) -> Option<Vec<CompletionContent>> {
    let site = if matches_accessors!(accessors, ["tool", "uv", "default-groups"])
        || matches_accessors!(accessors, ["tool", "uv", "default-groups", _])
    {
        PyprojectCompletionSite::DefaultGroup
    } else if matches_accessors!(accessors, ["tool", "uv", "dependency-groups"])
        || matches_accessors!(accessors, ["tool", "uv", "dependency-groups", _])
    {
        PyprojectCompletionSite::DependencyGroupKey
    } else if matches_accessors!(accessors, ["tool", "uv", "sources"])
        || matches_accessors!(accessors, ["tool", "uv", "sources", _])
    {
        PyprojectCompletionSite::SourcePackageKey
    } else if matches_accessors!(accessors, ["tool", "uv", "conflicts", _, _, "group"]) {
        PyprojectCompletionSite::Conflict("group")
    } else if matches_accessors!(accessors, ["tool", "uv", "conflicts", _, _, "extra"]) {
        PyprojectCompletionSite::Conflict("extra")
    } else if matches_accessors!(accessors, ["project", "dependencies"])
        || matches_accessors!(accessors, ["project", "dependencies", _])
    {
        PyprojectCompletionSite::WorkspaceProjectDependency
    } else {
        return None;
    };
    let package = matches!(
        site,
        PyprojectCompletionSite::SourcePackageKey
            | PyprojectCompletionSite::WorkspaceProjectDependency
    );
    let choices = match site {
        PyprojectCompletionSite::DefaultGroup
        | PyprojectCompletionSite::DependencyGroupKey
        | PyprojectCompletionSite::Conflict("group") => {
            table_keys(document, &["dependency-groups"])
        }
        PyprojectCompletionSite::Conflict(_) => {
            table_keys(document, &["project", "optional-dependencies"])
        }
        PyprojectCompletionSite::SourcePackageKey => unique_packages(
            collect_all_dependency_requirements_from_document_tree(document)
                .into_iter()
                .map(|dependency| dependency.requirement.name.to_string()),
        ),
        PyprojectCompletionSite::WorkspaceProjectDependency => {
            let path = uri.to_file_path().ok()?;
            crate::find_workspace_pyproject_toml(
                &path,
                version,
                crate::UNUSED_ENCODING,
                |workspace, _, tree, _| (workspace != path).then(|| package_names(tree)),
            )
            .flatten()?
        }
    };
    if matches!(
        site,
        PyprojectCompletionSite::DependencyGroupKey | PyprojectCompletionSite::SourcePackageKey
    ) {
        return complete_keys(document, root, offset, accessors, hint, choices, package);
    }
    if matches!(
        site,
        PyprojectCompletionSite::DefaultGroup | PyprojectCompletionSite::WorkspaceProjectDependency
    ) {
        let array_accessors = if matches!(accessors.last(), Some(Accessor::Index(_))) {
            &accessors[..accessors.len() - 1]
        } else {
            accessors
        };
        if !matches!(
            dig_accessors(document, array_accessors),
            Some((_, Value::Array(_)))
        ) {
            return Some(Vec::new());
        }
    }
    let mut slot = PyprojectStringSlot::at(document, root, offset, accessors, hint)?;
    if let PyprojectCompletionSite::Conflict(field) = site {
        if let Some((_, Value::Array(conflict))) = dig_accessors(document, &accessors[..4]) {
            for entry in conflict.values() {
                if let Value::Table(entry) = entry
                    && let Some(Value::String(value)) = entry.get(field)
                    && Some(value.span()) != slot.span
                {
                    slot.present.insert(value.value().to_owned());
                }
            }
        }
    }
    if package {
        slot.present = slot
            .present
            .into_iter()
            .filter_map(|value| {
                parse_requirement(&value)
                    .map(|requirement| normalize_pep503_name(requirement.name.as_ref()))
            })
            .collect();
    }
    Some(
        choices
            .into_iter()
            .filter(|choice| {
                if package {
                    normalize_pep503_name(choice).starts_with(&normalize_pep503_name(&slot.prefix))
                        && !slot.present.contains(&normalize_pep503_name(choice))
                } else {
                    choice.starts_with(&slot.prefix) && !slot.present.contains(choice)
                }
            })
            .map(|choice| {
                let label = serde_json::Value::String(choice).to_string();
                let edit = slot.edit(&label);
                CompletionContent::new_enum_value(label, None, None, edit, None, None)
            })
            .collect(),
    )
}

fn complete_keys(
    document: &DocumentTree<'_>,
    root: &tombi_ast_syntax::Root<'_>,
    offset: Offset,
    accessors: &[Accessor],
    hint: Option<CompletionHint>,
    choices: BTreeSet<String>,
    package: bool,
) -> Option<Vec<CompletionContent>> {
    let (_, Value::Table(table)) = dig_accessors(document, &accessors[..3])? else {
        return None;
    };
    let (span, prefix) = if let Some(Accessor::Key(name)) = accessors.get(3) {
        let (key, _) = table.get_key_value(name.as_str())?;
        if !key.span().contains_inclusive(offset) {
            return None;
        }
        let raw = &root.syntax().text()[Span::new(key.span().start, offset)];
        (key.span(), raw.trim_matches(['"', '\'']).to_owned())
    } else {
        (Span::empty(offset), String::new())
    };
    let present: BTreeSet<_> = table
        .keys()
        .filter(|key| key.span() != span)
        .map(|key| {
            if package {
                normalize_pep503_name(key.value())
            } else {
                key.value().to_owned()
            }
        })
        .collect();
    Some(
        choices
            .into_iter()
            .filter(|choice| {
                let choice = if package {
                    normalize_pep503_name(choice)
                } else {
                    choice.clone()
                };
                let prefix = if package {
                    normalize_pep503_name(&prefix)
                } else {
                    prefix.to_owned()
                };
                choice.starts_with(&prefix) && !present.contains(&choice)
            })
            .map(|choice| {
                let key = if choice
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
                {
                    choice
                } else {
                    serde_json::Value::String(choice).to_string()
                };
                let edit = CompletionEdit::new_key(&key, span, hint);
                let mut content =
                    CompletionContent::new_enum_value(key, None, None, edit, None, None);
                content.kind = tombi_extension::CompletionKind::Key;
                content
            })
            .collect(),
    )
}
