use itertools::Itertools;
use tombi_ast_syntax::AstNode as _;
use tombi_comment_directive::{
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, TombiCommentDirectiveImpl,
    document::TombiDocumentDirectiveContent,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_document_tree_syntax::IntoDocumentTreeAndErrors;
use tombi_uri::SchemaUri;

use crate::{
    comment_directive::{CommentDirectiveContext, GetCommentDirectiveContext},
    goto_type_definition::{TypeDefinition, get_type_definition},
    handler::get_hover_keys_with_span,
};
pub async fn get_tombi_document_comment_directive_type_definition(
    root: &tombi_ast_syntax::Root<'_>,
    offset: tombi_text::Offset,
) -> Vec<TypeDefinition> {
    if let Some(comment_directive_context) = root
        .tombi_document_comment_directives()
        .collect_vec()
        .get_context(offset)
    {
        get_tombi_value_comment_directive_type_definition(
            comment_directive_context,
            TombiDocumentDirectiveContent::comment_directive_schema_url(),
        )
        .await
    } else {
        Vec::new()
    }
}

pub async fn get_tombi_value_comment_directive_type_definition(
    comment_directive_context: CommentDirectiveContext<String>,
    schema_uri: SchemaUri,
) -> Vec<TypeDefinition> {
    let CommentDirectiveContext::Content {
        content,
        offset_in_content,
        ..
    } = comment_directive_context
    else {
        return Vec::new();
    };

    let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
    let parsed = tombi_parser::parse(&content);
    let root = parsed.root();
    let decoded = root.decode_strings(toml_version);

    let Some((keys, range)) =
        get_hover_keys_with_span(&root, &decoded, offset_in_content, toml_version).await
    else {
        return Vec::new();
    };

    if keys.is_empty() && range.is_none() {
        return Vec::new();
    }

    // The positions are in the content of the directive, which is parsed as a separate document.
    let line_index = parsed.line_index();
    let document_tree = root
        .into_document_tree_and_errors(toml_version, &decoded)
        .tree;

    let schema_store = tombi_comment_directive_store::schema_store().await;
    let source_schema = tombi_schema_store::SourceSchema::new(
        Some(comment_directive_document_schema(schema_store, schema_uri).await),
        tombi_hashmap::IndexMap::with_capacity(0),
        Some(toml_version),
        None,
        Default::default(),
        Default::default(),
        Default::default(),
    );

    let schema_context = tombi_schema_store::SchemaContext::from_source_schema(
        TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
        Some(&source_schema),
        schema_store,
        None,
    );

    get_type_definition(
        &document_tree,
        crate::CursorPosition::new(offset_in_content, line_index),
        &keys,
        &schema_context,
    )
    .await
}
