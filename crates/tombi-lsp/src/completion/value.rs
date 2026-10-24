mod all_of;
mod any_of;
mod array;
mod boolean;
mod branch_result;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod one_of;
mod string;
mod table;

use crate::hover::constraints::{EnumAndDefault, ValueConstraints};
use crate::schema_tooltip::{SchemaTooltip, SchemaTooltipContent};
pub use all_of::find_all_of_completion_items;
pub use any_of::find_any_of_completion_items;
use array::type_hint_array;
use boolean::type_hint_boolean;
use float::type_hint_float;
use integer::type_hint_integer;
use local_date::type_hint_local_date;
use local_date_time::type_hint_local_date_time;
use local_time::type_hint_local_time;
use offset_date_time::type_hint_offset_date_time;
pub use one_of::find_one_of_completion_items;
use string::type_hint_string;
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, AnythingSchema, ArraySchema, BooleanSchema, CurrentSchema, FloatSchema,
    IntegerSchema, LocalDateSchema, LocalDateTimeSchema, LocalTimeSchema, OffsetDateTimeSchema,
    SchemaDefinitions, SchemaStore, SchemaUri, SchemaView, StringSchema, TableSchema,
};

use super::{
    CompletionCandidate, CompletionContent, CompletionHint, FindCompletionContents,
    composite_detail, composite_documentation, schema_completion::SchemaCompletion,
};

impl FindCompletionContents for tombi_document_tree_syntax::Value<'_> {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        cursor: crate::CursorPosition<'a>,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        let offset = cursor.offset();
        log::trace!("self = {:?}", self);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("keys = {:?}", keys);
        log::trace!("current_schema = {:?}", current_schema);
        log::trace!("completion_hint = {:?}", completion_hint);

        async move {
            if let Some(Ok(current_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .find_completion_contents(
                        cursor,
                        keys,
                        accessors,
                        Some(&current_schema),
                        schema_context,
                        completion_hint,
                    )
                    .await;
            }

            let instance_type = tombi_schema_store::SchemaType::from_value_type(
                tombi_document_tree_syntax::ValueImpl::value_type(self),
            );
            let projected_schema = current_schema.and_then(|schema| {
                instance_type.and_then(|instance_type| {
                    schema.for_instance_type(instance_type, schema_context.string_formats())
                })
            });
            let unprojected_schema = current_schema;
            let current_schema = match current_schema {
                Some(schema) if schema.semantic_schema.is_some() && instance_type.is_some() => {
                    projected_schema.as_ref()
                }
                schema => schema,
            };

            match self {
                Self::Boolean(boolean) => {
                    boolean
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Integer(integer) => {
                    integer
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Float(float) => {
                    float
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::OffsetDateTime(offset_date_time) => {
                    offset_date_time
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::LocalDateTime(local_date_time) => {
                    local_date_time
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::LocalDate(local_date) => {
                    local_date
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::LocalTime(local_time) => {
                    local_time
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::String(string_value) => {
                    string_value
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Array(array)
                    if array.kind() == tombi_document_tree_syntax::ArrayKind::ArrayOfTable
                        && matches!(
                            completion_hint,
                            Some(CompletionHint::InTableHeader { .. })
                        )
                        && keys.is_empty()
                        && current_schema.is_none()
                        && unprojected_schema.is_some() =>
                {
                    // `[[table.█]]` is parsed as `[[table]]`, but the schema of `table` is not an array.
                    // The cursor is in the key of the table, so complete with the element table.
                    match array.values().iter().find(|value| value.contains(offset)) {
                        Some(value) => {
                            value
                                .find_completion_contents(
                                    cursor,
                                    keys,
                                    accessors,
                                    unprojected_schema,
                                    schema_context,
                                    completion_hint,
                                )
                                .await
                        }
                        None => Vec::new(),
                    }
                }
                Self::Array(array) => {
                    array
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Table(_)
                    if matches!(completion_hint, Some(CompletionHint::InTableHeader { .. }))
                        && keys.is_empty()
                        && current_schema.is_none()
                        && let Some(unprojected_schema) = unprojected_schema
                        && unprojected_schema
                            .for_instance_type(
                                tombi_schema_store::SchemaType::Array,
                                schema_context.string_formats(),
                            )
                            .is_some_and(|schema| {
                                matches!(schema.schema_view.as_ref(), SchemaView::Array(_))
                            }) =>
                {
                    // `[array.█]` is parsed as `[array]`. A table under an array schema means
                    // `[[array]]` has not been declared above, so no key can follow the dot.
                    Vec::new()
                }
                Self::Table(table) => {
                    table
                        .find_completion_contents(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Incomplete { .. } => match current_schema {
                    Some(current_schema) => {
                        SchemaCompletion
                            .find_completion_contents(
                                cursor,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                completion_hint,
                            )
                            .await
                    }
                    None => {
                        let last_key = keys.last();

                        match (&last_key, completion_hint) {
                            (Some(last_key), Some(CompletionHint::EqualTrigger { span, .. }))
                                if span.end < offset =>
                            {
                                vec![CompletionContent::new_type_hint_key(
                                    last_key.value(),
                                    last_key.span(),
                                    None,
                                    completion_hint,
                                )]
                            }
                            _ => type_hint_value(last_key, offset, None, completion_hint),
                        }
                    }
                },
            }
        }
        .boxed()
    }
}

pub fn type_hint_value(
    key: Option<&tombi_document_tree_syntax::Key<'_>>,
    offset: tombi_text::Offset,
    schema_base_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let mut completion_contents = itertools::concat([
        type_hint_boolean(offset, schema_base_uri, completion_hint),
        type_hint_integer(offset, schema_base_uri, completion_hint),
        type_hint_float(offset, schema_base_uri, completion_hint),
        type_hint_string(offset, schema_base_uri, completion_hint),
        type_hint_local_date_time(offset, schema_base_uri, completion_hint),
        type_hint_local_date(offset, schema_base_uri, completion_hint),
        type_hint_local_time(offset, schema_base_uri, completion_hint),
        type_hint_offset_date_time(offset, schema_base_uri, completion_hint),
        type_hint_array(offset, schema_base_uri, completion_hint),
        vec![CompletionContent::new_type_hint_inline_table(
            offset,
            schema_base_uri,
            completion_hint,
        )],
    ]);

    if let Some(key) = key {
        let need_key_hint = match completion_hint {
            Some(
                CompletionHint::DotTrigger { span, .. } | CompletionHint::EqualTrigger { span, .. },
            ) => span.end == offset || span.end <= key.span().start,
            Some(
                CompletionHint::InTableHeader { .. }
                | CompletionHint::InArray { .. }
                | CompletionHint::Comma { .. },
            )
            | None => true,
        };
        if need_key_hint {
            completion_contents.push(CompletionContent::new_type_hint_key(
                key.value(),
                key.span(),
                schema_base_uri,
                completion_hint,
            ));
        }
    } else {
        completion_contents.push(CompletionContent::new_type_hint_empty_key(
            offset,
            schema_base_uri,
            completion_hint,
        ))
    }

    completion_contents
}

impl CompletionCandidate for SchemaView {
    async fn detail(
        &self,
        schema_base_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &SchemaStore,
        parent_dynamic_scope: &[SchemaUri],
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        match self {
            Self::OneOf(schema) => {
                composite_detail(
                    schema,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
                .await
            }
            Self::AnyOf(schema) => {
                composite_detail(
                    schema,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
                .await
            }
            Self::AllOf(schema) => {
                composite_detail(
                    schema,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
                .await
            }
            _ => title_and_description(self).and_then(|(title, _)| title.map(ToString::to_string)),
        }
    }

    async fn documentation(
        &self,
        schema_base_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &SchemaStore,
        parent_dynamic_scope: &[SchemaUri],
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        match self {
            Self::OneOf(schema) => {
                composite_documentation(
                    schema,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
                .await
            }
            Self::AnyOf(schema) => {
                composite_documentation(
                    schema,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
                .await
            }
            Self::AllOf(schema) => {
                composite_documentation(
                    schema,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
                .await
            }
            _ => {
                let (_, description) = title_and_description(self)?;
                Some(
                    schema_tooltip_content(self, description.map(ToString::to_string))
                        .await
                        .to_string(),
                )
            }
        }
    }
}

/// The JSON Schema `title` / `description` of a non-composite schema.
fn title_and_description(schema_view: &SchemaView) -> Option<(Option<&str>, Option<&str>)> {
    match schema_view {
        SchemaView::Boolean(BooleanSchema {
            title, description, ..
        })
        | SchemaView::Integer(IntegerSchema {
            title, description, ..
        })
        | SchemaView::Float(FloatSchema {
            title, description, ..
        })
        | SchemaView::String(StringSchema {
            title, description, ..
        })
        | SchemaView::OffsetDateTime(OffsetDateTimeSchema {
            title, description, ..
        })
        | SchemaView::LocalDateTime(LocalDateTimeSchema {
            title, description, ..
        })
        | SchemaView::LocalDate(LocalDateSchema {
            title, description, ..
        })
        | SchemaView::LocalTime(LocalTimeSchema {
            title, description, ..
        })
        | SchemaView::Array(ArraySchema {
            title, description, ..
        })
        | SchemaView::Table(TableSchema {
            title, description, ..
        })
        | SchemaView::Anything(AnythingSchema {
            title, description, ..
        }) => Some((title.as_deref(), description.as_deref())),
        _ => None,
    }
}

/// Hover-style tooltip content for a schema.
///
/// `schema` is left empty: completion items link their schema via `schema_base_uri`.
pub(super) async fn schema_tooltip_content(
    schema_view: &SchemaView,
    description: Option<String>,
) -> SchemaTooltip {
    SchemaTooltip::Content(SchemaTooltipContent {
        title: None,
        description,
        value_type: schema_view.value_type().await.to_string(),
        constraints: constraints_markdown(schema_view.enum_and_default()),
        schema: None,
    })
}

/// Documentation for a key candidate resolved out of the property schema `parent`.
///
/// Like hover, `Value:` is the property's type (not the candidate's), marked nullable
/// when the key is optional, and a composite `parent`'s `default` applies to a candidate
/// that defines none.
pub(super) async fn key_documentation(
    parent: &SchemaView,
    candidate: &SchemaView,
    documentation: Option<String>,
    key_name: &str,
    required_keys: Option<&Vec<String>>,
) -> Option<String> {
    let Some((_, description)) = title_and_description(candidate) else {
        return documentation;
    };
    let mut constraints = candidate.enum_and_default();
    if matches!(
        parent,
        SchemaView::OneOf(_) | SchemaView::AnyOf(_) | SchemaView::AllOf(_)
    ) {
        constraints.default = constraints.default.or(parent.enum_and_default().default);
    }
    let required = required_keys.is_some_and(|keys| keys.iter().any(|key| key == key_name));
    let value_type = crate::hover::property_value_type(parent, required).await;
    Some(
        SchemaTooltip::Content(SchemaTooltipContent {
            title: None,
            description: description.map(ToString::to_string),
            value_type: value_type.to_string(),
            constraints: constraints_markdown(constraints),
            schema: None,
        })
        .to_string(),
    )
}

fn constraints_markdown(constraints: ValueConstraints) -> Option<String> {
    (constraints != ValueConstraints::default()).then(|| constraints.to_string())
}
