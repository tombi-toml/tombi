use tombi_config::TomlVersion;
use tombi_schema_store::{dig_accessors, matches_accessors};
use tower_lsp::lsp_types::{TextDocumentIdentifier, Url};

pub async fn goto_definition(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    _toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is tombi.toml
    if !text_document.uri.path().ends_with("tombi.toml") {
        return Ok(Default::default());
    }

    let Some(tombi_toml_path) = text_document.uri.to_file_path().ok() else {
        return Ok(Default::default());
    };

    let mut locations = vec![];

    if accessors.last() == Some(&tombi_schema_store::Accessor::Key("path".to_string())) {
        if let Some((_, tombi_document_tree::Value::String(path))) =
            dig_accessors(document_tree, accessors)
        {
            if let Some(uri) = get_definition_link(path.value(), &tombi_toml_path) {
                locations.push(tombi_extension::DefinitionLocation {
                    uri,
                    range: tombi_text::Range::default(),
                });
            }
        }
    }

    if matches!(accessors.len(), 3 | 4) {
        if matches_accessors!(accessors[..3], ["schema", "catalog", "paths"]) {
            if let Some((_, tombi_document_tree::Value::Array(paths))) =
                dig_accessors(document_tree, &accessors[..3])
            {
                let index = (accessors.len() == 4)
                    .then(|| accessors.last().and_then(|accessor| accessor.as_index()))
                    .flatten();

                for (i, path) in paths.iter().enumerate() {
                    let tombi_document_tree::Value::String(path) = path else {
                        continue;
                    };
                    if index.is_some() && index != Some(i) {
                        continue;
                    }
                    if let Some(uri) = get_definition_link(path.value(), &tombi_toml_path) {
                        locations.push(tombi_extension::DefinitionLocation {
                            uri,
                            range: tombi_text::Range::default(),
                        });
                    }
                }
            }
        }
    }

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}

fn get_definition_link(url_str: &str, tombi_toml_path: &std::path::Path) -> Option<Url> {
    if let Ok(url) = Url::parse(url_str) {
        Some(url)
    } else if let Some(tombi_config_dir) = tombi_toml_path.parent() {
        let mut file_path = std::path::PathBuf::from(url_str);
        if file_path.is_relative() {
            file_path = tombi_config_dir.join(file_path);
        }
        if file_path.exists() {
            Url::from_file_path(file_path).ok()
        } else {
            None
        }
    } else {
        None
    }
}
