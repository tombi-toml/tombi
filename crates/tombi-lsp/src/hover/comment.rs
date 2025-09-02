use tombi_comment_directive::{
    document::TombiDocumentDirectiveContent, TombiCommentDirectiveImpl,
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_uri::SchemaUri;

use crate::{
    comment_directive::{
        CommentDirectiveContext, GetCommentDirectiveContext, VALUE_TOMBI_DIRECTIVE_DESCRIPTION,
        VALUE_TOMBI_DIRECTIVE_TITLE,
    },
    handler::get_hover_keys_with_range,
    hover::{get_hover_content, HoverContent, HoverDirectiveContent},
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
};

pub async fn get_document_comment_directive_hover_content(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<HoverContent> {
    if let Some(comment_directive) = root
        .schema_document_comment_directive(source_path)
        .and_then(|comment_directive| comment_directive.get_context(position))
    {
        match comment_directive {
            CommentDirectiveContext::Directive { directive_range } => {
                return Some(HoverContent::Directive(HoverDirectiveContent {
                    title: DOCUMENT_SCHEMA_DIRECTIVE_TITLE.to_string(),
                    description: DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION.to_string(),
                    range: directive_range,
                }));
            }
            CommentDirectiveContext::Content { content_range, .. } => {
                return Some(HoverContent::Directive(HoverDirectiveContent {
                    title: "Schema URL".to_string(),
                    description: "The URL/Path of the schema that applies to this document."
                        .to_string(),
                    range: content_range,
                }))
            }
        }
    }

    match root
        .tombi_document_comment_directives()
        .get_context(position)
    {
        Some(CommentDirectiveContext::Content {
            content,
            content_range,
            position_in_content,
        }) => {
            return get_comment_directive_toml_content_hover_content(
                content,
                content_range,
                position_in_content,
                TombiDocumentDirectiveContent::comment_directive_schema_url(),
            )
            .await;
        }
        Some(CommentDirectiveContext::Directive { directive_range }) => {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: DOCUMENT_TOMBI_DIRECTIVE_TITLE.to_string(),
                description: DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                range: directive_range,
            }));
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
            content_range,
            position_in_content,
        } => {
            get_comment_directive_toml_content_hover_content(
                content,
                content_range,
                position_in_content,
                schema_uri,
            )
            .await
        }
        CommentDirectiveContext::Directive { directive_range } => {
            Some(HoverContent::Directive(HoverDirectiveContent {
                title: VALUE_TOMBI_DIRECTIVE_TITLE.to_string(),
                description: VALUE_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                range: directive_range,
            }))
        }
    }
}

async fn get_comment_directive_toml_content_hover_content(
    content: String,
    content_range: tombi_text::Range,
    position_in_content: tombi_text::Position,
    schema_uri: SchemaUri,
) -> Option<HoverContent> {
    let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
    // Parse the directive content as TOML
    let (directive_ast, _) = tombi_parser::parse(&content, toml_version).into_root_and_errors();

    // Get hover information from the directive AST
    if let Some((keys, range)) =
        get_hover_keys_with_range(&directive_ast, position_in_content, toml_version).await
    {
        // Adjust the range to match the original comment directive position
        let adjusted_range = if let Some(range) = range {
            let mut adjusted = content_range;
            adjusted.start.column += range.start.column;
            adjusted.end.column = content_range.start.column + range.end.column;
            Some(adjusted)
        } else {
            None
        };

        let schema_store = tombi_comment_directive_store::schema_store().await;
        let source_schema = tombi_schema_store::SourceSchema {
            root_schema: Some(comment_directive_document_schema(schema_store, schema_uri).await),
            sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
        };

        let schema_context = tombi_schema_store::SchemaContext {
            toml_version,
            root_schema: source_schema.root_schema.as_ref(),
            sub_schema_uri_map: None,
            store: schema_store,
            strict: None,
        };

        if let Some(hover_content) = get_hover_content(
            &directive_ast
                .into_document_tree_and_errors(toml_version)
                .tree,
            position_in_content,
            &keys,
            &schema_context,
        )
        .await
        {
            return match hover_content {
                HoverContent::Value(mut hover_value_content)
                | HoverContent::DirectiveContent(mut hover_value_content) => {
                    hover_value_content.range = adjusted_range;
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
