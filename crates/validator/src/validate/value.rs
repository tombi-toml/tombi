use futures::{future::BoxFuture, FutureExt};

use super::Validate;

impl Validate for document_tree::Value {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [schema_store::SchemaAccessor],
        current_schema: Option<&'a schema_store::CurrentSchema<'a>>,
        schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<diagnostic::Diagnostic>>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("current_schema = {:?}", current_schema);

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
