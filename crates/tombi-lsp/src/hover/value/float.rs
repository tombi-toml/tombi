use tombi_comment_directive::value::{FloatCommonFormatRules, FloatCommonLintRules};
use tombi_schema_store::{Accessor, CurrentSchema, FloatSchema, SchemaView};

use crate::{
    HoverContent,
    comment_directive::get_key_table_value_comment_directive_content_and_schema_uri,
    hover::{
        GetHoverContent, HoverValueContent,
        all_of::get_all_of_hover_content,
        any_of::get_any_of_hover_content,
        comment::get_value_comment_directive_hover_content,
        constraints::{ValueConstraints, build_enum_values},
        display_value::DisplayValue,
        merge_adjacent_hover_content,
        one_of::get_one_of_hover_content,
    },
};
use tombi_future::Boxable;

impl GetHoverContent for tombi_document_tree_syntax::Float {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        offset: tombi_text::Offset,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_key_table_value_comment_directive_content_and_schema_uri::<
                    FloatCommonFormatRules,
                    FloatCommonLintRules,
                >(self.comment_directives(), offset, accessors)
                && let Some(hover_content) =
                    get_value_comment_directive_hover_content(comment_directive_context, schema_uri)
                        .await
            {
                return Some(hover_content);
            }

            if let Some(current_schema) = current_schema {
                match current_schema.schema_view.as_ref() {
                    SchemaView::Float(float_schema) => {
                        if let Some(r#enum) = &float_schema.r#enum
                            && !r#enum.contains(&self.value())
                        {
                            return None;
                        }

                        let mut hover_content = float_schema
                            .get_hover_content(
                                offset,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;

                        if let Some(HoverContent::Value(hover_value_content)) =
                            hover_content.as_mut()
                        {
                            hover_value_content.span = Some(self.span());
                        }

                        merge_adjacent_hover_content(
                            self,
                            offset,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            hover_content,
                            float_schema.one_of.as_deref(),
                            float_schema.any_of.as_deref(),
                            float_schema.all_of.as_deref(),
                        )
                        .await
                    }
                    SchemaView::OneOf(one_of_schema) => {
                        get_one_of_hover_content(
                            self,
                            offset,
                            keys,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await
                    }
                    SchemaView::AnyOf(any_of_schema) => {
                        get_any_of_hover_content(
                            self,
                            offset,
                            keys,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await
                    }
                    SchemaView::AllOf(all_of_schema) => {
                        get_all_of_hover_content(
                            self,
                            offset,
                            keys,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await
                    }
                    _ => None,
                }
            } else {
                Some(HoverContent::Value(HoverValueContent {
                    title: None,
                    description: None,
                    accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
                    value_type: tombi_schema_store::ValueType::Float,
                    constraints: None,
                    schema_document_uri: None,
                    span: Some(self.span()),
                    schema_tooltip: None,
                }))
            }
        }
        .boxed()
    }
}

impl GetHoverContent for FloatSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Offset,
        _keys: &'a [tombi_document_tree_syntax::Key<'_>],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(HoverContent::Value(HoverValueContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
                value_type: tombi_schema_store::ValueType::Float,
                constraints: Some(ValueConstraints {
                    r#enum: build_enum_values(&self.const_value, &self.r#enum, |value| {
                        Some(DisplayValue::Float(*value))
                    }),
                    default: self.default.map(DisplayValue::Float),
                    examples: self.examples.as_ref().map(|examples| {
                        examples
                            .iter()
                            .map(|example| DisplayValue::Float(*example))
                            .collect()
                    }),
                    minimum: self.minimum.map(DisplayValue::Float),
                    maximum: self.maximum.map(DisplayValue::Float),
                    exclusive_minimum: self.exclusive_minimum.map(DisplayValue::Float),
                    exclusive_maximum: self.exclusive_maximum.map(DisplayValue::Float),
                    multiple_of: self.multiple_of.map(DisplayValue::Float),
                    ..Default::default()
                }),
                schema_document_uri: super::super::current_schema_link_uri(current_schema),
                span: None,
                schema_tooltip: None,
            }))
        }
        .boxed()
    }
}
