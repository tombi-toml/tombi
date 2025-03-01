use ast::{algo::ancestors_at_position, AstNode};
use document_tree::{IntoDocumentTreeAndErrors, TryIntoDocumentTree};
use itertools::Itertools;
use schema_store::SchemaContext;
use tower_lsp::lsp_types::{HoverParams, TextDocumentPositionParams};

use crate::{
    backend,
    hover::{get_hover_content, HoverContent},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_hover(
    backend: &backend::Backend,
    HoverParams {
        text_document_position_params:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        ..
    }: HoverParams,
) -> Result<Option<HoverContent>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_hover");
    tracing::trace!("text_document: {:#?}", text_document);

    let config = backend.config().await;

    if !config
        .server
        .and_then(|server| server.hover)
        .and_then(|hover| hover.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.hover.enabled` is false");
        return Ok(None);
    }

    let position = position.into();
    let toml_version = backend.toml_version().await.unwrap_or_default();

    let source_schema = backend
        .schema_store
        .try_get_source_schema_from_url(&text_document.uri)
        .await
        .ok()
        .flatten();

    let Some(root) = backend
        .get_incomplete_ast(&text_document.uri, toml_version)
        .await
    else {
        return Ok(None);
    };

    let Some((keys, range)) = get_hover_range(&root, position, toml_version).await else {
        return Ok(None);
    };

    if keys.is_empty() {
        return Ok(None);
    }

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    return Ok(get_hover_content(
        &document_tree,
        position,
        &keys,
        &SchemaContext {
            toml_version,
            root_schema: source_schema.as_ref().and_then(|s| s.root_schema),
            sub_schema_url_map: source_schema.as_ref().map(|s| &s.sub_schema_url_map),
            store: &backend.schema_store,
        },
    )
    .await
    .map(|mut content| {
        content.range = range;
        content
    }));
}

async fn get_hover_range(
    root: &ast::Root,
    position: text::Position,
    toml_version: config::TomlVersion,
) -> Option<(Vec<document_tree::Key>, Option<text::Range>)> {
    let mut keys_vec = vec![];
    let mut hover_range = None;

    for node in ancestors_at_position(root.syntax(), position) {
        if let Some(array) = ast::Array::cast(node.to_owned()) {
            for (value, comma) in array.values_with_comma() {
                if hover_range.is_none() {
                    let mut range = value.range();
                    if let Some(comma) = comma {
                        range += comma.range()
                    };
                    if range.contains(position) {
                        hover_range = Some(range);
                    }
                }
            }
        };

        let keys = if let Some(kv) = ast::KeyValue::cast(node.to_owned()) {
            if hover_range.is_none() {
                if let Some(inline_table) = ast::InlineTable::cast(node.parent().unwrap()) {
                    for (key_value, comma) in inline_table.key_values_with_comma() {
                        if hover_range.is_none() {
                            let mut range = key_value.range();
                            if let Some(comma) = comma {
                                range += comma.range()
                            };
                            if range.contains(position) {
                                hover_range = Some(range);
                                break;
                            }
                        }
                    }
                } else {
                    hover_range = Some(kv.range());
                }
            }
            kv.keys().unwrap()
        } else if let Some(table) = ast::Table::cast(node.to_owned()) {
            table.header().unwrap()
        } else if let Some(array_of_tables) = ast::ArrayOfTables::cast(node.to_owned()) {
            array_of_tables.header().unwrap()
        } else {
            continue;
        };

        let keys = if keys.range().contains(position) {
            let mut new_keys = Vec::with_capacity(keys.keys().count());
            for key in keys
                .keys()
                .take_while(|key| key.token().unwrap().range().start() <= position)
            {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key)) => new_keys.push(key),
                    _ => return None,
                }
            }
            new_keys
        } else {
            let mut new_keys = Vec::with_capacity(keys.keys().count());
            for key in keys.keys() {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key)) => new_keys.push(key),
                    _ => return None,
                }
            }
            new_keys
        };

        if hover_range.is_none() {
            hover_range = keys.iter().map(|key| key.range()).reduce(|k1, k2| k1 + k2);
        }

        keys_vec.push(keys);
    }

    Some((
        keys_vec.into_iter().rev().flatten().collect_vec(),
        hover_range,
    ))
}
