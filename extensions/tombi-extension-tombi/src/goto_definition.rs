use std::str::FromStr;

use tombi_config::{DOT_TOMBI_TOML_FILENAME, TOMBI_TOML_FILENAME, TomlVersion, config_base_dir};
use tombi_document_tree_syntax::dig_accessors;
use tombi_schema_store::matches_accessors;

use tombi_config::ExtendsTarget;

pub async fn goto_definition(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    _toml_version: TomlVersion,
    features: Option<&tombi_config::TombiExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp_server::jsonrpc::Error> {
    // Check if current file is .tombi.toml or tombi.toml
    let path = text_document_uri.path();
    if !(path.ends_with(DOT_TOMBI_TOML_FILENAME) || path.ends_with(TOMBI_TOML_FILENAME)) {
        return Ok(Default::default());
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_definition())
        .map(|goto_definition| goto_definition.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }

    let Some(tombi_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(Default::default());
    };

    let mut locations = vec![];

    if features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_definition())
        .and_then(|goto_definition| goto_definition.path())
        .map(|path| path.enabled())
        .unwrap_or_default()
        .value()
    {
        if accessors.last() == Some(&tombi_schema_store::Accessor::Key("path".to_string()))
            && let Some((_, tombi_document_tree_syntax::Value::String(path))) =
                dig_accessors(document_tree, accessors)
            && let Some(uri) = get_definition_link(path.value(), &tombi_toml_path)
        {
            locations.push(tombi_extension::Location { uri, range: None });
        }

        if matches!(accessors.len(), 3 | 4)
            && matches_accessors!(accessors[..3], ["schema", "catalog", "paths"])
            && let Some((_, tombi_document_tree_syntax::Value::Array(paths))) =
                dig_accessors(document_tree, &accessors[..3])
        {
            let index = (accessors.len() == 4)
                .then(|| accessors.last().and_then(|accessor| accessor.as_index()))
                .flatten();

            for (i, path) in paths.iter().enumerate() {
                let tombi_document_tree_syntax::Value::String(path) = path else {
                    continue;
                };
                if index.is_some() && index != Some(i) {
                    continue;
                }
                if let Some(uri) = get_definition_link(path.value(), &tombi_toml_path) {
                    locations.push(tombi_extension::Location { uri, range: None });
                }
            }
        }

        if accessors.first() == Some(&tombi_schema_store::Accessor::Key("extends".to_string()))
            && matches!(accessors.len(), 1 | 2)
            && let Some((_, extends)) = dig_accessors(document_tree, &accessors[..1])
        {
            let references = match extends {
                tombi_document_tree_syntax::Value::String(reference) => vec![reference],
                tombi_document_tree_syntax::Value::Array(references) => {
                    let index = accessors.get(1).and_then(|accessor| accessor.as_index());
                    references
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| index.is_none() || index == Some(*i))
                        .filter_map(|(_, value)| match value {
                            tombi_document_tree_syntax::Value::String(reference) => Some(reference),
                            _ => None,
                        })
                        .collect()
                }
                _ => Vec::new(),
            };

            for reference in references {
                if let Some(uri) = get_extends_link(reference.value(), &tombi_toml_path) {
                    locations.push(tombi_extension::Location { uri, range: None });
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
    } else if let Some(base_dir) = config_base_dir(tombi_toml_path) {
        let mut file_path = std::path::PathBuf::from(url_str);
        if file_path.is_relative() {
            file_path = base_dir.join(file_path);
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

fn get_extends_link(reference: &str, tombi_toml_path: &std::path::Path) -> Option<tombi_uri::Uri> {
    match tombi_config::resolve_local_extends(reference, tombi_toml_path).ok()? {
        ExtendsTarget::File(path) if tombi_fs::is_file(&path) => {
            tombi_uri::Uri::from_file_path(path).ok()
        }
        ExtendsTarget::File(_) => None,
        ExtendsTarget::Remote(uri) => Some(uri),
    }
}
