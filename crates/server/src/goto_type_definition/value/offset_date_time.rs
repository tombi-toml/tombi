use futures::{future::BoxFuture, FutureExt};
use schema_store::ValueSchema;

use crate::goto_type_definition::{
    all_of::get_all_of_type_definition, any_of::get_any_of_type_definition,
    one_of::get_one_of_type_definition, GetTypeDefinition, TypeDefinition,
};

impl GetTypeDefinition for tombi_document_tree::OffsetDateTime {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [schema_store::Accessor],
        current_schema: Option<&'a schema_store::CurrentSchema<'a>>,
        schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<crate::goto_type_definition::TypeDefinition>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::OffsetDateTime(offset_date_time_schema) => {
                        offset_date_time_schema
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

impl GetTypeDefinition for schema_store::OffsetDateTimeSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        _accessors: &'a [schema_store::Accessor],
        current_schema: Option<&'a schema_store::CurrentSchema<'a>>,
        _schema_context: &'a schema_store::SchemaContext,
    ) -> BoxFuture<'b, Option<TypeDefinition>> {
        async move {
            current_schema.map(|schema| TypeDefinition {
                schema_url: schema.schema_url.as_ref().clone(),
                range: tombi_text::Range::default(),
            })
        }
        .boxed()
    }
}
