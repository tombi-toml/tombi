use futures::{future::BoxFuture, FutureExt};
use schema_store::{Accessor, FloatSchema, SchemaUrl, ValueSchema};

use crate::hover::{
    all_of::get_all_of_hover_content, any_of::get_any_of_hover_content,
    constraints::DataConstraints, default_value::DefaultValue, one_of::get_one_of_hover_content,
    GetHoverContent, HoverContent,
};

impl GetHoverContent for document_tree::Float {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: text::Position,
        keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        schema_url: Option<&'a SchemaUrl>,
        value_schema: Option<&'a ValueSchema>,
        definitions: Option<&'a schema_store::SchemaDefinitions>,
        schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            if let (Some(schema_url), Some(value_schema), Some(definitions)) =
                (schema_url, value_schema, definitions)
            {
                match value_schema {
                    ValueSchema::Float(float_schema) => float_schema
                        .get_hover_content(
                            position,
                            keys,
                            accessors,
                            Some(schema_url),
                            Some(value_schema),
                            Some(definitions),
                            schema_context,
                        )
                        .await
                        .map(|mut hover_content| {
                            hover_content.range = Some(self.range());
                            hover_content
                        }),
                    ValueSchema::OneOf(one_of_schema) => {
                        get_one_of_hover_content(
                            self,
                            position,
                            keys,
                            accessors,
                            schema_url,
                            one_of_schema,
                            definitions,
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
                            schema_url,
                            any_of_schema,
                            definitions,
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
                            schema_url,
                            all_of_schema,
                            definitions,
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
                    value_type: schema_store::ValueType::Float,
                    constraints: None,
                    schema_url: None,
                    range: Some(self.range()),
                })
            }
        }
        .boxed()
    }
}

impl GetHoverContent for FloatSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: text::Position,
        _keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        schema_url: Option<&'a SchemaUrl>,
        _value_schema: Option<&'a ValueSchema>,
        _definitions: Option<&'a schema_store::SchemaDefinitions>,
        _schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<HoverContent>> {
        async move {
            Some(HoverContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: schema_store::Accessors::new(accessors.to_vec()),
                value_type: schema_store::ValueType::Float,
                constraints: Some(DataConstraints {
                    default: self.default.map(DefaultValue::Float),
                    enumerate: self.enumerate.as_ref().map(|value| {
                        value
                            .iter()
                            .map(|value| DefaultValue::Float(*value))
                            .collect()
                    }),
                    minimum: self.minimum.map(DefaultValue::Float),
                    maximum: self.maximum.map(DefaultValue::Float),
                    exclusive_minimum: self.exclusive_minimum.map(DefaultValue::Float),
                    exclusive_maximum: self.exclusive_maximum.map(DefaultValue::Float),
                    multiple_of: self.multiple_of.map(DefaultValue::Float),
                    ..Default::default()
                }),
                schema_url: schema_url.cloned(),
                range: None,
            })
        }
        .boxed()
    }
}
