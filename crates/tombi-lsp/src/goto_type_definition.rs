mod all_of;
mod any_of;
mod comment;
mod one_of;
mod type_definition_source;
mod value;

use std::ops::Deref;

pub use comment::get_tombi_document_comment_directive_type_definition;
use itertools::Itertools;
use tombi_schema_store::{
    Accessor, AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, SchemaUri,
};
use tower_lsp::lsp_types::GotoDefinitionResponse;

use crate::{Backend, goto_definition::open_remote_file};

use self::type_definition_source::TypeDefinitionSource;

pub async fn get_type_definition(
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Option<TypeDefinition> {
    let source = TypeDefinitionSource::new(document_tree, position, keys, schema_context).await?;

    match source {
        TypeDefinitionSource::Root {
            remaining_keys,
            accessors,
            current_schema,
        } => {
            document_tree
                .deref()
                .get_type_definition(
                    position,
                    remaining_keys,
                    &accessors,
                    current_schema.as_ref(),
                    schema_context,
                )
                .await
        }
        TypeDefinitionSource::Value {
            remaining_keys,
            accessors,
            current_schema,
        } => {
            let (_, value) = tombi_document_tree::dig_accessors(document_tree, &accessors)?;
            value
                .get_type_definition(
                    position,
                    remaining_keys,
                    &accessors,
                    current_schema.as_ref(),
                    schema_context,
                )
                .await
        }
        TypeDefinitionSource::Schema {
            remaining_keys,
            accessors,
            current_schema,
        } => {
            current_schema
                .value_schema
                .get_type_definition(
                    position,
                    remaining_keys,
                    &accessors,
                    Some(&current_schema),
                    schema_context,
                )
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

    let mut uri_set = tombi_hashmap::HashMap::new();
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

pub(super) trait GetTypeDefinition {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<crate::goto_type_definition::TypeDefinition>>;
}

pub(super) async fn adjacent_type_definition<
    T: GetTypeDefinition
        + Sync
        + Send
        + tombi_document_tree::ValueImpl
        + tombi_validator::Validate
        + std::fmt::Debug,
>(
    value: &T,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    accessors: &[Accessor],
    current_schema: Option<&CurrentSchema<'_>>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    one_of_schema: Option<&OneOfSchema>,
    any_of_schema: Option<&AnyOfSchema>,
    all_of_schema: Option<&AllOfSchema>,
) -> Option<TypeDefinition> {
    let current_schema = current_schema?;

    if let Some(one_of_schema) = one_of_schema
        && let Some(type_definition) = one_of::get_one_of_type_definition(
            value,
            position,
            keys,
            accessors,
            one_of_schema,
            &current_schema.schema_uri,
            &current_schema.definitions,
            schema_context,
        )
        .await
    {
        return Some(type_definition);
    }
    if let Some(any_of_schema) = any_of_schema
        && let Some(type_definition) = any_of::get_any_of_type_definition(
            value,
            position,
            keys,
            accessors,
            any_of_schema,
            &current_schema.schema_uri,
            &current_schema.definitions,
            schema_context,
        )
        .await
    {
        return Some(type_definition);
    }
    if let Some(all_of_schema) = all_of_schema
        && let Some(type_definition) = all_of::get_all_of_type_definition(
            value,
            position,
            keys,
            accessors,
            all_of_schema,
            &current_schema.schema_uri,
            &current_schema.definitions,
            schema_context,
        )
        .await
    {
        return Some(type_definition);
    }

    None
}

pub(super) fn schema_type_definition(
    schema_uri: &SchemaUri,
    accessors: &[Accessor],
    range: tombi_text::Range,
) -> TypeDefinition {
    let mut schema_uri = schema_uri.clone();
    schema_uri.set_fragment(Some(&format!("L{}", range.start.line + 1)));

    TypeDefinition {
        schema_uri,
        schema_accessors: accessors.iter().map(Into::into).collect_vec(),
        range: tombi_text::Range::default(),
    }
}
