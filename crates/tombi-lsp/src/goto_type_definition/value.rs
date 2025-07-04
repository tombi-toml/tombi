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
use super::GetTypeDefinition;
use tombi_future::Boxable;

impl GetTypeDefinition for tombi_document_tree::Value {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<crate::goto_type_definition::TypeDefinition>> {
        async move {
            match self {
                Self::Boolean(boolean) => {
                    boolean
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
                            position,
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
                            .value_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await
                    }
                    None => None,
                },
            }
        }
        .boxed()
    }
}

impl GetTypeDefinition for tombi_schema_store::ValueSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<crate::goto_type_definition::TypeDefinition>> {
        async move {
            match self {
                Self::Boolean(boolean_schema) => {
                    boolean_schema
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
                        .get_type_definition(
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
