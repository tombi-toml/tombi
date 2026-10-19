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

use super::{GetTypeDefinition, TypeDefinition, schema_type_definition};
use tombi_future::Boxable;

impl GetTypeDefinition for tombi_document_tree_syntax::Value<'_> {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        cursor: crate::CursorPosition<'a>,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        async move {
            if let Some(Ok(current_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .get_type_definition(
                        cursor,
                        keys,
                        accessors,
                        Some(&current_schema),
                        schema_context,
                    )
                    .await;
            }

            let projected_schema = current_schema.and_then(|schema| {
                crate::schema_resolver::project_schema_for_presentation(
                    self,
                    schema,
                    schema_context,
                )
            });
            let current_schema = match &projected_schema {
                Some(projected_schema) => projected_schema.as_ref(),
                None => current_schema,
            };

            let type_definition = match self {
                Self::Boolean(boolean) => {
                    boolean
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Integer(integer) => {
                    integer
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Float(float) => {
                    float
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::String(string) => {
                    string
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::OffsetDateTime(offset_date_time) => {
                    offset_date_time
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDateTime(local_date_time) => {
                    local_date_time
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDate(local_date) => {
                    local_date
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalTime(local_time) => {
                    local_time
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Array(array) => {
                    array
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Table(table) => {
                    table
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Incomplete { .. } => match current_schema {
                    Some(current_schema) => {
                        current_schema
                            .schema_view
                            .get_type_definition(
                                cursor,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await
                    }
                    None => Vec::new(),
                },
            };

            if !type_definition.is_empty() {
                return type_definition;
            }

            if let Some(current_schema) = current_schema
                && matches!(
                    current_schema.schema_view.as_ref(),
                    tombi_schema_store::SchemaView::Anything(_)
                )
            {
                return current_schema
                    .schema_view
                    .get_type_definition(
                        cursor,
                        keys,
                        accessors,
                        Some(current_schema),
                        schema_context,
                    )
                    .await;
            }

            Vec::new()
        }
        .boxed()
    }
}

impl GetTypeDefinition for tombi_schema_store::SchemaView {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        cursor: crate::CursorPosition<'a>,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        async move {
            match self {
                Self::Boolean(boolean_schema) => {
                    boolean_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Integer(integer_schema) => {
                    integer_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Float(float_schema) => {
                    float_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::String(string_schema) => {
                    string_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::OffsetDateTime(offset_date_time_schema) => {
                    offset_date_time_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDateTime(local_date_time_schema) => {
                    local_date_time_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalDate(local_date_schema) => {
                    local_date_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::LocalTime(local_time_schema) => {
                    local_time_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Array(array_schema) => {
                    array_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Table(table_schema) => {
                    table_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::OneOf(one_of_schema) => {
                    one_of_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::AnyOf(any_of_schema) => {
                    any_of_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::AllOf(all_of_schema) => {
                    all_of_schema
                        .get_type_definition(
                            cursor,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                        )
                        .await
                }
                Self::Anything(schema) => current_schema.map_or_else(Vec::new, |current_schema| {
                    vec![schema_type_definition(
                        current_schema,
                        accessors,
                        schema.span,
                    )]
                }),
                Self::Nothing(_) | Self::Null => Vec::new(),
            }
        }
        .boxed()
    }
}
