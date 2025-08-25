use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::Validate;

impl Validate for tombi_ast::Array {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = Vec::new();

            // Validate all values in the array
            for value in self.values() {
                if let Err(mut errs) = value
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
