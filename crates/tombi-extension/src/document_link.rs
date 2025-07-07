use std::borrow::Cow;

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
        let version = env!("CARGO_PKG_VERSION");
        let branch = if version == "0.0.0-dev" {
            "main".to_string()
        } else {
            format!("refs/tags/v{version}")
        };
        if let Some(schema_filename) = url.path().strip_prefix("/json/schemas/") {
            tower_lsp::lsp_types::Url::parse(&format!(
                "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/schemas/{schema_filename}"
            )).ok()
        } else if url.path() == "/json/catalog.json" {
            tower_lsp::lsp_types::Url::parse(&format!(
                "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/schemas/catalog.json"
            ))
            .ok()
        } else {
            None
        }
    } else {
        Some(url.clone())
    }
}
