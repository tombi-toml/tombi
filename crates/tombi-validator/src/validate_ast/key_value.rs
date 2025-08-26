use std::borrow::Cow;

use itertools::Itertools;
use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{CurrentSchema, DocumentSchema, ValueType};

use crate::{
    header_accessor::HeaderAccessor,
    validate_ast::{
        all_of::validate_all_of,
        any_of::validate_any_of,
        one_of::validate_one_of,
        table::{validate_table_schema, validate_table_without_schema},
        type_mismatch, Validate, ValueImpl,
    },
};

impl Validate for tombi_ast::KeyValue {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(keys) = self.keys() else {
                return Ok(());
            };
            let Some(value) = self.value() else {
                return Ok(());
            };

            let keys = keys.keys().collect_vec();

            (keys.as_slice(), &value)
                .validate(accessors, current_schema, schema_context, comment_context)
                .await
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::KeyValue {
    fn value_type(&self) -> tombi_schema_store::ValueType {
        tombi_schema_store::ValueType::Table
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl<T> Validate for (&[tombi_ast::Key], &T)
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let (keys, value) = *self;
            let header_accessors = keys
                .iter()
                .map(|key| HeaderAccessor::Key(key.clone()))
                .collect_vec();

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
                    tombi_schema_store::ValueSchema::Table(table_schema) => {
                        validate_table_schema(
                            &header_accessors,
                            value,
                            accessors,
                            table_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
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
                    tombi_schema_store::ValueSchema::AnyOf(any_of_schema) => {
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
                    tombi_schema_store::ValueSchema::AllOf(all_of_schema) => {
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
                    tombi_schema_store::ValueSchema::Null => return Ok(()),
                    value_schema => {
                        type_mismatch(ValueType::Table, value.range(), value_schema).await
                    }
                }
            } else {
                validate_table_without_schema(
                    &header_accessors,
                    value,
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

impl<T> ValueImpl for (&[tombi_ast::Key], &T)
where
    T: ValueImpl,
{
    fn value_type(&self) -> ValueType {
        ValueType::Table
    }

    fn range(&self) -> tombi_text::Range {
        let (keys, value) = *self;
        if let Some(key) = keys.first() {
            key.range() + value.range()
        } else {
            value.range()
        }
    }
}

impl<T> Validate for (&[HeaderAccessor], &T)
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let (header_accessors, value) = *self;

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
                    tombi_schema_store::ValueSchema::Table(table_schema) => {
                        validate_table_schema(
                            header_accessors,
                            value,
                            accessors,
                            table_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
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
                    tombi_schema_store::ValueSchema::AnyOf(any_of_schema) => {
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
                    tombi_schema_store::ValueSchema::AllOf(all_of_schema) => {
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
                    tombi_schema_store::ValueSchema::Null => return Ok(()),
                    value_schema => {
                        let (actual, range) = match header_accessors.first() {
                            Some(HeaderAccessor::Key(key)) => (ValueType::Table, key.range()),
                            Some(HeaderAccessor::Index { range, .. }) => (ValueType::Array, *range),
                            None => (value.value_type(), value.range()),
                        };
                        type_mismatch(actual, range, value_schema).await
                    }
                }
            } else {
                validate_table_without_schema(
                    header_accessors,
                    value,
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

impl<T> ValueImpl for (&[HeaderAccessor], &T)
where
    T: ValueImpl,
{
    fn value_type(&self) -> ValueType {
        let (header_accessors, value) = *self;
        if let Some(header_accessor) = header_accessors.first() {
            match header_accessor {
                HeaderAccessor::Key(_) => ValueType::Table,
                HeaderAccessor::Index { .. } => ValueType::Array,
            }
        } else {
            value.value_type()
        }
    }

    fn range(&self) -> tombi_text::Range {
        let (header_accessors, value) = *self;
        if let Some(header_accessor) = header_accessors.first() {
            header_accessor.range() + value.range()
        } else {
            value.range()
        }
    }
}
