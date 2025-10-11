mod all_of;
mod any_of;
mod comment;
mod one_of;
mod value;

use std::{borrow::Cow, ops::Deref};

use ahash::AHashMap;
pub use comment::get_tombi_document_comment_directive_type_definition;
use itertools::Itertools;
use tombi_schema_store::{CurrentSchema, SchemaUri};
use tower_lsp::lsp_types::GotoDefinitionResponse;

use crate::{goto_definition::open_remote_file, Backend};

pub async fn get_type_definition(
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Option<TypeDefinition> {
    let table = document_tree.deref();
    match schema_context.root_schema {
        Some(document_schema) => {
            let current_schema =
                document_schema
                    .value_schema
                    .as_ref()
                    .map(|value_schema| CurrentSchema {
                        value_schema: Cow::Borrowed(value_schema),
                        schema_uri: Cow::Borrowed(&document_schema.schema_uri),
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

pub async fn into_type_definition_locations(
    backend: &Backend,
    definitions: Option<Vec<tombi_extension::DefinitionLocation>>,
) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
    let Some(definitions) = definitions else {
        return Ok(None);
    };

    let mut uri_set = AHashMap::new();
    for definition in &definitions {
        if let Ok(Some(remote_uri)) = open_remote_file(backend, &definition.uri).await {
            uri_set.insert(definition.uri.clone(), remote_uri);
        }
    }

    let definitions = definitions
        .into_iter()
        .map(|mut definition| {
            if let Some(remote_uri) = uri_set.get(&definition.uri) {
                definition.uri = remote_uri.clone();
            }
            tower_lsp::lsp_types::Location::new(
                definition.uri.into(),
                tombi_text::convert_range_to_lsp(definition.range),
            )
        })
        .collect_vec();

    match definitions.len() {
        0 => Ok(None),
        1 => Ok(Some(GotoDefinitionResponse::Scalar(
            definitions.into_iter().next().unwrap(),
        ))),
        _ => Ok(Some(GotoDefinitionResponse::Array(definitions))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDefinition {
    pub schema_uri: SchemaUri,

    pub schema_accessors: Vec<tombi_schema_store::SchemaAccessor>,

    /// The range of the schema definition.
    ///
    /// It's JSON Schema file range, not TOML file range.
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
