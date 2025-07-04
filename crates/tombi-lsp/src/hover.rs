mod all_of;
mod any_of;
mod constraints;
mod display_value;
mod one_of;
mod value;

use std::{borrow::Cow, fmt::Debug, ops::Deref};

use constraints::ValueConstraints;
use tombi_schema_store::{
    get_schema_name, Accessor, Accessors, CurrentSchema, SchemaUrl, ValueType,
};

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
                        value_schema: Cow::Borrowed(value_schema),
                        schema_url: Cow::Borrowed(&document_schema.schema_url),
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

trait GetHoverContent {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>>;
}

#[derive(Debug, Clone)]
pub struct HoverContent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub accessors: Accessors,
    pub value_type: ValueType,
    pub constraints: Option<ValueConstraints>,
    pub schema_url: Option<SchemaUrl>,
    pub range: Option<tombi_text::Range>,
}

impl HoverContent {
    pub fn into_nullable(mut self) -> HoverContent {
        self.value_type = self.value_type.into_nullable();
        self
    }
}

impl PartialEq for HoverContent {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.description == other.description
            && self.accessors == other.accessors
            && self.value_type == other.value_type
            && self.range == other.range
    }
}

impl Eq for HoverContent {}

impl std::hash::Hash for HoverContent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.description.hash(state);
        self.accessors.hash(state);
        self.value_type.hash(state);
        self.range.hash(state);
    }
}

impl std::fmt::Display for HoverContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SECTION_SEPARATOR: &str = "-----";

        if let Some(title) = &self.title {
            writeln!(f, "#### {}\n", title)?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "{}\n", description)?;
        }

        if self.title.is_some() || self.description.is_some() {
            writeln!(f, "{}\n", SECTION_SEPARATOR)?;
        }

        if !self.accessors.is_empty() {
            writeln!(f, "Keys: `{}`\n", self.accessors)?;
        }
        writeln!(f, "Value: `{}`\n", self.value_type)?;

        if let Some(constraints) = &self.constraints {
            writeln!(f, "{}", constraints)?;
        }

        if let Some(schema_url) = &self.schema_url {
            if let Some(schema_filename) = get_schema_name(schema_url) {
                writeln!(f, "Schema: [{schema_filename}]({schema_url})\n",)?;
            }
        }

        Ok(())
    }
}

impl From<HoverContent> for tower_lsp::lsp_types::Hover {
    fn from(value: HoverContent) -> Self {
        tower_lsp::lsp_types::Hover {
            contents: tower_lsp::lsp_types::HoverContents::Markup(
                tower_lsp::lsp_types::MarkupContent {
                    kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                    value: value.to_string(),
                },
            ),
            range: value.range.map(Into::into),
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use tombi_schema_store::SchemaUrl;

    use super::*;

    #[rstest]
    #[case("https://json.schemastore.org/tombi.schema.json")]
    #[case("file://./folder/tombi.schema.json")]
    #[case("file://./tombi.schema.json")]
    #[case("file://tombi.schema.json")]
    fn url_content(#[case] url: &str) {
        let url = SchemaUrl::parse(url).unwrap();
        pretty_assertions::assert_eq!(get_schema_name(&url).unwrap(), "tombi.schema.json");
    }
}
