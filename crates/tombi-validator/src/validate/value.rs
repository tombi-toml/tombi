use tombi_future::{BoxFuture, Boxable};

use super::Validate;

impl Validate for tombi_document_tree::Value {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            match self {
                Self::Boolean(boolean) => {
                    boolean
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::Integer(integer) => {
                    integer
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::Float(float) => {
                    float
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::String(string) => {
                    string
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::OffsetDateTime(offset_date_time) => {
                    offset_date_time
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::LocalDateTime(local_date_time) => {
                    local_date_time
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::LocalDate(local_date) => {
                    local_date
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::LocalTime(local_time) => {
                    local_time
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::Array(array) => {
                    array
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::Table(table) => {
                    table
                        .validate(accessors, current_schema, schema_context)
                        .await
                }
                Self::Incomplete { .. } => Ok(()),
            }
        }
        .boxed()
    }
}
