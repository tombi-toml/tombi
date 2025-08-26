use std::borrow::Cow;

use ahash::AHashSet;
use itertools::Itertools;
use tombi_ast::LiteralValue;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, DocumentSchema, ValueSchema, ValueType};

use crate::validate_ast::{
    all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of, type_mismatch,
    Validate, ValueImpl,
};

impl Validate for tombi_ast::Array {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            if let Some(sub_schema_uri) = schema_context
                .sub_schema_uri_map
                .and_then(|map| map.get(&accessors.into_iter().map(Into::into).collect_vec()))
            {
                if current_schema
                    .is_some_and(|current_schema| &*current_schema.schema_uri != sub_schema_uri)
                {
                    if let Ok(Some(DocumentSchema {
                        value_schema: Some(value_schema),
                        schema_uri,
                        definitions,
                        ..
                    })) = schema_context
                        .store
                        .try_get_document_schema(sub_schema_uri)
                        .await
                    {
                        return self
                            .validate(
                                accessors,
                                Some(&CurrentSchema {
                                    value_schema: Cow::Borrowed(&value_schema),
                                    schema_uri: Cow::Borrowed(&schema_uri),
                                    definitions: Cow::Borrowed(&definitions),
                                }),
                                schema_context,
                                comment_context,
                            )
                            .await;
                    }
                }
            }

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
                        validate_array_schema(
                            self,
                            array_schema,
                            accessors,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        validate_any_of(
                            self,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        validate_all_of(
                            self,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::Null => Ok(()),
                    value_schema => {
                        type_mismatch(ValueType::Array, self.range(), value_schema).await
                    }
                }
            } else {
                validate_array(self, accessors, schema_context, comment_context).await
            }
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::Array {
    fn value_type(&self) -> tombi_schema_store::ValueType {
        tombi_schema_store::ValueType::Array
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

async fn validate_array_schema<'a>(
    value: &'a tombi_ast::Array,
    array_schema: &tombi_schema_store::ArraySchema,
    accessors: &[tombi_schema_store::Accessor],
    current_schema: &'a tombi_schema_store::CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_context: &'a CommentContext<'a>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    if let Some(items) = &array_schema.items {
        let mut referable_schema = items.write().await;
        if let Ok(Some(current_schema)) = referable_schema
            .resolve(
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            .inspect_err(|err| tracing::warn!("{err}"))
        {
            for (index, item) in value.items().enumerate() {
                let new_accessors = accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                    .collect_vec();

                if let Err(schema_diagnostics) = item
                    .validate(
                        &new_accessors,
                        Some(&current_schema),
                        schema_context,
                        comment_context,
                    )
                    .await
                {
                    diagnostics.extend(schema_diagnostics);
                }
            }
        }
    }

    let items_len = value.items().count();

    if let Some(max_items) = array_schema.max_items {
        if items_len > max_items {
            crate::Error {
                kind: crate::ErrorKind::ArrayMaxItems {
                    max_values: max_items,
                    actual: items_len,
                },
                range: value.range(),
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(min_items) = array_schema.min_items {
        if items_len < min_items {
            crate::Error {
                kind: crate::ErrorKind::ArrayMinItems {
                    min_values: min_items,
                    actual: items_len,
                },
                range: value.range(),
            }
            .set_diagnostics(&mut diagnostics);
        }
    }
    if array_schema.unique_items == Some(true) {
        let literal_values = value
            .items()
            .filter_map(Option::<LiteralValue>::from)
            .counts();

        let duplicated_values = literal_values
            .iter()
            .filter_map(|(value, count)| if *count > 1 { Some(value) } else { None })
            .collect::<AHashSet<_>>();

        for item in value.items() {
            let range = item.range();
            if let Some(literal_value) = Option::<LiteralValue>::from(item) {
                if duplicated_values.contains(&literal_value) {
                    crate::Error {
                        kind: crate::ErrorKind::ArrayUniqueItems,
                        range,
                    }
                    .set_diagnostics(&mut diagnostics);
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_array<'a: 'b, 'b>(
    value: &'a tombi_ast::Array,
    accessors: &'a [tombi_schema_store::Accessor],
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
    async move {
        let mut diagnostics = vec![];
        for (index, value) in value.items().enumerate() {
            if let Err(value_diagnostics) = value
                .validate(
                    &accessors
                        .iter()
                        .cloned()
                        .chain(std::iter::once(tombi_schema_store::Accessor::Index(index)))
                        .collect_vec(),
                    None,
                    schema_context,
                    comment_context,
                )
                .await
            {
                diagnostics.extend(value_diagnostics);
            }
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
    .boxed()
}
