use std::{borrow::Cow, str::FromStr};

use tombi_schema_store::get_tombi_schemastore_content;
use tombi_text::{FromLsp, IntoLsp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentLink {
    pub target: tombi_uri::Uri,
    pub range: tombi_text::Range,
    pub tooltip: Cow<'static, str>,
}

impl FromLsp<DocumentLink> for tower_lsp::lsp_types::DocumentLink {
    fn from_lsp(
        source: DocumentLink,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::DocumentLink {
        tower_lsp::lsp_types::DocumentLink {
            range: source.range.into_lsp(line_index),
            target: Some(source.target.into()),
            tooltip: Some(source.tooltip.into_owned()),
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
            let mut host = uri.host_str().unwrap();
            if host == "www.schemastore.org" {
                host = "json.schemastore.org";
            }

            if uri.path().ends_with("/json/catalog.json") {
                tombi_uri::Uri::from_str(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/{host}/api/json/catalog.json",
                ))
                .ok()
            } else if let Some(schema_filename) = uri
                .path_segments()
                .and_then(|mut segments| segments.next_back())
            {
                tombi_uri::Uri::from_str(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/{host}/{schema_filename}",
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
