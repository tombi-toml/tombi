use tombi_document_tree::TableKind;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

impl crate::Edit for tombi_ast::Value {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            match (self, node) {
                (tombi_ast::Value::Array(array), tombi_document_tree::Value::Array(_)) => {
                    array
                        .edit(node, accessors, source_path, current_schema, schema_context)
                        .await
                }
                (
                    tombi_ast::Value::InlineTable(inline_table),
                    tombi_document_tree::Value::Table(table),
                ) if matches!(table.kind(), TableKind::InlineTable { .. }) => {
                    inline_table
                        .edit(node, accessors, source_path, current_schema, schema_context)
                        .await
                }
                _ => Vec::with_capacity(0),
            }
        }
        .boxed()
    }
}
