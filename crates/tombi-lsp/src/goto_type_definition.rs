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
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cursor: crate::CursorPosition<'_>,
    keys: &[tombi_document_tree_syntax::Key<'_>],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Vec<TypeDefinition> {
    let offset = cursor.offset();
    let Some(source) = TypeDefinitionSource::new(document_tree, offset, keys, schema_context).await
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
                    cursor,
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
                    cursor,
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
                    cursor,
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
    locations: Option<Vec<SchemaLocation>>,
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
    pub schema_base_uri: SchemaUri,

    pub schema_accessors: Vec<tombi_schema_store::SchemaAccessor>,

    /// The span of the schema definition in the JSON Schema file, with its line index.
    ///
    /// `None` opens the file at the line of the fragment of [`Self::schema_base_uri`].
    pub span: Option<tombi_extension::LocatedSpan>,
}

/// A location in a JSON Schema file.
///
/// Its range is converted from the span of the JSON Schema file with the encoding of the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaLocation {
    pub uri: tombi_uri::Uri,
    pub range: tombi_text::Range,
}

pub(crate) fn location_key(
    schema_base_uri: &SchemaUri,
    span: Option<tombi_text::Span>,
) -> (&str, Option<tombi_text::Span>) {
    let uri = schema_base_uri.as_str();
    match span {
        None => (uri, span),
        Some(_) => (uri.split_once('#').map_or(uri, |(base, _)| base), span),
    }
}

/// The fragment of a schema URI that opens the JSON Schema file at the line of `span`.
pub(crate) fn schema_line_fragment(
    line_index: &tombi_text::OwnedLineIndex,
    span: tombi_text::Span,
) -> String {
    format!("L{}", line_index.as_line_index().line(span.start) + 1)
}

impl TypeDefinition {
    /// Replaces the span with `span` of the document of `line_index`
    /// when this type definition is for `accessors`.
    pub fn update_span(
        mut self,
        accessors: &[tombi_schema_store::Accessor],
        span: tombi_text::Span,
        line_index: &std::sync::Arc<tombi_text::OwnedLineIndex>,
    ) -> Self {
        if self.schema_accessors == accessors {
            self.span = Some(tombi_extension::LocatedSpan {
                span,
                line_index: line_index.clone(),
            });
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
        cursor: crate::CursorPosition<'a>,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
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
    cursor: crate::CursorPosition<'_>,
    keys: &[tombi_document_tree_syntax::Key<'_>],
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
            cursor,
            keys,
            accessors,
            one_of_schema,
            current_schema,
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
            cursor,
            keys,
            accessors,
            any_of_schema,
            current_schema,
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
            cursor,
            keys,
            accessors,
            all_of_schema,
            current_schema,
            schema_context,
        )
        .await
        && !type_definitions.is_empty()
    {
        return type_definitions;
    }

    Vec::new()
}

/// A type definition that opens the JSON Schema file of `current_schema` at the line of `span`.
pub(super) fn schema_type_definition(
    current_schema: &CurrentSchema<'_>,
    accessors: &[Accessor],
    span: tombi_text::Span,
) -> TypeDefinition {
    let mut schema_base_uri = current_schema.schema_base_uri.as_ref().clone();
    schema_base_uri.set_fragment(Some(&schema_line_fragment(
        &current_schema.line_index,
        span,
    )));

    TypeDefinition {
        schema_base_uri,
        schema_accessors: accessors.iter().map(Into::into).collect_vec(),
        span: None,
    }
}

/// A type definition of the schema view of `current_schema`, at the line of `span`.
pub(super) fn schema_view_type_definition(
    current_schema: &CurrentSchema<'_>,
    accessors: &[Accessor],
    span: tombi_text::Span,
) -> TypeDefinition {
    let mut schema_base_uri = current_schema.schema_base_uri.as_ref().clone();
    schema_base_uri.set_fragment(Some(&schema_line_fragment(
        &current_schema.line_index,
        span,
    )));

    TypeDefinition {
        schema_base_uri,
        schema_accessors: accessors.iter().map(Into::into).collect_vec(),
        span: Some(tombi_extension::LocatedSpan {
            span: current_schema.schema_view.span(),
            line_index: current_schema.line_index.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::location_key;

    #[test]
    fn location_key_preserves_fragment_when_span_is_unknown() {
        let first = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L1").unwrap();
        let second = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L2").unwrap();

        assert_ne!(location_key(&first, None), location_key(&second, None));
    }

    #[test]
    fn location_key_ignores_fragment_when_span_identifies_the_location() {
        let first = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L1").unwrap();
        let second = tombi_schema_store::SchemaUri::from_str("file:///schema.json#L2").unwrap();
        let span = Some(tombi_text::Span::new(
            tombi_text::Offset::new(3),
            tombi_text::Offset::new(8),
        ));

        assert_eq!(location_key(&first, span), location_key(&second, span));
    }
}
