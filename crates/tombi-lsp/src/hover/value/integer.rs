use tombi_schema_store::{Accessor, CurrentSchema, IntegerSchema, ValueSchema};

use crate::hover::{
    all_of::get_all_of_hover_content,
    any_of::get_any_of_hover_content,
    constraints::{build_enumerate_values, ValueConstraints},
    display_value::DisplayValue,
    one_of::get_one_of_hover_content,
    GetHoverContent, HoverValueContent,
};
use tombi_future::Boxable;

impl GetHoverContent for tombi_document_tree::Integer {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverValueContent>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Integer(integer_schema) => {
                        if let Some(enumerate) = &integer_schema.enumerate {
                            if !enumerate.contains(&self.value()) {
                                return None;
                            }
                        }

                        integer_schema
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
                Some(HoverValueContent {
                    title: None,
                    description: None,
                    accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                    value_type: tombi_schema_store::ValueType::Integer,
                    constraints: None,
                    schema_url: None,
                    range: Some(self.range()),
                })
            }
        }
        .boxed()
    }
}

impl GetHoverContent for IntegerSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverValueContent>> {
        async move {
            Some(HoverValueContent {
                title: self.title.clone(),
                description: self.description.clone(),
                accessors: tombi_schema_store::Accessors::new(accessors.to_vec()),
                value_type: tombi_schema_store::ValueType::Integer,
                constraints: Some(ValueConstraints {
                    enumerate: build_enumerate_values(
                        &self.const_value,
                        &self.enumerate,
                        |value| Some(DisplayValue::Integer(*value)),
                    ),
                    default: self.default.map(DisplayValue::Integer),
                    examples: self.examples.as_ref().map(|examples| {
                        examples
                            .iter()
                            .map(|example| DisplayValue::Integer(*example))
                            .collect()
                    }),
                    minimum: self.minimum.map(DisplayValue::Integer),
                    maximum: self.maximum.map(DisplayValue::Integer),
                    exclusive_minimum: self.exclusive_minimum.map(DisplayValue::Integer),
                    exclusive_maximum: self.exclusive_maximum.map(DisplayValue::Integer),
                    multiple_of: self.multiple_of.map(DisplayValue::Integer),
                    ..Default::default()
                }),
                schema_url: current_schema.map(|schema| schema.schema_url.as_ref().clone()),
                range: None,
            })
        }
        .boxed()
    }
}
