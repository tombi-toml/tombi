use crate::extension::IntoLsp as _;
use crate::{
    Backend,
    code_action::{dot_keys_to_inline_table_code_action, inline_table_to_dot_keys_code_action},
    completion::get_completion_keys_with_context,
    config_manager::ConfigSchemaStore,
};
use tombi_document_tree_syntax::get_accessors;
use tombi_schema_store::build_accessor_contexts;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{CodeActionOrCommand, CodeActionParams};

pub async fn handle_code_action(
    backend: &Backend,
    params: CodeActionParams,
) -> Result<Option<Vec<CodeActionOrCommand>>, tower_lsp::jsonrpc::Error> {
    log::trace!("{:?}", params);

    let CodeActionParams {
        text_document,
        range,
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
        .lsp
        .as_ref()
        .and_then(|server| server.code_action.as_ref())
        .and_then(|code_action| code_action.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`server.code_action.enabled` is false");
        return Ok(None);
    }

    log::info!("handle_code_action");

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(None);
    };
    let converter = tombi_extension::SpanConverter::new(
        document_source.line_index(),
        document_source.encoding_kind(),
    );

    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();
    let encoding = document_source.encoding_kind();

    let offset: tombi_text::Offset = range.start.into_lsp(line_index, encoding);

    let Some((keys, key_contexts)) = get_completion_keys_with_context(
        &document_source.ast(),
        document_source.decoded(),
        offset,
        toml_version,
    )
    .await
    else {
        return Ok(None);
    };

    let root = document_source.ast();
    let document_tree = document_source.document_tree();
    let accessors = get_accessors(document_tree, &keys, offset);
    let mut key_contexts = key_contexts.into_iter();
    let accessor_contexts = build_accessor_contexts(&accessors, &mut key_contexts);

    let mut code_actions = Vec::new();

    if let Some(code_action) = dot_keys_to_inline_table_code_action(
        &text_document_uri,
        line_index,
        encoding,
        &root,
        document_tree,
        &accessors,
        &accessor_contexts,
    ) {
        code_actions.push(CodeActionOrCommand::CodeAction(code_action));
    }

    if let Some(code_action) = inline_table_to_dot_keys_code_action(
        &text_document_uri,
        line_index,
        encoding,
        &root,
        document_tree,
        &accessors,
        &accessor_contexts,
    ) {
        code_actions.push(CodeActionOrCommand::CodeAction(code_action));
    }

    if config.cargo_extension_enabled()
        && let Some(extension_code_actions) = tombi_extension_cargo::code_action(
            &text_document_uri,
            converter,
            &root,
            document_tree,
            &accessors,
            &accessor_contexts,
            document_source.toml_version,
            config.cargo_extension_features(),
            schema_store.offline(),
            schema_store.cache_options(),
        )
        .await?
    {
        code_actions.extend(
            extension_code_actions
                .into_iter()
                .map(|action| action.into_lsp_type(line_index, encoding)),
        );
    }

    if config.pyproject_extension_enabled()
        && let Some(extension_code_actions) = tombi_extension_pyproject::code_action(
            &text_document_uri,
            &root,
            document_tree,
            &accessors,
            document_source.toml_version,
            converter,
            config.pyproject_extension_features(),
            schema_store.offline(),
            schema_store.cache_options(),
        )
        .await?
    {
        code_actions.extend(
            extension_code_actions
                .into_iter()
                .map(|action| action.into_lsp_type(line_index, encoding)),
        );
    }

    if code_actions.is_empty() {
        return Ok(None);
    }

    Ok(Some(code_actions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_ast_syntax::AstNode as _;
    use tombi_config::TomlVersion;
    use tombi_parser::parse;
    use tombi_schema_store::AccessorKeyKind;
    use tombi_text::{EncodingKind, Position};

    macro_rules! test_get_completion_keys_with_context {
        (#[tokio::test] async fn $name:ident($src:expr, $pos:expr) -> None;) => {
            #[tokio::test]
            async fn $name() {
                let src = $src.trim();
                let parsed = parse(src);
                let root = parsed.root();
                let decoded = root.decode_strings(TomlVersion::V1_0_0);
                let offset = root
                    .syntax()
                    .line_index()
                    .offset($pos, EncodingKind::GraphemeCluster);
                let result =
                    get_completion_keys_with_context(&root, &decoded, offset, TomlVersion::V1_0_0)
                        .await;

                assert!(result.is_none());
            }
        };

        (#[tokio::test] async fn $name:ident($src:expr, $pos:expr) -> Some(($keys:ident, $contexts:ident) $assertions:block);) => {
            #[tokio::test]
            async fn $name() {
                let src = $src.trim();
                let parsed = parse(src);
                let root = parsed.root();
                let decoded = root.decode_strings(TomlVersion::V1_0_0);
                let offset = root
                    .syntax()
                    .line_index()
                    .offset($pos, EncodingKind::GraphemeCluster);
                let result =
                    get_completion_keys_with_context(&root, &decoded, offset, TomlVersion::V1_0_0)
                        .await;

                assert!(result.is_some());
                let ($keys, $contexts) = result.unwrap();
                $assertions
            }
        };
    }

    test_get_completion_keys_with_context! {
        #[tokio::test]
        async fn test_get_completion_keys_with_context_simple_keyvalue(
            r#"
            foo = 1
            bar = 2
            "#,
            Position::new(0, 2)
        ) -> Some((keys, contexts) {
            assert_eq!(keys.len(), 1);
            assert_eq!(contexts.len(), 1);
            assert_eq!(contexts[0].kind, AccessorKeyKind::KeyValue);
        });
    }

    test_get_completion_keys_with_context! {
        #[tokio::test]
        async fn test_get_completion_keys_with_context_table_header(
            r#"
            [table]
            foo = 1
            "#,
            Position::new(0, 2)
        ) -> Some((keys, contexts) {
            assert!(!keys.is_empty());
            assert!(contexts.iter().any(|c| c.kind == AccessorKeyKind::Header));
        });
    }

    test_get_completion_keys_with_context! {
        #[tokio::test]
        async fn test_get_completion_keys_with_context_empty(
            "# just a comment",
            Position::new(0, 0)
        ) -> None;
    }

    test_get_completion_keys_with_context! {
        #[tokio::test]
        async fn test_get_completion_keys_with_context_simple_keyvalue_span(
            r#"
            foo = 1
            bar = 2
            "#,
            Position::new(0, 2)
        ) -> Some((keys, contexts) {
            pretty_assertions::assert_eq!(keys.len(), 1);
            pretty_assertions::assert_eq!(keys.len(), contexts.len());

            for (key, ctx) in keys.iter().zip(contexts.iter()) {
                pretty_assertions::assert_eq!(ctx.span, key.span());
            }
        });
    }

    test_get_completion_keys_with_context! {
        #[tokio::test]
        async fn test_get_completion_keys_with_context_table_header_span(
            r#"
            [table]
            foo = 1
            "#,
            Position::new(0, 2)
        ) -> Some((keys, contexts) {
            assert!(!keys.is_empty());

            for (key, ctx) in keys.iter().zip(contexts.iter()) {
                pretty_assertions::assert_eq!(ctx.span, key.span());
            }
        });
    }
}
