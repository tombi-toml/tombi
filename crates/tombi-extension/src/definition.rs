use tower_lsp::lsp_types::request::GotoDeclarationResponse;

#[derive(Debug)]
pub struct DefinitionLocation {
    pub uri: tower_lsp::lsp_types::Url,
    pub range: tombi_text::Range,
}

impl From<DefinitionLocation> for tower_lsp::lsp_types::Location {
    fn from(definition_location: DefinitionLocation) -> Self {
        tower_lsp::lsp_types::Location::new(
            definition_location.uri,
            definition_location.range.into(),
        )
    }
}

impl From<DefinitionLocation> for GotoDeclarationResponse {
    fn from(val: DefinitionLocation) -> Self {
        GotoDeclarationResponse::Scalar(val.into())
    }
}

impl From<DefinitionLocation> for DefinitionLocations {
    fn from(val: DefinitionLocation) -> Self {
        DefinitionLocations(vec![val])
    }
}

#[derive(Debug)]
pub struct DefinitionLocations(pub Vec<DefinitionLocation>);

impl From<DefinitionLocations> for Option<GotoDeclarationResponse> {
    fn from(val: DefinitionLocations) -> Self {
        match val.0.len() {
            0 => None,
            1 => Some(GotoDeclarationResponse::Scalar(
                val.0.into_iter().next().unwrap().into(),
            )),
            _ => Some(GotoDeclarationResponse::Array(
                val.0.into_iter().map(|location| location.into()).collect(),
            )),
        }
    }
}
