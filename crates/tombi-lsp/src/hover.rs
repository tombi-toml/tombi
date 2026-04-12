mod all_of;
mod any_of;
mod comment;
mod constraints;
mod display_value;
mod one_of;
mod value;

use std::{borrow::Cow, fmt::Debug, ops::Deref};

pub use comment::get_document_comment_directive_hover_content;
use constraints::ValueConstraints;

use tombi_extension::get_tombi_github_uri;
use tombi_schema_store::{
    Accessor, Accessors, AllOfSchema, AnyOfSchema, CurrentSchema, OneOfSchema, SchemaUri,
    ValueType, get_schema_name,
};
use tombi_text::{FromLsp, IntoLsp};

pub async fn get_hover_content(
    tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
) -> Option<HoverContent> {
    let table = tree.deref();
    match schema_context.root_schema {
        Some(document_schema) => {
            let current_schema =
                document_schema
                    .value_schema
                    .as_ref()
                    .map(|value_schema| CurrentSchema {
                        value_schema: value_schema.clone(),
                        schema_uri: Cow::Borrowed(&document_schema.schema_uri),
                        definitions: Cow::Borrowed(&document_schema.definitions),
                    });
            table
                .get_hover_content(position, keys, &[], current_schema.as_ref(), schema_context)
                .await
        }
        None => {
            table
                .get_hover_content(position, keys, &[], None, schema_context)
                .await
        }
    }
}

pub(super) trait GetHoverContent {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>>;
}

fn merge_optional_vec<T: PartialEq>(
    base: Option<Vec<T>>,
    adjacent: Option<Vec<T>>,
) -> Option<Vec<T>> {
    let mut values = base.unwrap_or_default();
    for value in adjacent.unwrap_or_default() {
        if !values.contains(&value) {
            values.push(value);
        }
    }

    (!values.is_empty()).then_some(values)
}

fn merge_constraints(
    base: Option<ValueConstraints>,
    adjacent: Option<ValueConstraints>,
) -> Option<ValueConstraints> {
    match (base, adjacent) {
        (Some(mut base), Some(adjacent)) => {
            base.r#enum = merge_optional_vec(base.r#enum, adjacent.r#enum);
            base.default = base.default.or(adjacent.default);
            base.examples = merge_optional_vec(base.examples, adjacent.examples);
            base.minimum = base.minimum.or(adjacent.minimum);
            base.maximum = base.maximum.or(adjacent.maximum);
            base.exclusive_minimum = base.exclusive_minimum.or(adjacent.exclusive_minimum);
            base.exclusive_maximum = base.exclusive_maximum.or(adjacent.exclusive_maximum);
            base.multiple_of = base.multiple_of.or(adjacent.multiple_of);
            base.min_length = base.min_length.or(adjacent.min_length);
            base.max_length = base.max_length.or(adjacent.max_length);
            base.format = base.format.or(adjacent.format);
            base.pattern = base.pattern.or(adjacent.pattern);
            base.min_items = base.min_items.or(adjacent.min_items);
            base.max_items = base.max_items.or(adjacent.max_items);
            base.unique_items = base.unique_items.or(adjacent.unique_items);
            base.values_order = base.values_order.or(adjacent.values_order);
            base.required_keys = merge_optional_vec(base.required_keys, adjacent.required_keys);
            base.min_keys = base.min_keys.or(adjacent.min_keys);
            base.max_keys = base.max_keys.or(adjacent.max_keys);
            base.key_patterns = merge_optional_vec(base.key_patterns, adjacent.key_patterns);
            base.additional_keys = base.additional_keys.or(adjacent.additional_keys);
            base.pattern_keys |= adjacent.pattern_keys;
            base.keys_order = base.keys_order.or(adjacent.keys_order);
            base.array_values_order_by = base
                .array_values_order_by
                .or(adjacent.array_values_order_by);
            Some(base)
        }
        (Some(base), None) => Some(base),
        (None, Some(adjacent)) => Some(adjacent),
        (None, None) => None,
    }
}

fn merge_hover_value_content(
    mut base: HoverValueContent,
    adjacent: HoverValueContent,
) -> HoverValueContent {
    base.title = base.title.or(adjacent.title);
    base.description = base.description.or(adjacent.description);
    base.constraints = merge_constraints(base.constraints, adjacent.constraints);
    base.schema_uri = base.schema_uri.or(adjacent.schema_uri);
    base.range = base.range.or(adjacent.range);
    base
}

fn merge_hover_content(
    base: Option<HoverContent>,
    adjacent: Option<HoverContent>,
) -> Option<HoverContent> {
    match (base, adjacent) {
        (Some(HoverContent::Value(base)), Some(HoverContent::Value(adjacent))) => Some(
            HoverContent::Value(merge_hover_value_content(base, adjacent)),
        ),
        (
            Some(HoverContent::DirectiveContent(base)),
            Some(HoverContent::DirectiveContent(adjacent)),
        ) => Some(HoverContent::DirectiveContent(merge_hover_value_content(
            base, adjacent,
        ))),
        (Some(HoverContent::Value(base)), Some(HoverContent::DirectiveContent(adjacent)))
        | (Some(HoverContent::DirectiveContent(base)), Some(HoverContent::Value(adjacent))) => {
            Some(HoverContent::Value(merge_hover_value_content(
                base, adjacent,
            )))
        }
        (Some(base), None) => Some(base),
        (None, Some(adjacent)) => Some(adjacent),
        (Some(base), Some(_)) => Some(base),
        (None, None) => None,
    }
}

pub(super) async fn merge_adjacent_hover_content<
    T: GetHoverContent
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
    base_hover_content: Option<HoverContent>,
    one_of_schema: Option<&OneOfSchema>,
    any_of_schema: Option<&AnyOfSchema>,
    all_of_schema: Option<&AllOfSchema>,
) -> Option<HoverContent> {
    let Some(current_schema) = current_schema else {
        return base_hover_content;
    };

    let mut hover_content = base_hover_content;

    if let Some(one_of_schema) = one_of_schema {
        hover_content = merge_hover_content(
            hover_content,
            one_of::get_one_of_hover_content(
                value,
                position,
                keys,
                accessors,
                one_of_schema,
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context,
            )
            .await,
        );
    }
    if let Some(any_of_schema) = any_of_schema {
        hover_content = merge_hover_content(
            hover_content,
            any_of::get_any_of_hover_content(
                value,
                position,
                keys,
                accessors,
                any_of_schema,
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context,
            )
            .await,
        );
    }
    if let Some(all_of_schema) = all_of_schema {
        hover_content = merge_hover_content(
            hover_content,
            all_of::get_all_of_hover_content(
                value,
                position,
                keys,
                accessors,
                all_of_schema,
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context,
            )
            .await,
        );
    }

    hover_content
}

#[derive(Debug, Clone)]
pub enum HoverContent {
    Value(HoverValueContent),
    Directive(HoverDirectiveContent),
    DirectiveContent(HoverValueContent),
}

impl FromLsp<HoverContent> for tower_lsp::lsp_types::Hover {
    fn from_lsp(source: HoverContent, line_index: &tombi_text::LineIndex) -> Self {
        match source {
            HoverContent::Value(content) => content.into_lsp(line_index),
            HoverContent::Directive(content) => content.into_lsp(line_index),
            HoverContent::DirectiveContent(content) => content.into_lsp(line_index),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HoverDirectiveContent {
    pub title: String,
    pub description: String,
    pub range: tombi_text::Range,
}

impl FromLsp<HoverDirectiveContent> for tower_lsp::lsp_types::Hover {
    fn from_lsp(source: HoverDirectiveContent, line_index: &tombi_text::LineIndex) -> Self {
        tower_lsp::lsp_types::Hover {
            contents: tower_lsp::lsp_types::HoverContents::Markup(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: format!("#### {}\n\n{}", source.title, source.description),
                },
            ),
            range: Some(source.range.into_lsp(line_index)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HoverValueContent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub accessors: Accessors,
    pub value_type: ValueType,
    pub constraints: Option<ValueConstraints>,
    pub schema_uri: Option<SchemaUri>,
    pub range: Option<tombi_text::Range>,
}

impl PartialEq for HoverValueContent {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.description == other.description
            && self.accessors == other.accessors
            && self.value_type == other.value_type
            && self.constraints == other.constraints
            && self.range == other.range
    }
}

impl Eq for HoverValueContent {}

impl std::hash::Hash for HoverValueContent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.description.hash(state);
        self.accessors.hash(state);
        self.value_type.hash(state);
        self.constraints.hash(state);
        self.range.hash(state);
    }
}

impl std::fmt::Display for HoverValueContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SECTION_SEPARATOR: &str = "-----";

        if let Some(title) = &self.title {
            writeln!(f, "#### {title}\n")?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "{description}\n")?;
        }

        if self.title.is_some() || self.description.is_some() {
            writeln!(f, "{SECTION_SEPARATOR}\n")?;
        }

        if !self.accessors.is_empty() {
            writeln!(f, "Keys: `{}`\n", self.accessors)?;
        }
        writeln!(f, "Value: `{}`\n", self.value_type)?;

        if let Some(constraints) = &self.constraints {
            writeln!(f, "{constraints}")?;
        }

        if let Some(schema_uri) = &self
            .schema_uri
            .as_ref()
            .and_then(|url| get_tombi_github_uri(url))
            && let Some(schema_filename) = get_schema_name(schema_uri)
        {
            writeln!(f, "Schema: [{schema_filename}]({schema_uri})\n",)?;
        }

        Ok(())
    }
}

impl FromLsp<HoverValueContent> for tower_lsp::lsp_types::Hover {
    fn from_lsp(source: HoverValueContent, line_index: &tombi_text::LineIndex) -> Self {
        tower_lsp::lsp_types::Hover {
            contents: tower_lsp::lsp_types::HoverContents::Markup(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: source.to_string(),
                },
            ),
            range: source.range.map(|range| range.into_lsp(line_index)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use rstest::rstest;
    use tombi_schema_store::SchemaUri;

    use super::*;

    #[rstest]
    #[case(concat!("https://", tombi_uri::schemastore_hostname!(), "/tombi.json"))]
    #[case("file://./folder/tombi.json")]
    #[case("file://./tombi.json")]
    #[case("file://tombi.json")]
    fn url_content(#[case] url: &str) {
        let url = SchemaUri::from_str(url).unwrap();
        pretty_assertions::assert_eq!(get_schema_name(&url).unwrap(), "tombi.json");
    }
}
