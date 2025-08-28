use tombi_future::{BoxFuture, Boxable};

impl crate::Edit for tombi_ast::Value {
    fn edit<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            match self {
                tombi_ast::Value::Array(array) => {
                    array
                        .edit(accessors, source_path, current_schema, schema_context)
                        .await
                }
                tombi_ast::Value::InlineTable(inline_table) => {
                    inline_table
                        .edit(accessors, source_path, current_schema, schema_context)
                        .await
                }
                _ => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}
