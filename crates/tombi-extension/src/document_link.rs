use std::borrow::Cow;

use tombi_schema_store::get_tombi_schemastore_content;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentLink {
    pub target: tower_lsp::lsp_types::Url,
    pub range: tombi_text::Range,
    pub tooltip: Cow<'static, str>,
}

impl From<DocumentLink> for tower_lsp::lsp_types::DocumentLink {
    fn from(value: DocumentLink) -> Self {
        tower_lsp::lsp_types::DocumentLink {
            range: value.range.into(),
            target: Some(value.target),
            tooltip: Some(value.tooltip.into_owned()),
            data: None,
        }
    }
}

pub fn get_tombi_github_url(url: &tower_lsp::lsp_types::Url) -> Option<tower_lsp::lsp_types::Url> {
    if url.scheme() == "tombi" {
        if get_tombi_schemastore_content(url).is_some() {
            let version = env!("CARGO_PKG_VERSION");
            let branch = if version == "0.0.0-dev" {
                "main".to_string()
            } else {
                format!("refs/tags/v{version}")
            };

            if url.path().ends_with("/json/catalog.json") {
                tower_lsp::lsp_types::Url::parse(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/json.schemastore.org/api/json/catalog.json"
                ))
                .ok()
            } else if let Some(schema_filename) =
                url.path_segments().and_then(|segments| segments.last())
            {
                tower_lsp::lsp_types::Url::parse(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/json.schemastore.org/{schema_filename}"
                )).ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        Some(url.clone())
    }
}
