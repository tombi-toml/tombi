use crate::header_accessor::GetHeaderAccessors;
use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::validate_ast::Validate;

impl Validate for tombi_ast::ArrayOfTable {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(header_accessors) = self.get_header_accessor() else {
                return Ok(());
            };

            let mut total_diagnostics = vec![];

            for key_value in self.key_values() {
                if let Err(mut diagnostics) = (header_accessors.as_slice(), &key_value)
                    .validate(accessors, current_schema, schema_context, comment_context)
                    .await
                {
                    total_diagnostics.append(&mut diagnostics);
                }
            }

            if total_diagnostics.is_empty() {
                Ok(())
            } else {
                Err(total_diagnostics)
            }
        }
        .boxed()
    }
}
