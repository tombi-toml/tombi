use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaView};
use tombi_x_keyword::StringFormat;

use crate::schema_tooltip::{SchemaTooltip, SchemaTooltipContent};

use super::{
    CompletionContent, CompletionHint, FindCompletionContents,
    merge_adjacent_schema_completion_items, tombi_json_value_to_completion_enum_item,
    value::{
        find_all_of_completion_items, find_any_of_completion_items, find_one_of_completion_items,
    },
};

fn set_schema_link_uri(
    completion_items: &mut [CompletionContent],
    current_schema: &CurrentSchema<'_>,
) {
    let schema_uri = tombi_extension::get_schema_link_uri(
        current_schema.schema_uri.as_ref(),
        current_schema.schema_view.range().start,
    );
    for item in completion_items {
        if item.schema_uri.as_ref() == Some(current_schema.schema_uri.as_ref()) {
            item.schema_uri = Some(schema_uri.clone().into());
        }
    }
}

/// A tag data that indicates that only schema information is used for completion.
#[derive(Debug)]
pub struct SchemaCompletion;

#[derive(Debug, Clone, Copy)]
pub struct InstanceSchemaCompletion(pub tombi_schema_store::SchemaType);

impl FindCompletionContents for InstanceSchemaCompletion {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        async move {
            let Some(mut projected_schema) = current_schema.and_then(|schema| {
                schema.for_instance_type(self.0, schema_context.string_formats())
            }) else {
                return Vec::new();
            };
            projected_schema.semantic_schema = None;
            SchemaCompletion
                .find_completion_contents(
                    position,
                    keys,
                    accessors,
                    Some(&projected_schema),
                    schema_context,
                    completion_hint,
                )
                .await
        }
        .boxed()
    }
}

impl FindCompletionContents for SchemaCompletion {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        log::trace!("accessors = {:?}", accessors);
        log::trace!("keys = {:?}", keys);
        log::trace!("current_schema = {:?}", current_schema);
        log::trace!("completion_hint = {:?}", completion_hint);

        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("SchemaCompletion::find_completion_contents called without a schema");
            };

            let has_toml_datetime_format = current_schema
                .semantic_schema
                .as_deref()
                .and_then(|schema| schema.string_format())
                .and_then(|format| format.parse::<StringFormat>().ok())
                .is_some_and(|format| format.toml_date_time_type().is_some());
            if !has_toml_datetime_format
                && let Some(candidates) = current_schema
                    .semantic_schema
                    .as_deref()
                    .and_then(|schema| schema.finite_literal_candidates())
            {
                let detail = current_schema.schema_view.title().map(ToString::to_string);
                let documentation = current_schema
                    .schema_view
                    .description()
                    .map(ToString::to_string);
                let mut completion_items = candidates
                    .iter()
                    .filter_map(|value| {
                        tombi_json_value_to_completion_enum_item(
                            value,
                            position,
                            detail.clone(),
                            documentation.clone(),
                            Some(current_schema.schema_uri.as_ref()),
                            completion_hint,
                        )
                    })
                    .collect::<Vec<_>>();
                set_schema_link_uri(&mut completion_items, current_schema);
                return completion_items;
            }

            let projected_schema = current_schema.for_completion(schema_context.string_formats());
            let current_schema = projected_schema.as_ref().unwrap_or(current_schema);

            let mut completion_items = match current_schema.schema_view.as_ref() {
                SchemaView::Boolean(boolean_schema) => {
                    boolean_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::Integer(integer_schema) => {
                    integer_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::Float(float_schema) => {
                    float_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::String(string_schema) => {
                    string_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::OffsetDateTime(offset_date_time_schema) => {
                    offset_date_time_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::LocalDateTime(local_date_time_schema) => {
                    local_date_time_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::LocalDate(local_date_schema) => {
                    local_date_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::LocalTime(local_time_schema) => {
                    local_time_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::Array(array_schema) => {
                    array_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                SchemaView::Table(table_schema) => {
                    let base_completion_items = table_schema
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            completion_hint,
                        )
                        .await;
                    merge_adjacent_schema_completion_items(
                        position,
                        keys,
                        accessors,
                        Some(current_schema),
                        schema_context,
                        completion_hint,
                        base_completion_items,
                        table_schema.one_of.as_deref(),
                        table_schema.any_of.as_deref(),
                        table_schema.all_of.as_deref(),
                    )
                    .await
                }
                SchemaView::OneOf(one_of_schema) => {
                    find_one_of_completion_items(
                        self,
                        position,
                        keys,
                        accessors,
                        one_of_schema,
                        current_schema,
                        schema_context,
                        completion_hint,
                    )
                    .await
                }
                SchemaView::AnyOf(any_of_schema) => {
                    find_any_of_completion_items(
                        self,
                        position,
                        keys,
                        accessors,
                        any_of_schema,
                        current_schema,
                        schema_context,
                        completion_hint,
                    )
                    .await
                }
                SchemaView::AllOf(all_of_schema) => {
                    find_all_of_completion_items(
                        self,
                        position,
                        keys,
                        accessors,
                        all_of_schema,
                        current_schema,
                        schema_context,
                        completion_hint,
                    )
                    .await
                }
                SchemaView::Anything(_) | SchemaView::Nothing(_) | SchemaView::Null => Vec::new(),
            };

            let needs_documentation = |item: &CompletionContent| {
                item.documentation.is_none()
                    && matches!(
                        item.priority,
                        tombi_extension::CompletionContentPriority::TypeHint
                            | tombi_extension::CompletionContentPriority::TypeHintTrue
                            | tombi_extension::CompletionContentPriority::TypeHintFalse
                    )
            };

            if completion_items.iter().any(needs_documentation) {
                let documentation = SchemaTooltip::Content(SchemaTooltipContent {
                    title: current_schema.schema_view.title().map(ToString::to_string),
                    description: current_schema
                        .schema_view
                        .description()
                        .map(ToString::to_string),
                    value_type: current_schema.schema_view.value_type().await.to_string(),
                    constraints: None,
                    schema: None,
                })
                .to_string();

                for item in &mut completion_items {
                    if needs_documentation(item) {
                        item.documentation = Some(documentation.clone());
                    }
                }
            }

            set_schema_link_uri(&mut completion_items, current_schema);

            completion_items
        }
        .boxed()
    }
}

impl tombi_validator::Validate for SchemaCompletion {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::Accessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Result<tombi_validator::Valid, tombi_validator::Invalid>> {
        async move { Ok(tombi_validator::Valid::new()) }.boxed()
    }
}

impl tombi_validator::Validate for InstanceSchemaCompletion {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::Accessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Result<tombi_validator::Valid, tombi_validator::Invalid>> {
        async move { Ok(tombi_validator::Valid::new()) }.boxed()
    }
}
