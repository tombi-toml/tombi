use tombi_ast::{
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective, TombiValueCommentDirective,
};
use tombi_comment_directive::{TombiCommentDirectiveImpl, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION};
use tombi_comment_directive_store::{
    comment_directive_document_schema, document_comment_directive_schema_uri,
};
use tombi_document_tree::IntoDocumentTreeAndErrors;

use crate::{
    comment_directive::{
        get_schema_document_comment_directive, get_tombi_document_comment_directive,
        VALUE_TOMBI_DIRECTIVE_DESCRIPTION, VALUE_TOMBI_DIRECTIVE_TITLE,
    },
    handler::get_hover_keys_with_range,
    hover::{get_hover_content, HoverContent, HoverDirectiveContent},
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
};

pub async fn get_document_comment_directive_hover_info(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<HoverContent> {
    if let Some(SchemaDocumentCommentDirective {
        directive_range,
        uri_range,
        ..
    }) = get_schema_document_comment_directive(root, position, source_path)
    {
        if directive_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: DOCUMENT_SCHEMA_DIRECTIVE_TITLE.to_string(),
                description: DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION.to_string(),
                range: directive_range,
            }));
        }
        if uri_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: "Schema URL".to_string(),
                description: "The URL/Path of the schema that applies to this document."
                    .to_string(),
                range: uri_range,
            }));
        }
        return None;
    }
    if let Some(TombiDocumentCommentDirective {
        directive_range,
        content,
        content_range,
    }) = get_tombi_document_comment_directive(root, position)
    {
        if directive_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: DOCUMENT_TOMBI_DIRECTIVE_TITLE.to_string(),
                description: DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                range: directive_range,
            }));
        }
        if content_range.contains(position) {
            let toml_version = tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
            // Parse the directive content as TOML
            let (directive_ast, _) =
                tombi_parser::parse(&content, toml_version).into_root_and_errors();
            let position_in_content =
                tombi_text::Position::new(0, position.column - (directive_range.end.column + 1));

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
                let document_schema = comment_directive_document_schema(
                    schema_store,
                    document_comment_directive_schema_uri(),
                )
                .await;
                // Try to use the source schema if available, otherwise fall back to tombi schema
                let schema_context = tombi_schema_store::SchemaContext {
                    toml_version,
                    root_schema: Some(&document_schema),
                    sub_schema_uri_map: None,
                    store: schema_store,
                    strict: None,
                };

                let mut hover_content = get_hover_content(
                    &directive_ast
                        .into_document_tree_and_errors(toml_version)
                        .tree,
                    position_in_content,
                    &keys,
                    &schema_context,
                )
                .await;

                if let Some(HoverContent::Value(hover_value_content)) = hover_content.as_mut() {
                    hover_value_content.range = adjusted_range;
                }
                return hover_content;
            }
        }
    }

    None
}

pub async fn get_value_comment_directive_hover_info<CommentDirective>(
    comment_directive: &TombiValueCommentDirective,
    position: tombi_text::Position,
) -> Option<HoverContent>
where
    CommentDirective: TombiCommentDirectiveImpl,
{
    let TombiValueCommentDirective {
        directive_range,
        content,
        content_range,
    } = comment_directive;
    if directive_range.contains(position) {
        return Some(HoverContent::Directive(HoverDirectiveContent {
            title: VALUE_TOMBI_DIRECTIVE_TITLE.to_string(),
            description: VALUE_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
            range: *directive_range,
        }));
    }

    if content_range.contains(position) {
        let toml_version = tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
        // Parse the directive content as TOML
        let (directive_ast, _) = tombi_parser::parse(&content, toml_version).into_root_and_errors();
        let position_in_content =
            tombi_text::Position::new(0, position.column - (directive_range.end.column + 1));

        // Get hover information from the directive AST
        if let Some((keys, range)) =
            get_hover_keys_with_range(&directive_ast, position_in_content, toml_version).await
        {
            // Adjust the range to match the original comment directive position
            let adjusted_range = if let Some(range) = range {
                let mut adjusted = *content_range;
                adjusted.start.column += range.start.column;
                adjusted.end.column = content_range.start.column + range.end.column;
                Some(adjusted)
            } else {
                None
            };

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

            let mut hover_content = get_hover_content(
                &directive_ast
                    .into_document_tree_and_errors(toml_version)
                    .tree,
                position_in_content,
                &keys,
                &schema_context,
            )
            .await;

            if let Some(HoverContent::Value(hover_value_content)) = hover_content.as_mut() {
                hover_value_content.range = adjusted_range.map(|r| r.to_owned());
            }

            return hover_content;
        }
    }
    None
}
