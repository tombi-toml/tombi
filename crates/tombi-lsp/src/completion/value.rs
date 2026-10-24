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

use crate::hover::{
    constraints::{ValueConstraints, build_enum_values},
    display_value::DisplayValue,
};
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
    Accessor, AllOfSchema, AnyOfSchema, AnythingSchema, ArraySchema, BooleanSchema, CurrentSchema,
    FloatSchema, IntegerSchema, LocalDateSchema, LocalDateTimeSchema, LocalTimeSchema,
    OffsetDateTimeSchema, OneOfSchema, SchemaDefinitions, SchemaStore, SchemaUri, SchemaView,
    StringSchema, TableSchema,
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
                CompletionHint::InTableHeader
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
                    schema_tooltip_content(self, None, description.map(ToString::to_string))
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
    title: Option<String>,
    description: Option<String>,
) -> SchemaTooltip {
    let (r#enum, default) = default_and_enum(schema_view);
    SchemaTooltip::Content(SchemaTooltipContent {
        title,
        description,
        value_type: schema_view.value_type().await.to_string(),
        constraints: constraints_markdown(r#enum, default),
        schema: None,
    })
}

/// Like hover, a composite schema's `default` applies to its branches that define none.
pub(super) fn with_inherited_default(
    documentation: Option<String>,
    parent: &SchemaView,
    candidate: &SchemaView,
) -> Option<String> {
    if !matches!(
        parent,
        SchemaView::OneOf(_) | SchemaView::AnyOf(_) | SchemaView::AllOf(_)
    ) || default_and_enum(candidate).1.is_some()
    {
        return documentation;
    }
    let Some(constraints) = constraints_markdown(None, default_and_enum(parent).1) else {
        return documentation;
    };
    match documentation {
        Some(documentation) => Some(format!("{}\n\n{constraints}", documentation.trim_end())),
        None => Some(constraints),
    }
}

fn constraints_markdown(
    r#enum: Option<Vec<DisplayValue>>,
    default: Option<DisplayValue>,
) -> Option<String> {
    let constraints = ValueConstraints {
        r#enum,
        default,
        ..Default::default()
    };
    (constraints != ValueConstraints::default()).then(|| constraints.to_string())
}

fn default_and_enum(schema_view: &SchemaView) -> (Option<Vec<DisplayValue>>, Option<DisplayValue>) {
    let try_from_json = |value: &tombi_json::Value| DisplayValue::try_from(value).ok();
    let try_from_object = |value: &tombi_json::Object| DisplayValue::try_from(value).ok();

    match schema_view {
        SchemaView::Boolean(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                Some(DisplayValue::Boolean(*value))
            }),
            schema.default.map(DisplayValue::Boolean),
        ),
        SchemaView::Integer(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                Some(DisplayValue::Integer(*value))
            }),
            schema.default.map(DisplayValue::Integer),
        ),
        SchemaView::Float(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                Some(DisplayValue::Float(*value))
            }),
            schema.default.map(DisplayValue::Float),
        ),
        SchemaView::String(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                Some(DisplayValue::String(value.clone()))
            }),
            schema
                .default
                .as_ref()
                .map(|value| DisplayValue::String(value.clone())),
        ),
        SchemaView::OffsetDateTime(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                DisplayValue::try_new_offset_date_time(value).ok()
            }),
            schema
                .default
                .as_ref()
                .and_then(|value| DisplayValue::try_new_offset_date_time(value).ok()),
        ),
        SchemaView::LocalDateTime(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                DisplayValue::try_new_local_date_time(value).ok()
            }),
            schema
                .default
                .as_ref()
                .and_then(|value| DisplayValue::try_new_local_date_time(value).ok()),
        ),
        SchemaView::LocalDate(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                DisplayValue::try_new_local_date(value).ok()
            }),
            schema
                .default
                .as_ref()
                .and_then(|value| DisplayValue::try_new_local_date(value).ok()),
        ),
        SchemaView::LocalTime(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, |value| {
                DisplayValue::try_new_local_time(value).ok()
            }),
            schema
                .default
                .as_ref()
                .and_then(|value| DisplayValue::try_new_local_time(value).ok()),
        ),
        SchemaView::Array(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, try_from_json),
            schema.default.as_ref().and_then(try_from_json),
        ),
        SchemaView::Table(schema) => (
            build_enum_values(&schema.const_value, &schema.r#enum, try_from_object),
            schema.default.as_ref().and_then(try_from_object),
        ),
        SchemaView::OneOf(OneOfSchema { default, .. })
        | SchemaView::AnyOf(AnyOfSchema { default, .. })
        | SchemaView::AllOf(AllOfSchema { default, .. }) => {
            (None, default.as_ref().and_then(try_from_json))
        }
        SchemaView::Anything(_) | SchemaView::Nothing(_) | SchemaView::Null => (None, None),
    }
}
