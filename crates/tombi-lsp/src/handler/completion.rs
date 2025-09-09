use itertools::Either;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_extension::{CommentContext, CompletionContent, CompletionHint};
use tower_lsp::lsp_types::{
    CompletionContext, CompletionParams, CompletionTriggerKind, TextDocumentPositionParams,
};

use crate::{
    backend,
    completion::{
        extract_keys_and_hint, find_completion_contents_with_tree, get_comment_context,
        get_document_comment_directive_completion_contents,
    },
    config_manager::ConfigSchemaStore,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_completion(
    backend: &backend::Backend,
    params: CompletionParams,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_completion");
    tracing::trace!(?params);

    let CompletionParams {
        text_document_position:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        context,
        ..
    } = params;

    let text_document_uri = text_document.uri.into();

    let ConfigSchemaStore {
        config,
        schema_store,
        ..
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.completion.as_ref())
        .and_then(|completion| completion.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.completion.enabled` is false");
        return Ok(None);
    }

    if !config
        .schema
        .as_ref()
        .and_then(|s| s.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`schema.enabled` is false");
        return Ok(None);
    }

    let Some(root) = backend.get_incomplete_ast(&text_document_uri).await else {
        return Ok(None);
    };

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document_uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend
        .source_toml_version(
            tombi_validator::comment_directive::get_tombi_document_comment_directive(&root).await,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let document_sources = backend.document_sources.read().await;
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        tracing::trace!("document_source not found");
        return Ok(None);
    };

    let root_schema = source_schema
        .as_ref()
        .and_then(|schema| schema.root_schema.as_ref());

    // Skip completion if the trigger character is a whitespace or if there is no schema.
    if let Some(CompletionContext {
        trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
        trigger_character: Some(trigger_character),
        ..
    }) = context
    {
        if trigger_character == "\n" {
            let pos_line = position.line as usize;
            if pos_line > 0 {
                if let Some(prev_line) = &document_source.text.lines().nth(pos_line - 1) {
                    if prev_line.trim().is_empty() || root_schema.is_none() {
                        tracing::trace!("completion skipped due to consecutive line breaks");
                        return Ok(None);
                    }
                }
            }
        }
    }

    let mut completion_items = Vec::new();
    let position = position.into();

    let comment_context = get_comment_context(&root, position);
    let (document_tree, keys, completion_hint) = match &comment_context {
        Some(CommentContext::DocumentDirective(comment)) => {
            if let Some(comment_completion_contents) =
                get_document_comment_directive_completion_contents(
                    &root,
                    comment,
                    position,
                    &text_document_uri,
                )
                .await
            {
                return Ok(Some(comment_completion_contents));
            }
            let Some((document_tree, keys, completion_hint)) =
                get_document_tree_and_keys_and_completion_hint(
                    root,
                    position,
                    toml_version,
                    comment_context.as_ref(),
                )
            else {
                return Ok(Some(Vec::with_capacity(0)));
            };

            (document_tree, keys, completion_hint)
        }
        Some(CommentContext::ValueDirective(_)) | None => {
            let Some((document_tree, keys, completion_hint)) =
                get_document_tree_and_keys_and_completion_hint(
                    root,
                    position,
                    toml_version,
                    comment_context.as_ref(),
                )
            else {
                return Ok(Some(Vec::with_capacity(0)));
            };

            let schema_context = tombi_schema_store::SchemaContext {
                toml_version,
                root_schema,
                sub_schema_uri_map: source_schema
                    .as_ref()
                    .map(|schema| &schema.sub_schema_uri_map),
                store: &schema_store,
                strict: None,
            };

            completion_items.extend(
                find_completion_contents_with_tree(
                    &document_tree,
                    position,
                    &keys,
                    &schema_context,
                    completion_hint,
                )
                .await,
            );

            (document_tree, keys, completion_hint)
        }
        Some(CommentContext::Normal(_)) => {
            let Some((document_tree, keys, completion_hint)) =
                get_document_tree_and_keys_and_completion_hint(
                    root,
                    position,
                    toml_version,
                    comment_context.as_ref(),
                )
            else {
                return Ok(Some(Vec::with_capacity(0)));
            };

            (document_tree, keys, completion_hint)
        }
    };

    let accessors = tombi_document_tree::get_accessors(&document_tree, &keys, position);
    if let Some(items) = tombi_extension_cargo::completion(
        &text_document_uri,
        &document_tree,
        position,
        &accessors,
        toml_version,
        completion_hint,
        comment_context.as_ref(),
    )
    .await?
    {
        completion_items.extend(items);
    }

    Ok(Some(completion_items))
}

fn get_document_tree_and_keys_and_completion_hint(
    root: tombi_ast::Root,
    position: tombi_text::Position,
    toml_version: tombi_config::TomlVersion,
    comment_context: Option<&CommentContext>,
) -> Option<(
    tombi_document_tree::DocumentTree,
    Vec<tombi_document_tree::Key>,
    Option<CompletionHint>,
)> {
    let Some((keys, completion_hint)) =
        extract_keys_and_hint(&root, position, toml_version, comment_context)
    else {
        tracing::trace!("keys and completion_hint not found");
        return None;
    };
    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    Some((document_tree, keys, completion_hint))
}
