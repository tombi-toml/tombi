use tombi_document_tree_syntax::TableKind;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

impl crate::editor::Edit for tombi_ast_syntax::Value {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree_syntax::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::editor::Change>> {
        async move {
            log::trace!("node = {:?}", node);
            log::trace!("accessors = {:?}", accessors);
            log::trace!("current_schema = {:?}", current_schema);

            match (self, node) {
                (
                    tombi_ast_syntax::Value::Array(array),
                    tombi_document_tree_syntax::Value::Array(_),
                ) => {
                    array
                        .edit(node, accessors, source_path, current_schema, schema_context)
                        .await
                }
                (
                    tombi_ast_syntax::Value::InlineTable(inline_table),
                    tombi_document_tree_syntax::Value::Table(table),
                ) if matches!(table.kind(), TableKind::InlineTable { .. }) => {
                    inline_table
                        .edit(node, accessors, source_path, current_schema, schema_context)
                        .await
                }
                _ => Vec::new(),
            }
        }
        .boxed()
    }
}
