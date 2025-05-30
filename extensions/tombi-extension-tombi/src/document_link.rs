use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tower_lsp::lsp_types::TextDocumentIdentifier;

pub enum DocumentLinkToolTip {
    Catalog,
    Schema,
}

impl std::fmt::Display for DocumentLinkToolTip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentLinkToolTip::Catalog => write!(f, "Open JSON Schema Catalog"),
            DocumentLinkToolTip::Schema => write!(f, "Open JSON Schema"),
        }
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

    if let Some((_, tombi_document_tree::Value::String(path))) =
        dig_keys(document_tree, &["schema", "catalog", "path"])
    {
        if let Some(target) = crate::str2url(path.value(), &tombi_toml_path) {
            document_links.push(tombi_extension::DocumentLink {
                target,
                range: path.unquoted_range(),
                tooltip: DocumentLinkToolTip::Catalog.to_string(),
            });
        };
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
                tooltip: DocumentLinkToolTip::Schema.to_string(),
            });
        }
    }

    if document_links.is_empty() {
        return Ok(None);
    }

    Ok(Some(document_links))
}
