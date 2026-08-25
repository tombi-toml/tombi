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

use crate::{Backend, remote_file::open_remote_file};

use self::type_definition_source::TypeDefinitionSource;

pub async fn get_type_definition(
    document_tree: &tombi_document_tree_syntax::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree_syntax::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Vec<TypeDefinition> {
    let Some(source) =
        TypeDefinitionSource::new(document_tree, position, keys, schema_context).await
    else {
        return Vec::new();
    };

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
            let Some((_, value)) =
                tombi_document_tree_syntax::dig_accessors(document_tree, &accessors)
            else {
                return Vec::new();
            };
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
                .schema_view
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

pub async fn try_get_type_definition_response(
    backend: &Backend,
    locations: Option<Vec<tombi_extension::Location>>,
) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
    let Some(locations) = locations else {
        return Ok(None);
    };

    let mut uri_set = tombi_hashmap::HashMap::new();
    for location in &locations {
        if let Ok(Some(remote_uri)) = open_remote_file(backend, &location.uri).await {
            uri_set.insert(location.uri.clone(), remote_uri);
        }
    }

    let locations = locations
        .into_iter()
        .map(|mut location| {
            if let Some(remote_uri) = uri_set.get(&location.uri) {
                location.uri = remote_uri.clone();
            }
            tower_lsp::lsp_types::Location::new(
                location.uri.into(),
                tombi_text::convert_range_to_lsp(location.range),
            )
        })
        .collect_vec();

    match locations.len() {
        0 => Ok(None),
        1 => Ok(Some(GotoDefinitionResponse::Scalar(
            locations.into_iter().next().unwrap(),
        ))),
        _ => Ok(Some(GotoDefinitionResponse::Array(locations))),
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

pub(crate) fn location_key(
    schema_uri: &SchemaUri,
    range: tombi_text::Range,
) -> (&str, tombi_text::Range) {
    let uri = schema_uri.as_str();
    if range == tombi_text::Range::default() {
        (uri, range)
    } else {
        (uri.split_once('#').map_or(uri, |(base, _)| base), range)
    }
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

fn prefer_type_definitions(
    type_definitions: Vec<TypeDefinition>,
    fallback: Vec<TypeDefinition>,
) -> Vec<TypeDefinition> {
    if type_definitions.is_empty() {
        fallback
    } else {
        type_definitions
    }
}

pub(super) trait GetTypeDefinition {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>>;
}

pub(super) async fn adjacent_type_definition<
    T: GetTypeDefinition
        + Sync
        + Send
        + tombi_document_tree_syntax::ValueImpl
        + tombi_validator::Validate
        + std::fmt::Debug,
>(
    value: &T,
    position: tombi_text::Position,
    keys: &[tombi_document_tree_syntax::Key],
    accessors: &[Accessor],
    current_schema: Option<&CurrentSchema<'_>>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    one_of_schema: Option<&OneOfSchema>,
    any_of_schema: Option<&AnyOfSchema>,
    all_of_schema: Option<&AllOfSchema>,
) -> Vec<TypeDefinition> {
    let Some(current_schema) = current_schema else {
        return Vec::new();
    };

    if let Some(one_of_schema) = one_of_schema
        && let type_definitions = one_of::get_one_of_type_definition(
            value,
            position,
            keys,
            accessors,
            one_of_schema,
            &current_schema.schema_uri,
            &current_schema.definitions,
            current_schema.strict,
            schema_context,
        )
        .await
        && !type_definitions.is_empty()
    {
        return type_definitions;
    }
    if let Some(any_of_schema) = any_of_schema
        && let type_definitions = any_of::get_any_of_type_definition(
            value,
            position,
            keys,
            accessors,
            any_of_schema,
            &current_schema.schema_uri,
            &current_schema.definitions,
            current_schema.strict,
            schema_context,
        )
        .await
        && !type_definitions.is_empty()
    {
        return type_definitions;
    }
    if let Some(all_of_schema) = all_of_schema
        && let type_definitions = all_of::get_all_of_type_definition(
            value,
            position,
            keys,
            accessors,
            all_of_schema,
            &current_schema.schema_uri,
            &current_schema.definitions,
            current_schema.strict,
            schema_context,
        )
        .await
        && !type_definitions.is_empty()
    {
        return type_definitions;
    }

    Vec::new()
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::location_key;

    #[test]
    fn location_key_preserves_fragment_when_range_is_unknown() {
        let first = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L1").unwrap();
        let second = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L2").unwrap();

        assert_ne!(
            location_key(&first, tombi_text::Range::default()),
            location_key(&second, tombi_text::Range::default()),
        );
    }

    #[test]
    fn location_key_ignores_fragment_when_range_identifies_the_location() {
        let first = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L1").unwrap();
        let second = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L2").unwrap();
        let range = tombi_text::Range::new(
            tombi_text::Position::new(2, 3),
            tombi_text::Position::new(2, 8),
        );

        assert_eq!(location_key(&first, range), location_key(&second, range));
    }
}
