use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, CurrentSchema, StringSchema, ValueSchema};

use crate::hover::{
    all_of::get_all_of_hover_content, any_of::get_any_of_hover_content,
    constraints::DataConstraints, default_value::DefaultValue, one_of::get_one_of_hover_content,
    GetHoverContent, HoverContent,
};

impl GetHoverContent for document_tree::String {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: text::Position,
        keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::String(string_schema) => {
                        if let Some(enumerate) = &string_schema.enumerate {
                            if !enumerate.iter().any(|x| x == self.value()) {
                                return None;
                            }
                        }

                        string_schema
                            .get_hover_content(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await
                            .map(|mut hover_content| {
                                hover_content.range = Some(self.range());
                                hover_content
                            })
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        get_one_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            &current_schema.schema_url,
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
                            &current_schema.schema_url,
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
                            &current_schema.schema_url,
                            &current_schema.definitions,
                            schema_context,
                        )
                        .await
                    }
                    _ => None,
                }
            } else {
                Some(HoverContent {
                    title: None,
                    description: None,
                    accessors: schema_store::Accessors::new(accessors.to_vec()),
                    value_type: schema_store::ValueType::String,
                    constraints: None,
                    schema_url: None,
                    range: Some(self.range()),
                })
            }
        }
        .boxed()
    }
}

impl GetHoverContent for StringSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: text::Position,
        _keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(HoverContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: schema_store::Accessors::new(accessors.to_vec()),
                value_type: schema_store::ValueType::String,
                constraints: Some(DataConstraints {
                    default: self
                        .default
                        .as_ref()
                        .map(|value| DefaultValue::String(value.clone())),
                    enumerate: self.enumerate.as_ref().map(|value| {
                        value
                            .iter()
                            .map(|value| DefaultValue::String(value.clone()))
                            .collect()
                    }),
                    min_length: self.min_length,
                    max_length: self.max_length,
                    pattern: self.pattern.clone(),
                    ..Default::default()
                }),
                schema_url: current_schema.map(|schema| schema.schema_url.as_ref().clone()),
                range: None,
            })
        }
        .boxed()
    }
}
