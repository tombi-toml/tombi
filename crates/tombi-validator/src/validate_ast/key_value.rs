use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};

use crate::validate_ast::Validate;

impl Validate for tombi_ast::KeyValue {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(keys) = self.keys() else {
                return Ok(());
            };
            let Some(value) = self.value() else {
                return Ok(());
            };

            let keys = keys.keys().collect_vec();

            (keys.as_slice(), &value)
                .validate(accessors, current_schema, schema_context, comment_context)
                .await
        }
        .boxed()
    }
}
