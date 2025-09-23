use std::sync::Arc;

use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::Accessor;

use crate::edit::edit_recursive;

impl crate::Edit for tombi_ast::KeyValue {
    fn edit<'a: 'b, 'b>(
        &'a self,
        node: &'a tombi_document_tree::Value,
        accessors: &'a [Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        async move {
            tracing::trace!("node = {:?}", node);
            tracing::trace!("accessors = {:?}", accessors);
            tracing::trace!("current_schema = {:?}", current_schema);

            let Some(key_accessors) = self.get_accessors(schema_context.toml_version) else {
                return Vec::with_capacity(0);
            };

            edit_recursive(
                node,
                |node, accessors, current_schema| {
                    async move {
                        tracing::trace!("node = {:?}", node);
                        tracing::trace!("accessors = {:?}", accessors);
                        tracing::trace!("current_schema = {:?}", current_schema);

                        if let Some(value) = self.value() {
                            value
                                .edit(
                                    node,
                                    &accessors,
                                    source_path,
                                    current_schema.as_ref(),
                                    schema_context,
                                )
                                .await
                        } else {
                            Vec::with_capacity(0)
                        }
                    }
                    .boxed()
                },
                &key_accessors,
                Arc::from(accessors.to_vec()),
                current_schema.cloned(),
                schema_context,
            )
            .await
        }
        .boxed()
    }
}
