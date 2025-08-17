#[derive(Debug)]
pub struct DefinitionLocation {
    pub uri: tombi_uri::Uri,
    pub range: tombi_text::Range,
}

impl From<DefinitionLocation> for tower_lsp::lsp_types::Location {
    fn from(definition_location: DefinitionLocation) -> Self {
        tower_lsp::lsp_types::Location::new(
            definition_location.uri.into(),
            definition_location.range.into(),
        )
    }
}
