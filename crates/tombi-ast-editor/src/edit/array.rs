use std::borrow::Cow;

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{ArrayCommonLintRules, ArrayFormatRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, ValueSchema};

use crate::rule::{array_comma_trailing_comment, array_values_order};

impl crate::Edit for tombi_ast::Array {
    fn edit<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        tracing::trace!("current_schema = {:?}", current_schema);

        async move {
            let mut changes = vec![];

            let comment_directive =
                get_comment_directive_content::<ArrayFormatRules, ArrayCommonLintRules>(
                    if let Some(key_value) = self
                        .syntax()
                        .parent()
                        .and_then(|parent| tombi_ast::KeyValue::cast(parent))
                    {
                        key_value
                            .comment_directives()
                            .chain(self.comment_directives())
                            .collect_vec()
                    } else {
                        self.comment_directives().collect_vec()
                    },
                );

            let mut use_item_schema = false;
            if let Some(current_schema) = current_schema {
                if let ValueSchema::Array(array_schema) = current_schema.value_schema.as_ref() {
                    if let Some(item_schema) = &array_schema.items {
                        if let Ok(Some(current_schema)) = item_schema
                            .write()
                            .await
                            .resolve(
                                Cow::Borrowed(&current_schema.schema_uri),
                                Cow::Borrowed(&current_schema.definitions),
                                schema_context.store,
                            )
                            .await
                            .inspect_err(|err| tracing::warn!("{err}"))
                        {
                            use_item_schema = true;
                            for (index, (value, comma)) in self.values_with_comma().enumerate() {
                                changes
                                    .extend(array_comma_trailing_comment(&value, comma.as_ref()));
                                changes.extend(
                                    value
                                        .edit(
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(Accessor::Index(index)))
                                                .collect_vec(),
                                            source_path,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await,
                                );
                            }
                        }
                    }
                }
            }
            if !use_item_schema {
                for (value, comma) in self.values_with_comma() {
                    changes.extend(array_comma_trailing_comment(&value, comma.as_ref()));
                    changes.extend(
                        value
                            .edit(accessors, source_path, None, schema_context)
                            .await,
                    );
                }
            }

            changes.extend(
                array_values_order(
                    self.values_with_comma().collect_vec(),
                    current_schema,
                    schema_context,
                    comment_directive,
                )
                .await,
            );

            changes
        }
        .boxed()
    }
}
