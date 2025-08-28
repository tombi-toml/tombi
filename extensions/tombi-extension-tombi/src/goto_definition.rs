use std::str::FromStr;

use tombi_config::TomlVersion;
use tombi_document_tree::dig_accessors;
use tombi_schema_store::matches_accessors;

pub async fn goto_definition(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    _toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is tombi.toml
    if !text_document_uri.path().ends_with("tombi.toml") {
        return Ok(Default::default());
    }

    let Some(tombi_toml_path) = text_document_uri.to_file_path().ok() else {
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

    if matches!(accessors.len(), 3 | 4)
        && matches_accessors!(accessors[..3], ["schema", "catalog", "paths"])
    {
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

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}

fn get_definition_link(url_str: &str, tombi_toml_path: &std::path::Path) -> Option<tombi_uri::Uri> {
    if let Ok(uri) = tombi_uri::Uri::from_str(url_str) {
        Some(uri)
    } else if let Some(tombi_config_dir) = tombi_toml_path.parent() {
        let mut file_path = std::path::PathBuf::from(url_str);
        if file_path.is_relative() {
            file_path = tombi_config_dir.join(file_path);
        }
        if file_path.exists() {
            tombi_uri::Uri::from_file_path(file_path).ok()
        } else {
            None
        }
    } else {
        None
    }
}
