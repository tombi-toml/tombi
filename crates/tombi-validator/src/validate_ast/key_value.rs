use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::Validate;

impl Validate for tombi_ast::KeyValue {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            // Get the value from the key-value pair
            if let Some(value) = self.value() {
                // Validate the value
                value
                    .validate(accessors, current_schema, schema_context, comment_context)
                    .await
            } else {
                Ok(())
            }
        }
        .boxed()
    }
}
