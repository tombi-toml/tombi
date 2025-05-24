mod all_of;
mod any_of;
mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod one_of;
mod string;
mod table;
mod value;

use std::borrow::Cow;

use all_of::validate_all_of;
use any_of::validate_any_of;
use one_of::validate_one_of;
use tombi_schema_store::CurrentSchema;
use tombi_wasm_compat::box_future::{BoxFuture, Boxable};

pub trait Validate {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>;
}

pub fn validate<'a: 'b, 'b>(
    tree: tombi_document_tree::DocumentTree,
    source_schema: &'a tombi_schema_store::SourceSchema,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
    async move {
        let current_schema = source_schema.root_schema.as_ref().and_then(|root_schema| {
            root_schema
                .value_schema
                .as_ref()
                .map(|value_schema| CurrentSchema {
                    value_schema: Cow::Borrowed(value_schema),
                    schema_url: Cow::Borrowed(&root_schema.schema_url),
                    definitions: Cow::Borrowed(&root_schema.definitions),
                })
        });

        tree.validate(&[], current_schema.as_ref(), schema_context)
            .await?;

        Ok(())
    }
    .boxed()
}
