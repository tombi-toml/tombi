mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;
mod value;

use config::TomlVersion;
use dashmap::DashMap;
use itertools::Itertools;
use schema_store::{Accessor, Accessors, DocumentSchema, ValueSchema, ValueType};
use std::{fmt::Debug, ops::Deref};

trait GetHoverContent {
    fn get_hover_content(
        &self,
        accessors: &mut Vec<Accessor>,
        value_schema: Option<&ValueSchema>,
        toml_version: TomlVersion,
        position: text::Position,
        keys: &[document_tree::Key],
        definitions: &schema_store::SchemaDefinitions,
    ) -> Option<HoverContent>;
}

#[derive(Debug, Default)]
pub struct HoverContent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keys: Accessors,
    pub value_type: ValueType,
    pub enumerated_values: Vec<String>,
    pub schema_url: Option<tower_lsp::lsp_types::Url>,
    pub range: Option<text::Range>,
}

impl std::fmt::Display for HoverContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(title) = &self.title {
            writeln!(f, "## {}\n", title)?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "{}\n", description.split("\n").join("\n\n"))?;
        }

        writeln!(f, "Keys: `{}`\n", self.keys)?;
        writeln!(f, "Value: `{:?}`\n", self.value_type)?;

        if !self.enumerated_values.is_empty() {
            writeln!(f, "Allowed Values:\n")?;
            for value in &self.enumerated_values {
                writeln!(f, "- `{}`", value)?;
            }
            writeln!(f)?;
        }

        if let Some(schema_url) = &self.schema_url {
            if let Some(schema_filename) = get_schema_name(schema_url) {
                writeln!(f, "Source: [{schema_filename}]({schema_url})\n",)?;
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

pub fn get_hover_content(
    root: &document_tree::Root,
    toml_version: TomlVersion,
    position: text::Position,
    keys: &[document_tree::Key],
    document_schema: Option<&DocumentSchema>,
) -> Option<HoverContent> {
    let table = root.deref();
    table.get_hover_content(
        &mut vec![],
        None,
        toml_version,
        position,
        keys,
        document_schema
            .map(|schema| &schema.definitions)
            .unwrap_or(&DashMap::new()),
    )
}

fn get_schema_name(schema_url: &tower_lsp::lsp_types::Url) -> Option<&str> {
    if let Some(path) = schema_url.path().split('/').last() {
        if !path.is_empty() {
            return Some(path);
        }
    }
    schema_url.host_str()
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
    use tower_lsp::lsp_types::Url;

    #[rstest]
    #[case("https://json.schemastore.org/tombi.schema.json")]
    #[case("file://./folder/tombi.schema.json")]
    #[case("file://./tombi.schema.json")]
    #[case("file://tombi.schema.json")]
    fn url_content(#[case] url: &str) {
        let url = Url::parse(url).unwrap();
        assert_eq!(get_schema_name(&url).unwrap(), "tombi.schema.json");
    }
}
