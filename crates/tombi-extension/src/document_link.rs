use std::{borrow::Cow, str::FromStr};

use tombi_schema_store::get_tombi_schemastore_content;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentLink {
    pub target: tombi_uri::Uri,
    pub range: tombi_text::Range,
    pub tooltip: Cow<'static, str>,
}

impl From<DocumentLink> for tower_lsp::lsp_types::DocumentLink {
    fn from(value: DocumentLink) -> Self {
        tower_lsp::lsp_types::DocumentLink {
            range: value.range.into(),
            target: Some(value.target.into()),
            tooltip: Some(value.tooltip.into_owned()),
            data: None,
        }
    }
}

pub fn get_tombi_github_uri(uri: &tombi_uri::Uri) -> Option<tombi_uri::Uri> {
    if uri.scheme() == "tombi" {
        if get_tombi_schemastore_content(uri).is_some() {
            let version = env!("CARGO_PKG_VERSION");
            let branch = if version == "0.0.0-dev" {
                "main".to_string()
            } else {
                format!("refs/tags/v{version}")
            };

            if uri.path().ends_with("/json/catalog.json") {
                tombi_uri::Uri::from_str(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/{host}/api/json/catalog.json",
                    host = uri.host_str().unwrap()
                ))
                .ok()
            } else if let Some(schema_filename) = uri
                .path_segments()
                .and_then(|mut segments| segments.next_back())
            {
                tombi_uri::Uri::from_str(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/{host}/{schema_filename}",
                    host = uri.host_str().unwrap()
                )).ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        Some(uri.clone())
    }
}
