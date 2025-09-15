use std::borrow::Cow;

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_comment_directive::value::{ArrayCommonFormatRules, ArrayCommonLintRules};
use tombi_comment_directive_serde::get_comment_directive_content;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{AllOfSchema, AnyOfSchema, OneOfSchema, ValueSchema};
use tombi_validator::Validate;

use crate::rule::{array_comma_trailing_comment, array_values_order};

impl crate::Edit for tombi_ast::Array {
    fn edit<'a: 'b, 'b>(
        &'a self,
        _accessors: &'a [tombi_schema_store::Accessor],
        source_path: Option<&'a std::path::Path>,
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    ) -> BoxFuture<'b, Vec<crate::Change>> {
        tracing::trace!("current_schema = {:?}", current_schema);

        async move {
            let mut changes = vec![];

            let mut use_item_schema = false;
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
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
                                for (value, comma) in self.values_with_comma() {
                                    changes.extend(array_comma_trailing_comment(
                                        &value,
                                        comma.as_ref(),
                                    ));
                                    changes.extend(
                                        value
                                            .edit(
                                                &[],
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
                    ValueSchema::AllOf(AllOfSchema { schemas, .. })
                    | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                    | ValueSchema::OneOf(OneOfSchema { schemas, .. }) => {
                        let value = &self
                            .clone()
                            .into_document_tree_and_errors(schema_context.toml_version)
                            .tree;

                        for referable_schema in schemas.write().await.iter_mut() {
                            if let Ok(Some(current_schema)) = referable_schema
                                .resolve(
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                                .inspect_err(|err| tracing::warn!("{err}"))
                            {
                                if value
                                    .validate(&[], Some(&current_schema), schema_context)
                                    .await
                                    .is_ok()
                                {
                                    return self
                                        .edit(
                                            &[],
                                            source_path,
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !use_item_schema {
                for (value, comma) in self.values_with_comma() {
                    changes.extend(array_comma_trailing_comment(&value, comma.as_ref()));
                    changes.extend(value.edit(&[], source_path, None, schema_context).await);
                }
            }

            let comment_directive =
                get_comment_directive_content::<ArrayCommonFormatRules, ArrayCommonLintRules>(
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
