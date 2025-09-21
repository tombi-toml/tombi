use std::sync::Arc;

use tombi_future::BoxFuture;
use tombi_schema_store::Accessor;

mod array;
mod array_of_table;
mod inline_table;
mod key_value;
mod root;
mod table;
mod value;

pub trait Edit {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>>;
}

#[allow(dead_code)]
pub trait EditRecursive {
    fn edit_recursive<'a: 'b, 'b>(
        &'a self,
        edit_fn: impl FnOnce(
                &'a tombi_document_tree::Value,
                Arc<[Accessor]>,
                Option<tombi_schema_store::CurrentSchema<'a>>,
            ) -> BoxFuture<'b, Vec<crate::Change>>
            + std::marker::Send
            + 'b,
        key_accessors: &'a [Accessor],
        accessors: Arc<[Accessor]>,
        current_schema: Option<tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>>;
}
