use itertools::Itertools;
use tombi_future::Boxable;
use tombi_schema_store::ValueSchema;

use crate::goto_type_definition::{
    all_of::get_all_of_type_definition, any_of::get_any_of_type_definition,
    one_of::get_one_of_type_definition, GetTypeDefinition, TypeDefinition,
};

impl GetTypeDefinition for tombi_document_tree::String {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<crate::goto_type_definition::TypeDefinition>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::String(string_schema) => {
                        string_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        get_one_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            current_schema.schema_url.as_ref(),
                            current_schema.definitions.as_ref(),
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        get_any_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            current_schema.schema_url.as_ref(),
                            current_schema.definitions.as_ref(),
                            schema_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        get_all_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            current_schema.schema_url.as_ref(),
                            current_schema.definitions.as_ref(),
                            schema_context,
                        )
                        .await
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        .boxed()
    }
}

impl GetTypeDefinition for tombi_schema_store::StringSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<TypeDefinition>> {
        async move {
            current_schema.map(|schema| {
                let mut schema_url = schema.schema_url.as_ref().clone();
                schema_url.set_fragment(Some(&format!("L{}", self.range.start.line + 1)));

                TypeDefinition {
                    schema_url,
                    schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                    range: schema.value_schema.range(),
                }
            })
        }
        .boxed()
    }
}
