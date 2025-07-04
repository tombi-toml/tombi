use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, ValueSchema};

use super::{
    value::{
        find_all_of_completion_items, find_any_of_completion_items, find_one_of_completion_items,
    },
    CompletionContent, CompletionHint, FindCompletionContents,
};

/// A tag data that indicates that only schema information is used for completion.
pub struct SchemaCompletion;

impl FindCompletionContents for SchemaCompletion {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>> {
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("current_schema = {:?}", current_schema);
        tracing::trace!("completion_hint = {:?}", completion_hint);

        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("SchemaCompletion::find_completion_contents called without a schema");
            };

            match current_schema.value_schema.as_ref() {
                ValueSchema::Boolean(boolean_schema) => {
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
                ValueSchema::Integer(integer_schema) => {
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
                ValueSchema::Float(float_schema) => {
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
                ValueSchema::String(string_schema) => {
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
                ValueSchema::OffsetDateTime(offset_date_time_schema) => {
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
                ValueSchema::LocalDateTime(local_date_time_schema) => {
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
                ValueSchema::LocalDate(local_date_schema) => {
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
                ValueSchema::LocalTime(local_time_schema) => {
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
                ValueSchema::Array(array_schema) => {
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
                ValueSchema::Table(table_schema) => {
                    table_schema
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
                ValueSchema::OneOf(one_of_schema) => {
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
                ValueSchema::AnyOf(any_of_schema) => {
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
                ValueSchema::AllOf(all_of_schema) => {
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
                ValueSchema::Null => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}

impl tombi_validator::Validate for SchemaCompletion {
    fn validate<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::SchemaAccessor],
        _current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move { Ok(()) }.boxed()
    }
}
