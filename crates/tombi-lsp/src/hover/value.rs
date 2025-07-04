mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;

use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, ValueSchema};

use super::{GetHoverContent, HoverContent};

impl GetHoverContent for tombi_document_tree::Value {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            match self {
                Self::Boolean(boolean) => {
                    boolean
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Integer(integer) => {
                    integer
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Float(float) => {
                    float
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::String(string) => {
                    string
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::OffsetDateTime(offset_date_time) => {
                    offset_date_time
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDateTime(local_date_time) => {
                    local_date_time
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDate(local_date) => {
                    local_date
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalTime(local_time) => {
                    local_time
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Array(array) => {
                    array
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Table(table) => {
                    table
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Incomplete { range } => match current_schema {
                    Some(current_schema) => current_schema
                        .value_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                        )
                        .await
                        .map(|mut hover_content| {
                            hover_content.range = Some(*range);
                            hover_content
                        }),
                    None => None,
                },
            }
        }
        .boxed()
    }
}

impl GetHoverContent for ValueSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            match self {
                Self::Boolean(boolean_schema) => {
                    boolean_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Integer(integer_schema) => {
                    integer_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Float(float_schema) => {
                    float_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::String(string_schema) => {
                    string_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::OffsetDateTime(offset_date_time_schema) => {
                    offset_date_time_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDateTime(local_date_time_schema) => {
                    local_date_time_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDate(local_date_schema) => {
                    local_date_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalTime(local_time_schema) => {
                    local_time_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Array(array_schema) => {
                    array_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Table(table_schema) => {
                    table_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::OneOf(one_of_schema) => {
                    one_of_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::AnyOf(any_of_schema) => {
                    any_of_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::AllOf(all_of_schema) => {
                    all_of_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Null => None,
            }
        }
        .boxed()
    }
}
