use std::collections::BTreeSet;

use tombi_ast_syntax::{AstNode, TomlNode};
use tombi_document_tree_syntax::{DocumentTree, Value, dig_accessors, dig_keys};
use tombi_extension::{CompletionContent, CompletionEdit, CompletionHint};
use tombi_schema_store::{Accessor, matches_accessors};
use tombi_text::{Offset, Span};

enum CargoCompletionSite<'a> {
    DefaultMember,
    RequiredFeature,
    FeatureActivation(&'a str),
}

struct CargoStringSlot {
    span: Option<Span>,
    prefix: Option<tombi_extension::CompletionStringPrefix>,
    present: BTreeSet<String>,
    offset: Offset,
    hint: Option<CompletionHint>,
}

impl CargoStringSlot {
    fn at(
        document: &DocumentTree<'_>,
        root: &tombi_ast_syntax::Root<'_>,
        offset: Offset,
        accessors: &[Accessor],
        hint: Option<CompletionHint>,
        version: tombi_config::TomlVersion,
    ) -> Option<Self> {
        let array_accessors = if matches!(accessors.last(), Some(Accessor::Index(_))) {
            &accessors[..accessors.len() - 1]
        } else {
            accessors
        };
        let array = dig_accessors(document, array_accessors).and_then(|(_, value)| match value {
            Value::Array(array) if offset > array.span().start && offset <= array.span().end => {
                root.array_at_span(array.span())
                    .filter(|syntax| {
                        syntax
                            .bracket_end()
                            .is_none_or(|bracket| offset <= bracket.span().start)
                    })
                    .map(|_| array)
            }
            _ => None,
        });
        array?;
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
        } else if array.is_some_and(|array| {
            array.values().iter().any(|value| {
                !matches!(value, Value::Incomplete { .. })
                    && value.span().start < offset
                    && offset < value.span().end
            })
        }) {
            return None;
        } else if array.is_some()
            || (matches!(value, Some(Value::Incomplete { .. }))
                && matches!(hint, Some(CompletionHint::EqualTrigger { .. })))
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
        let prefix = span.and_then(|span| {
            tombi_extension::CompletionStringPrefix::parse(
                root.syntax().text(),
                span,
                offset,
                version,
            )
        });
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

    fn prefix(&self) -> &str {
        self.prefix
            .as_ref()
            .map_or("", tombi_extension::CompletionStringPrefix::value)
    }

    fn edit(&self, value: &str) -> Option<CompletionEdit> {
        if let Some(span) = self.span {
            if self.prefix().starts_with("dep:") {
                CompletionEdit::new_string_literal_while_editing_with_cursor(value, span)
            } else {
                let label = serde_json::Value::String(value.to_owned()).to_string();
                CompletionEdit::new_string_literal_while_editing(&label, span)
            }
        } else {
            let label = serde_json::Value::String(value.to_owned()).to_string();
            CompletionEdit::new_literal(&label, self.offset, self.hint)
        }
    }
}

fn table_keys(document: &DocumentTree<'_>, keys: &[&str]) -> BTreeSet<String> {
    match dig_keys(document, keys) {
        Some((_, Value::Table(table))) => table.keys().map(|key| key.value().to_owned()).collect(),
        _ => BTreeSet::new(),
    }
}

fn optional_dependencies(document: &DocumentTree<'_>) -> BTreeSet<String> {
    fn collect(dependencies: &Value<'_>, names: &mut BTreeSet<String>) {
        if let Value::Table(dependencies) = dependencies {
            for (name, dependency) in dependencies.key_values() {
                if let Value::Table(dependency) = dependency
                    && let Some(Value::Boolean(optional)) = dependency.get("optional")
                    && optional.value()
                {
                    names.insert(name.value().to_owned());
                }
            }
        }
    }
    let mut names = BTreeSet::new();
    for key in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some((_, dependencies)) = dig_keys(document, &[key]) {
            collect(dependencies, &mut names);
        }
        if let Some((_, Value::Table(targets))) = dig_keys(document, &["target"]) {
            for (_, target) in targets.key_values() {
                if let Value::Table(target) = target
                    && let Some(dependencies) = target.get(key)
                {
                    collect(dependencies, &mut names);
                }
            }
        }
    }
    names
}

fn explicitly_activated_dependencies(document: &DocumentTree<'_>) -> BTreeSet<String> {
    let Some((_, Value::Table(features))) = dig_keys(document, &["features"]) else {
        return BTreeSet::new();
    };
    features
        .values()
        .filter_map(|value| match value {
            Value::Array(values) => Some(values),
            _ => None,
        })
        .flat_map(|values| values.values())
        .filter_map(|value| match value {
            Value::String(value) => value.value().strip_prefix("dep:"),
            _ => None,
        })
        .filter(|dependency| !dependency.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn complete(
    document: &DocumentTree<'_>,
    root: &tombi_ast_syntax::Root<'_>,
    offset: Offset,
    accessors: &[Accessor],
    hint: Option<CompletionHint>,
    version: tombi_config::TomlVersion,
) -> Option<Vec<CompletionContent>> {
    let site = if matches_accessors!(accessors, ["workspace", "default-members"])
        || matches_accessors!(accessors, ["workspace", "default-members", _])
    {
        CargoCompletionSite::DefaultMember
    } else if accessors
        .first()
        .and_then(Accessor::as_key)
        .is_some_and(|key| matches!(key, "bin" | "bench" | "test" | "example"))
        && (matches_accessors!(accessors, [_, _, "required-features"])
            || matches_accessors!(accessors, [_, _, "required-features", _]))
    {
        CargoCompletionSite::RequiredFeature
    } else if (matches_accessors!(accessors, ["features", _])
        || matches_accessors!(accessors, ["features", _, _]))
        && let Some(Accessor::Key(owner)) = accessors.get(1)
    {
        CargoCompletionSite::FeatureActivation(owner)
    } else {
        return None;
    };
    let Some(slot) = CargoStringSlot::at(document, root, offset, accessors, hint, version) else {
        return Some(Vec::new());
    };
    let mut choices = match site {
        CargoCompletionSite::DefaultMember => match dig_keys(document, &["workspace", "members"]) {
            Some((_, Value::Array(array))) => array
                .values()
                .iter()
                .filter_map(|value| match value {
                    Value::String(value) => Some(value.value().to_owned()),
                    _ => None,
                })
                .collect(),
            _ => BTreeSet::new(),
        },
        CargoCompletionSite::RequiredFeature => table_keys(document, &["features"]),
        CargoCompletionSite::FeatureActivation(owner) => {
            let dependencies = optional_dependencies(document);
            let mut features = table_keys(document, &["features"]);
            let explicitly_activated = explicitly_activated_dependencies(document);
            features.extend(
                dependencies
                    .iter()
                    .filter(|name| !explicitly_activated.contains(*name))
                    .cloned(),
            );
            features.remove(owner);
            features.extend(dependencies.into_iter().map(|name| format!("dep:{name}")));
            features
        }
    };
    choices.retain(|choice| choice.starts_with(slot.prefix()) && !slot.present.contains(choice));
    Some(
        choices
            .into_iter()
            .map(|choice| {
                let label = serde_json::Value::String(choice.clone()).to_string();
                let edit = slot.edit(&choice);
                let mut content =
                    CompletionContent::new_enum_value(label, None, None, edit, None, None);
                content.filter_text = slot
                    .prefix
                    .as_ref()
                    .map(|prefix| prefix.filter_text_for_suffix(&choice[slot.prefix().len()..]));
                content
            })
            .collect(),
    )
}
