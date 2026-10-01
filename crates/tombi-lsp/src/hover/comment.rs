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
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
    comment_directive::{
        CommentDirectiveContext, GetCommentDirectiveContext, VALUE_TOMBI_DIRECTIVE_DESCRIPTION,
        VALUE_TOMBI_DIRECTIVE_TITLE,
    },
    handler::get_hover_keys_with_span,
    hover::{HoverContent, HoverDirectiveContent, get_hover_content},
};

pub async fn get_document_comment_directive_hover_content(
    root: &tombi_ast_syntax::Root<'_>,
    offset: tombi_text::Offset,
    source_path: Option<&std::path::Path>,
) -> Option<HoverContent> {
    if let Some(comment_directive) = root
        .schema_document_comment_directive(source_path)
        .and_then(|comment_directive| comment_directive.get_context(offset))
    {
        match comment_directive {
            CommentDirectiveContext::Directive { directive_span } => {
                return Some(HoverContent::Directive(HoverDirectiveContent {
                    title: DOCUMENT_SCHEMA_DIRECTIVE_TITLE.to_string(),
                    description: DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION.to_string(),
                    span: directive_span,
                }));
            }
            CommentDirectiveContext::Content { content_span, .. } => {
                return Some(HoverContent::Directive(HoverDirectiveContent {
                    title: "Schema URL".to_string(),
                    description: "The URL/Path of the schema that applies to this document."
                        .to_string(),
                    span: content_span,
                }));
            }
        }
    }

    match root
        .tombi_document_comment_directives()
        .collect_vec()
        .get_context(offset)
    {
        Some(CommentDirectiveContext::Content {
            content,
            content_span,
            offset_in_content,
        }) => {
            get_comment_directive_toml_content_hover_content(
                content,
                content_span,
                offset_in_content,
                TombiDocumentDirectiveContent::comment_directive_schema_url(),
            )
            .await
        }
        Some(CommentDirectiveContext::Directive { directive_span }) => {
            Some(HoverContent::Directive(HoverDirectiveContent {
                title: DOCUMENT_TOMBI_DIRECTIVE_TITLE.to_string(),
                description: DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                span: directive_span,
            }))
        }
        None => None,
    }
}

pub async fn get_value_comment_directive_hover_content(
    comment_directive_context: CommentDirectiveContext<String>,
    schema_uri: tombi_uri::SchemaUri,
) -> Option<HoverContent> {
    match comment_directive_context {
        CommentDirectiveContext::Content {
            content,
            content_span,
            offset_in_content,
        } => {
            get_comment_directive_toml_content_hover_content(
                content,
                content_span,
                offset_in_content,
                schema_uri,
            )
            .await
        }
        CommentDirectiveContext::Directive { directive_span } => {
            Some(HoverContent::Directive(HoverDirectiveContent {
                title: VALUE_TOMBI_DIRECTIVE_TITLE.to_string(),
                description: VALUE_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                span: directive_span,
            }))
        }
    }
}

async fn get_comment_directive_toml_content_hover_content(
    content: String,
    content_span: tombi_text::Span,
    offset_in_content: tombi_text::Offset,
    schema_uri: SchemaUri,
) -> Option<HoverContent> {
    let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
    // Parse the directive content as TOML
    let parsed = tombi_parser::parse(&content);
    let directive_ast = parsed.root();
    let decoded = directive_ast.decode_strings(toml_version);

    // Get hover information from the directive AST
    if let Some((keys, span)) =
        get_hover_keys_with_span(&directive_ast, &decoded, offset_in_content, toml_version).await
    {
        // Adjust the span to match the original comment directive offset
        let adjusted_span = span.map(|span| span + content_span.start);

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
            toml_version,
            Some(&source_schema),
            schema_store,
            None,
        );

        if let Some(hover_content) = get_hover_content(
            &directive_ast
                .into_document_tree_and_errors(toml_version, &decoded)
                .tree,
            offset_in_content,
            &keys,
            &schema_context,
        )
        .await
        {
            return match hover_content {
                HoverContent::Value(mut hover_value_content)
                | HoverContent::DirectiveContent(mut hover_value_content) => {
                    hover_value_content.span = adjusted_span;
                    Some(HoverContent::DirectiveContent(hover_value_content))
                }
                HoverContent::Directive(hover_content) => {
                    Some(HoverContent::Directive(hover_content))
                }
            };
        }
    }

    None
}
