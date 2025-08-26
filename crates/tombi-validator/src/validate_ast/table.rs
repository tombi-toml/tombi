use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::validate_ast::Validate;

impl Validate for tombi_ast::Table {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = Vec::new();

            // Validate all key-value pairs in the table
            for key_value in self.key_values() {
                if let Err(mut errs) = key_value
                    .validate(accessors, current_schema, schema_context, comment_context)
                    .await
                {
                    diagnostics.append(&mut errs);
                }
            }

            if diagnostics.is_empty() {
                Ok(())
            } else {
                Err(diagnostics)
            }
        }
        .boxed()
    }
}
