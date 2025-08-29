use tombi_ast::TombiValueCommentDirective;
use tombi_comment_directive::{TombiCommentDirectiveImpl, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_document_tree::IntoDocumentTreeAndErrors;

use crate::{
    comment_directive::{CommentDirectiveContext, GetCommentDirectiveContext},
    goto_type_definition::{get_type_definition, TypeDefinition},
    handler::get_hover_keys_with_range,
};

pub async fn get_tombi_value_comment_directive_type_definition<CommentDirective>(
    comment_directive: &TombiValueCommentDirective,
    position: tombi_text::Position,
) -> Option<TypeDefinition>
where
    CommentDirective: TombiCommentDirectiveImpl,
{
    let Some(CommentDirectiveContext::Content {
        content,
        position_in_content,
        ..
    }) = comment_directive.get_context(position)
    else {
        return None;
    };

    let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
    let (root, _) = tombi_parser::parse(&content, toml_version).into_root_and_errors();

    let Some((keys, range)) =
        get_hover_keys_with_range(&root, position_in_content, toml_version).await
    else {
        return None;
    };

    if keys.is_empty() && range.is_none() {
        return None;
    }

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    let schema_store = tombi_comment_directive_store::schema_store().await;
    let source_schema = tombi_schema_store::SourceSchema {
        root_schema: Some(
            comment_directive_document_schema(
                schema_store,
                CommentDirective::comment_directive_schema_url(),
            )
            .await,
        ),
        sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
    };

    let schema_context = tombi_schema_store::SchemaContext {
        toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
        root_schema: source_schema.root_schema.as_ref(),
        sub_schema_uri_map: None,
        store: schema_store,
        strict: None,
    };

    get_type_definition(&document_tree, position, &keys, &schema_context).await
}
