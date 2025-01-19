use tower_lsp::lsp_types::Url;

use super::GetHoverContent;

impl GetHoverContent for document_tree::Value {
    fn get_hover_content(
        &self,
        accessors: &Vec<schema_store::Accessor>,
        value_schema: Option<&schema_store::ValueSchema>,
        toml_version: config::TomlVersion,
        position: text::Position,
        keys: &[document_tree::Key],
        schema_url: Option<&Url>,
        definitions: &schema_store::SchemaDefinitions,
    ) -> Option<super::HoverContent> {
        match self {
            Self::Boolean(boolean) => boolean.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::Integer(integer) => integer.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::Float(float) => float.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::String(string) => string.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::OffsetDateTime(offset_date_time) => offset_date_time.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::LocalDateTime(local_date_time) => local_date_time.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::LocalDate(local_date) => local_date.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::LocalTime(local_time) => local_time.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::Array(array) => array.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Self::Table(table) => table.get_hover_content(
                accessors,
                value_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
        }
    }
}
