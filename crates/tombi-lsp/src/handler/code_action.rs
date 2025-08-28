use crate::{
    code_action::{dot_keys_to_inline_table_code_action, inline_table_to_dot_keys_code_action},
    completion::get_completion_keys_with_context,
    config_manager::ConfigSchemaStore,
    Backend,
};
use itertools::Either;
use tombi_document_tree::{get_accessors, TryIntoDocumentTree};
use tombi_schema_store::build_accessor_contexts;
use tower_lsp::lsp_types::{CodeActionOrCommand, CodeActionParams};

pub async fn handle_code_action(
    backend: &Backend,
    params: CodeActionParams,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_code_action");
    tracing::trace!(?params);

    let CodeActionParams {
        text_document,
        range,
        ..
    } = params;

    let text_document_uri = text_document.uri.into();
    let position: tombi_text::Position = range.start.into();

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
        .and_then(|server| server.code_action.as_ref())
        .and_then(|code_action| code_action.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.code_action.enabled` is false");
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

    let tombi_document_comment_directive =
        tombi_validator::comment_directive::get_tombi_document_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            tombi_document_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let Some((keys, key_contexts)) =
        get_completion_keys_with_context(&root, position, toml_version).await
    else {
        return Ok(None);
    };

    let Ok(document_tree) = root.try_into_document_tree(toml_version) else {
        return Ok(None);
    };

    let accessors = get_accessors(&document_tree, &keys, position);
    let mut key_contexts = key_contexts.into_iter();
    let accessor_contexts = build_accessor_contexts(&accessors, &mut key_contexts);

    let mut code_actions = Vec::new();

    if let Some(code_action) = dot_keys_to_inline_table_code_action(
        &text_document_uri,
        &document_tree,
        &accessors,
        &accessor_contexts,
    ) {
        code_actions.push(code_action.into());
    }
    if let Some(code_action) = inline_table_to_dot_keys_code_action(
        &text_document_uri,
        &document_tree,
        &accessors,
        &accessor_contexts,
    ) {
        code_actions.push(code_action.into());
    }

    if let Some(extension_code_actions) = tombi_extension_cargo::code_action(
        &text_document_uri,
        &document_tree,
        &accessors,
        &accessor_contexts,
        toml_version,
    )? {
        code_actions.extend(extension_code_actions);
    }

    if code_actions.is_empty() {
        return Ok(None);
    }

    Ok(Some(code_actions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_ast::AstNode;
    use tombi_config::TomlVersion;
    use tombi_parser::parse;
    use tombi_schema_store::AccessorKeyKind;
    use tombi_text::Position;

    #[tokio::test]
    async fn test_get_completion_keys_with_context_simple_keyvalue() {
        let src = r#"foo = 1\nbar = 2\n"#;
        let root =
            tombi_ast::Root::cast(parse(src, TomlVersion::V1_0_0).into_syntax_node()).unwrap();
        let pos = Position::new(0, 2); // somewhere in 'foo'
        let toml_version = TomlVersion::V1_0_0;
        let result = get_completion_keys_with_context(&root, pos, toml_version).await;
        assert!(result.is_some());
        let (keys, contexts) = result.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].kind, AccessorKeyKind::KeyValue);
    }

    #[tokio::test]
    async fn test_get_completion_keys_with_context_table_header() {
        let src = r#"[table]\nfoo = 1\n"#;
        let root =
            tombi_ast::Root::cast(parse(src, TomlVersion::V1_0_0).into_syntax_node()).unwrap();
        let pos = Position::new(0, 2); // somewhere in 'table'
        let toml_version = TomlVersion::V1_0_0;
        let result = get_completion_keys_with_context(&root, pos, toml_version).await;
        assert!(result.is_some());
        let (keys, contexts) = result.unwrap();
        assert!(!keys.is_empty());

        assert!(contexts.iter().any(|c| c.kind == AccessorKeyKind::Header));
    }

    #[tokio::test]
    async fn test_get_completion_keys_with_context_empty() {
        let src = r#"# just a comment\n"#;
        let root =
            tombi_ast::Root::cast(parse(src, TomlVersion::V1_0_0).into_syntax_node()).unwrap();
        let pos = Position::new(0, 0);
        let toml_version = TomlVersion::V1_0_0;
        let result = get_completion_keys_with_context(&root, pos, toml_version).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_completion_keys_with_context_simple_keyvalue_range() {
        let src = "foo = 1\nbar = 2\n";
        let root =
            tombi_ast::Root::cast(parse(src, TomlVersion::V1_0_0).into_syntax_node()).unwrap();
        let pos = Position::new(0, 2); // somewhere in 'foo'
        let toml_version = TomlVersion::V1_0_0;
        let result = get_completion_keys_with_context(&root, pos, toml_version).await;
        assert!(result.is_some());
        let (keys, contexts) = result.unwrap();

        pretty_assertions::assert_eq!(keys.len(), 1);
        pretty_assertions::assert_eq!(keys.len(), contexts.len());

        for (key, ctx) in keys.iter().zip(contexts.iter()) {
            pretty_assertions::assert_eq!(ctx.range, key.range());
        }
    }

    #[tokio::test]
    async fn test_get_completion_keys_with_context_table_header_range() {
        let src = "[table]\nfoo = 1\n";
        let root =
            tombi_ast::Root::cast(parse(src, TomlVersion::V1_0_0).into_syntax_node()).unwrap();
        let pos = Position::new(0, 2); // somewhere in 'table'
        let toml_version = TomlVersion::V1_0_0;
        let result = get_completion_keys_with_context(&root, pos, toml_version).await;
        assert!(result.is_some());
        let (keys, contexts) = result.unwrap();
        assert!(!keys.is_empty());

        for (key, ctx) in keys.iter().zip(contexts.iter()) {
            pretty_assertions::assert_eq!(ctx.range, key.range());
        }
    }
}
