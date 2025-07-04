mod all_of;
mod any_of;
mod one_of;
mod value;

use std::{borrow::Cow, ops::Deref};

use tombi_schema_store::{CurrentSchema, SchemaUrl};

pub async fn get_type_definition(
    tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Option<TypeDefinition> {
    let table = tree.deref();
    match schema_context.root_schema {
        Some(document_schema) => {
            let current_schema =
                document_schema
                    .value_schema
                    .as_ref()
                    .map(|value_schema| CurrentSchema {
                        value_schema: Cow::Borrowed(value_schema),
                        schema_url: Cow::Borrowed(&document_schema.schema_url),
                        definitions: Cow::Borrowed(&document_schema.definitions),
                    });
            table
                .get_type_definition(position, keys, &[], current_schema.as_ref(), schema_context)
                .await
        }
        None => {
            table
                .get_type_definition(position, keys, &[], None, schema_context)
                .await
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDefinition {
    pub schema_url: SchemaUrl,
    pub schema_accessors: Vec<tombi_schema_store::SchemaAccessor>,
    pub range: tombi_text::Range,
}

impl TypeDefinition {
    pub fn update_range(
        mut self,
        accessors: &[tombi_schema_store::Accessor],
        range: &tombi_text::Range,
    ) -> Self {
        if self.schema_accessors == accessors {
            self.range = *range;
        }
        self
    }
}

trait GetTypeDefinition {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<crate::goto_type_definition::TypeDefinition>>;
}
