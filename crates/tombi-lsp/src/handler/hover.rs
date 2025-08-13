use itertools::{Either, Itertools};
use tombi_ast::{algo::ancestors_at_position, AstNode};
use tombi_document_tree::{IntoDocumentTreeAndErrors, TryIntoDocumentTree};
use tombi_schema_store::SchemaContext;
use tower_lsp::lsp_types::{HoverParams, TextDocumentPositionParams};

use crate::{
    backend,
    config_manager::ConfigSchemaStore,
    hover::{get_comment_directive_hover_info, get_hover_content, HoverContent},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_hover(
    backend: &backend::Backend,
    params: HoverParams,
) -> Result<Option<HoverContent>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_hover");
    tracing::trace!(?params);

    let HoverParams {
        text_document_position_params:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        ..
    } = params;

    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(&text_document.uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.hover.as_ref())
        .and_then(|hover| hover.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.hover.enabled` is false");
        return Ok(None);
    }

    let position = position.into();
    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let document_tombi_comment_directive =
        tombi_comment_directive::get_document_tombi_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            document_tombi_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let source_path = text_document.uri.to_file_path().ok();

    // Check if position is in a #:tombi comment directive
    if let Some(content) =
        get_comment_directive_hover_info(&root, position, source_path.as_deref()).await
    {
        return Ok(Some(content));
    }

    let Some((keys, range)) = get_hover_keys_with_range(&root, position, toml_version).await else {
        return Ok(None);
    };

    if keys.is_empty() && range.is_none() {
        return Ok(None);
    }

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    return Ok(get_hover_content(
        &document_tree,
        position,
        &keys,
        &SchemaContext {
            toml_version,
            root_schema: source_schema.as_ref().and_then(|s| s.root_schema.as_ref()),
            sub_schema_url_map: source_schema.as_ref().map(|s| &s.sub_schema_url_map),
            store: &schema_store,
        },
    )
    .await
    .map(|mut content| {
        content.range = range;
        HoverContent::Value(content)
    }));
}

pub async fn get_hover_keys_with_range(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    toml_version: tombi_config::TomlVersion,
) -> Option<(Vec<tombi_document_tree::Key>, Option<tombi_text::Range>)> {
    let mut keys_vec = vec![];
    let mut hover_range = None;

    for node in ancestors_at_position(root.syntax(), position) {
        if let Some(array) = tombi_ast::Array::cast(node.to_owned()) {
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
        } else if let Some(inline_table) = tombi_ast::InlineTable::cast(node.to_owned()) {
            for (key_value, comma) in inline_table.key_values_with_comma() {
                if hover_range.is_none() {
                    let mut range = key_value.range();
                    if let Some(comma) = comma {
                        range += comma.range()
                    };
                    if range.contains(position) {
                        hover_range = Some(range);
                    }
                }
            }
        };

        let keys = if let Some(kv) = tombi_ast::KeyValue::cast(node.to_owned()) {
            if hover_range.is_none() {
                if let Some(inline_table) = tombi_ast::InlineTable::cast(node.parent().unwrap()) {
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
            kv.keys()
        } else if let Some(table) = tombi_ast::Table::cast(node.to_owned()) {
            let header = table.header();
            if let Some(header) = &header {
                if hover_range.is_none()
                    && (header
                        .keys()
                        .last()
                        .is_none_or(|key| key.syntax().range().contains(position))
                        || table
                            .leading_comments()
                            .any(|comment| comment.syntax().range().contains(position))
                        || table
                            .tailing_comment()
                            .is_some_and(|comment| comment.syntax().range().contains(position))
                        || table
                            .key_values_begin_dangling_comments()
                            .into_iter()
                            .any(|comments| {
                                comments
                                    .into_iter()
                                    .any(|comment| comment.syntax().range().contains(position))
                            })
                        || table
                            .key_values_end_dangling_comments()
                            .into_iter()
                            .any(|comments| {
                                comments
                                    .into_iter()
                                    .any(|comment| comment.syntax().range().contains(position))
                            }))
                {
                    let mut range = table.syntax().range();
                    if let Some(max_end) = table
                        .subtables()
                        .map(|subtable| subtable.syntax().range().end)
                        .max()
                    {
                        range.end = max_end;
                    }
                    hover_range = Some(range);
                }
            }

            header
        } else if let Some(array_of_table) = tombi_ast::ArrayOfTable::cast(node.to_owned()) {
            let header = array_of_table.header();
            if let Some(header) = &header {
                if hover_range.is_none()
                    && (header
                        .keys()
                        .last()
                        .is_none_or(|key| key.syntax().range().contains(position))
                        || array_of_table
                            .leading_comments()
                            .any(|comment| comment.syntax().range().contains(position))
                        || array_of_table
                            .tailing_comment()
                            .is_some_and(|comment| comment.syntax().range().contains(position))
                        || array_of_table
                            .key_values_begin_dangling_comments()
                            .into_iter()
                            .any(|comments| {
                                comments
                                    .into_iter()
                                    .any(|comment| comment.syntax().range().contains(position))
                            })
                        || array_of_table
                            .key_values_end_dangling_comments()
                            .into_iter()
                            .any(|comments| {
                                comments
                                    .into_iter()
                                    .any(|comment| comment.syntax().range().contains(position))
                            }))
                {
                    let mut range = array_of_table.syntax().range();
                    if let Some(max_end) = array_of_table
                        .subtables()
                        .map(|subtable| subtable.syntax().range().end)
                        .max()
                    {
                        range.end = max_end;
                    }
                    hover_range = Some(range);
                }
            }
            header
        } else if let Some(root) = tombi_ast::Root::cast(node.to_owned()) {
            if hover_range.is_none()
                && (root
                    .key_values_begin_dangling_comments()
                    .into_iter()
                    .any(|comments| {
                        comments
                            .into_iter()
                            .any(|comment| comment.syntax().range().contains(position))
                    })
                    || root
                        .key_values_end_dangling_comments()
                        .into_iter()
                        .any(|comments| {
                            comments
                                .into_iter()
                                .any(|comment| comment.syntax().range().contains(position))
                        }))
            {
                hover_range = Some(root.syntax().range());
            }
            continue;
        } else {
            continue;
        };

        let Some(keys) = keys else { continue };

        let keys = if keys.range().contains(position) {
            let mut new_keys = Vec::with_capacity(keys.keys().count());
            for key in keys
                .keys()
                .take_while(|key| key.token().unwrap().range().start <= position)
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
