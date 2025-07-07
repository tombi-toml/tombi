use itertools::Either;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_extension::CompletionContent;
use tombi_schema_store::get_accessors;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionParams, CompletionTriggerKind, TextDocumentPositionParams,
};

use crate::{
    backend,
    completion::{
        extract_keys_and_hint, find_completion_contents_with_tree, get_comment_completion_contents,
    },
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

    let config = backend.config().await;

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
        .and_then(|s| s.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`schema.enabled` is false");
        return Ok(None);
    }

    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let source_schema = backend
        .schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend.source_toml_version(source_schema.as_ref()).await;

    let document_sources = backend.document_sources.read().await;
    let Some(document_source) = document_sources.get(&text_document.uri) else {
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

    if let Some(comment_completion_contents) =
        get_comment_completion_contents(&root, position, &text_document.uri)
    {
        return Ok(Some(comment_completion_contents));
    }

    let Some((keys, completion_hint)) = extract_keys_and_hint(&root, position, toml_version) else {
        return Ok(Some(Vec::with_capacity(0)));
    };
    let document_tree = root.into_document_tree_and_errors(toml_version).tree;
    let schema_context = tombi_schema_store::SchemaContext {
        toml_version,
        root_schema,
        sub_schema_url_map: source_schema
            .as_ref()
            .map(|schema| &schema.sub_schema_url_map),
        store: &backend.schema_store,
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

    let accessors = get_accessors(&document_tree, &keys, position);
    if let Some(items) = tombi_extension_cargo::completion(
        &text_document,
        &document_tree,
        position,
        &accessors,
        toml_version,
        completion_hint,
    )
    .await?
    {
        completion_items.extend(items);
    }

    Ok(Some(completion_items))
}
