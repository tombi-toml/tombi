use ahash::AHashSet;
use itertools::Itertools;
use tombi_ast::LiteralValue;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{ValueSchema, ValueType};

use crate::{
    header_accessor::HeaderAccessor,
    validate_ast::{
        all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of,
        table::validate_accessor_without_schema, type_mismatch, Validate, ValueImpl,
    },
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
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Array(array_schema) => {
                        let mut total_diagnostics = vec![];
                        if let Err(schema_diagnostics) = validate_array_schema(
                            &[],
                            self,
                            accessors,
                            array_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                        {
                            total_diagnostics.extend(schema_diagnostics);
                        }

                        let items_len = self.items().count();

                        if let Some(max_items) = array_schema.max_items {
                            if items_len > max_items {
                                crate::Error {
                                    kind: crate::ErrorKind::ArrayMaxItems {
                                        max_values: max_items,
                                        actual: items_len,
                                    },
                                    range: self.range(),
                                }
                                .set_diagnostics(&mut total_diagnostics);
                            }
                        }

                        if let Some(min_items) = array_schema.min_items {
                            if items_len < min_items {
                                crate::Error {
                                    kind: crate::ErrorKind::ArrayMinItems {
                                        min_values: min_items,
                                        actual: items_len,
                                    },
                                    range: self.range(),
                                }
                                .set_diagnostics(&mut total_diagnostics);
                            }
                        }
                        if array_schema.unique_items == Some(true) {
                            let literal_values = self
                                .items()
                                .filter_map(Option::<LiteralValue>::from)
                                .counts();

                            let duplicated_values = literal_values
                                .iter()
                                .filter_map(
                                    |(value, count)| if *count > 1 { Some(value) } else { None },
                                )
                                .collect::<AHashSet<_>>();

                            for item in self.items() {
                                let range = item.range();
                                if let Some(literal_value) = Option::<LiteralValue>::from(item) {
                                    if duplicated_values.contains(&literal_value) {
                                        crate::Error {
                                            kind: crate::ErrorKind::ArrayUniqueItems,
                                            range,
                                        }
                                        .set_diagnostics(&mut total_diagnostics);
                                    }
                                }
                            }
                        }
                        if total_diagnostics.is_empty() {
                            Ok(())
                        } else {
                            Err(total_diagnostics)
                        }
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
                    value_schema => type_mismatch(
                        value_schema.value_type().await,
                        ValueType::Array,
                        self.range(),
                    ),
                }
            } else {
                validate_accessor_without_schema(
                    &[],
                    self,
                    accessors,
                    schema_context,
                    comment_context,
                )
                .await
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

pub fn validate_array_schema<'a: 'b, 'b, T>(
    header_accessors: &'a [HeaderAccessor],
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    array_schema: &'a tombi_schema_store::ArraySchema,
    current_schema: &'a tombi_schema_store::CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        match header_accessors.first() {
            Some(HeaderAccessor::Index { index, .. }) => {
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
                        let new_accessors = accessors
                            .iter()
                            .cloned()
                            .chain(std::iter::once(tombi_schema_store::Accessor::Index(*index)))
                            .collect_vec();

                        if let Err(schema_diagnostics) = value
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

                if diagnostics.is_empty() {
                    Ok(())
                } else {
                    Err(diagnostics)
                }
            }
            None => {
                validate_accessor_without_schema(
                    header_accessors,
                    value,
                    accessors,
                    schema_context,
                    comment_context,
                )
                .await
            }
            Some(HeaderAccessor::Key(key)) => {
                type_mismatch(ValueType::Array, ValueType::Table, key.range())
            }
        }
    }
    .boxed()
}
