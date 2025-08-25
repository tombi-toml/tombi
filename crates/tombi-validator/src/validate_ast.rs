mod all_of;
mod any_of;
mod array;
mod array_of_table;
mod boolean;
mod date_time;
mod float;
mod inline_table;
mod integer;
mod key_value;
mod one_of;
mod root;
mod string;
mod table;
mod value;

use crate::Validate;
use float::validate_float_schema;
use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;

pub fn validate_ast<'a: 'b, 'b>(
    root: &'a tombi_ast::Root,
    source_schema: Option<&'a tombi_schema_store::SourceSchema>,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
    async move {
        let current_schema = source_schema.as_ref().and_then(|source_schema| {
            source_schema.root_schema.as_ref().and_then(|root_schema| {
                root_schema.value_schema.as_ref().map(|value_schema| {
                    tombi_schema_store::CurrentSchema {
                        value_schema: std::borrow::Cow::Borrowed(value_schema),
                        schema_uri: std::borrow::Cow::Borrowed(&root_schema.schema_uri),
                        definitions: std::borrow::Cow::Borrowed(&root_schema.definitions),
                    }
                })
            })
        });

        root.validate(
            &[],
            current_schema.as_ref(),
            schema_context,
            &CommentContext::default(),
        )
        .await?;

        Ok(())
    }
    .boxed()
}

trait ValueImpl {
    fn value_type(&self) -> ValueType;

    fn range(&self) -> tombi_text::Range;
}
