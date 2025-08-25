use tombi_comment_directive::CommentContext;
use tombi_schema_store::{Accessor, CurrentSchema, StringSchema, ValueSchema};

use crate::{
    hover::{
        all_of::get_all_of_hover_content,
        any_of::get_any_of_hover_content,
        comment::get_value_comment_directive_hover_info,
        constraints::{build_enumerate_values, ValueConstraints},
        display_value::DisplayValue,
        one_of::get_one_of_hover_content,
        GetHoverContent, HoverValueContent,
    },
    HoverContent,
};
use tombi_future::Boxable;

impl GetHoverContent for tombi_document_tree::String {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::String(string_schema) => {
                        if let Some(enumerate) = &string_schema.enumerate {
                            if !enumerate.iter().any(|x| x == self.value()) {
                                return None;
                            }
                        }

                        let mut hover_content = string_schema
                            .get_hover_content(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                comment_context,
                            )
                            .await;

                        if let Some(HoverContent::Value(hover_value_content)) =
                            hover_content.as_mut()
                        {
                            hover_value_content.range = Some(self.range());
                        }

                        hover_content
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        get_one_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        get_any_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        get_all_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    _ => None,
                }
            } else {
                for comment in self.leading_comments() {
                    if let Some(hover_content) =
                        get_value_comment_directive_hover_info(comment, position).await
                    {
                        return Some(hover_content);
                    }
                }
                Some(
                    HoverValueContent {
                        title: None,
                        description: None,
                        accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                        value_type: tombi_schema_store::ValueType::String,
                        constraints: None,
                        schema_uri: None,
                        range: Some(self.range()),
                    }
                    .into(),
                )
            }
        }
        .boxed()
    }
}

impl GetHoverContent for StringSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(
                HoverValueContent {
                    title: self.title.clone(),
                    description: self.description.clone(),
                    accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                    value_type: tombi_schema_store::ValueType::String,
                    constraints: Some(ValueConstraints {
                        enumerate: build_enumerate_values(
                            &self.const_value,
                            &self.enumerate,
                            |value| Some(DisplayValue::String(value.clone())),
                        ),
                        default: self
                            .default
                            .as_ref()
                            .map(|value| DisplayValue::String(value.clone())),
                        examples: self.examples.as_ref().map(|examples| {
                            examples
                                .iter()
                                .map(|example| DisplayValue::String(example.clone()))
                                .collect()
                        }),
                        min_length: self.min_length,
                        max_length: self.max_length,
                        format: self.format,
                        pattern: self.pattern.clone(),
                        ..Default::default()
                    }),
                    schema_uri: current_schema.map(|schema| schema.schema_uri.as_ref().clone()),
                    range: None,
                }
                .into(),
            )
        }
        .boxed()
    }
}
