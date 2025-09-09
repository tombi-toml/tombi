use tombi_comment_directive::value::OffsetDateTimeCommonRules;
use tombi_schema_store::{Accessor, CurrentSchema, OffsetDateTimeSchema, ValueSchema};

use crate::{
    comment_directive::get_key_table_value_comment_directive_content_and_schema_uri,
    hover::{
        all_of::get_all_of_hover_content,
        any_of::get_any_of_hover_content,
        comment::get_value_comment_directive_hover_content,
        constraints::{build_enumerate_values, ValueConstraints},
        display_value::DisplayValue,
        one_of::get_one_of_hover_content,
        GetHoverContent, HoverValueContent,
    },
    HoverContent,
};
use tombi_future::Boxable;

impl GetHoverContent for tombi_document_tree::OffsetDateTime {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_key_table_value_comment_directive_content_and_schema_uri::<OffsetDateTimeCommonRules>(
                    self.comment_directives(),
                    position,
                    accessors,
                )
            {
                if let Some(hover_content) =
                    get_value_comment_directive_hover_content(comment_directive_context, schema_uri)
                        .await
                {
                    return Some(hover_content);
                }
            }

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::OffsetDateTime(offset_date_time_schema) => {
                        let mut hover_content = offset_date_time_schema
                            .get_hover_content(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
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
                    value_type: tombi_schema_store::ValueType::OffsetDateTime,
                    constraints: None,
                    schema_uri: None,
                    range: Some(self.range()),
                }))
            }
        }
        .boxed()
    }
}

impl GetHoverContent for OffsetDateTimeSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(HoverContent::Value(HoverValueContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
                value_type: tombi_schema_store::ValueType::OffsetDateTime,
                constraints: Some(ValueConstraints {
                    enumerate: build_enumerate_values(
                        &self.const_value,
                        &self.enumerate,
                        |value| DisplayValue::try_new_offset_date_time(value).ok(),
                    ),
                    default: self
                        .default
                        .as_ref()
                        .and_then(|value| DisplayValue::try_new_offset_date_time(value).ok()),
                    examples: self.examples.as_ref().map(|examples| {
                        examples
                            .iter()
                            .filter_map(|example| {
                                DisplayValue::try_new_offset_date_time(example).ok()
                            })
                            .collect()
                    }),
                    ..Default::default()
                }),
                schema_uri: current_schema.map(|schema| schema.schema_uri.as_ref().clone()),
                range: None,
            }))
        }
        .boxed()
    }
}
