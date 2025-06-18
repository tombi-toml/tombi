use std::borrow::Cow;

use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tower_lsp::lsp_types::TextDocumentIdentifier;

pub enum DocumentLinkToolTip {
    Catalog,
    Schema,
}

impl From<&DocumentLinkToolTip> for &'static str {
    fn from(val: &DocumentLinkToolTip) -> Self {
        match val {
            DocumentLinkToolTip::Catalog => "Open JSON Schema Catalog",
            DocumentLinkToolTip::Schema => "Open JSON Schema",
        }
    }
}

impl From<DocumentLinkToolTip> for &'static str {
    fn from(val: DocumentLinkToolTip) -> Self {
        (&val).into()
    }
}

impl From<DocumentLinkToolTip> for Cow<'static, str> {
    fn from(val: DocumentLinkToolTip) -> Self {
        Cow::Borrowed(val.into())
    }
}

impl std::fmt::Display for DocumentLinkToolTip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(self))
    }
}

pub async fn document_link(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    _toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DocumentLink>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is tombi.toml
    if !text_document.uri.path().ends_with("tombi.toml") {
        return Ok(None);
    }

    let Some(tombi_toml_path) = text_document.uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut document_links = vec![];

    if let Some((_, path)) = dig_keys(document_tree, &["schema", "catalog", "path"]) {
        let paths = match path {
            tombi_document_tree::Value::String(path) => vec![path],
            tombi_document_tree::Value::Array(paths) => paths
                .iter()
                .filter_map(|v| {
                    if let tombi_document_tree::Value::String(s) = v {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::with_capacity(0),
        };
        for path in paths {
            // Convert the path to a URL
            if let Some(target) = crate::str2url(path.value(), &tombi_toml_path) {
                document_links.push(tombi_extension::DocumentLink {
                    target,
                    range: path.unquoted_range(),
                    tooltip: DocumentLinkToolTip::Catalog.into(),
                });
            }
        }
    }

    if let Some((_, tombi_document_tree::Value::Array(paths))) =
        dig_keys(document_tree, &["schema", "catalog", "paths"])
    {
        for path in paths.iter() {
            let tombi_document_tree::Value::String(path) = path else {
                continue;
            };
            // Convert the path to a URL
            if let Some(target) = crate::str2url(path.value(), &tombi_toml_path) {
                document_links.push(tombi_extension::DocumentLink {
                    target,
                    range: path.unquoted_range(),
                    tooltip: DocumentLinkToolTip::Catalog.into(),
                });
            }
        }
    }

    if let Some((_, tombi_document_tree::Value::Array(schemas))) =
        dig_keys(document_tree, &["schemas"])
    {
        for schema in schemas.iter() {
            let tombi_document_tree::Value::Table(table) = schema else {
                continue;
            };
            let Some(tombi_document_tree::Value::String(path)) = table.get("path") else {
                continue;
            };
            let Some(target) = crate::str2url(path.value(), &tombi_toml_path) else {
                continue;
            };

            document_links.push(tombi_extension::DocumentLink {
                target,
                range: path.unquoted_range(),
                tooltip: DocumentLinkToolTip::Schema.into(),
            });
        }
    }

    if document_links.is_empty() {
        return Ok(None);
    }

    Ok(Some(document_links))
}
