use tombi_text::{FromLsp, IntoLsp};

#[derive(Debug)]
pub struct Location {
    pub uri: tombi_uri::Uri,
    pub range: tombi_text::Range,
}

impl FromLsp<Location> for tower_lsp::lsp_types::Location {
    fn from_lsp(
        source: Location,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::Location {
        tower_lsp::lsp_types::Location::new(source.uri.into(), source.range.into_lsp(line_index))
    }
}
