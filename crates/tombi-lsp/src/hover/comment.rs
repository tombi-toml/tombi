use tombi_document_tree::IntoDocumentTreeAndErrors;

use crate::{
    comment_directive::{
        get_schema_comment_directive, get_tombi_comment_directive, TombiCommentDirective,
        TombiDirective, TombiDirectiveContent,
    },
    handler::get_hover_keys_with_range,
    hover::{get_hover_content, HoverContent, HoverDirectiveContent},
    SCHEMA_DIRECTIVE_DESCRIPTION, SCHEMA_DIRECTIVE_TITLE, TOMBI_DIRECTIVE_DESCRIPTION,
    TOMBI_DIRECTIVE_TITLE,
};

pub async fn get_comment_directive_hover_info(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<HoverContent> {
    if let Some(schema_comment_directive) = get_schema_comment_directive(&root, position) {
        if schema_comment_directive.directive_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: SCHEMA_DIRECTIVE_TITLE.to_string(),
                description: SCHEMA_DIRECTIVE_DESCRIPTION.to_string(),
                range: schema_comment_directive.directive_range,
            }));
        }
        if schema_comment_directive.url_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: "Schema URL".to_string(),
                description: "The URL/Path of the schema that applies to this document."
                    .to_string(),
                range: schema_comment_directive.url_range,
            }));
        }
        return None;
    }

    // Check if position is in a #:tombi comment directive
    if let Some(comments) = root.get_document_header_comments() {
        for comment in comments {
            if let Some(comment_directive) = get_tombi_comment_directive(&comment, position) {
                match comment_directive {
                    TombiCommentDirective::Directive(TombiDirective { directive_range }) => {
                        if directive_range.contains(position) {
                            return Some(HoverContent::Directive(HoverDirectiveContent {
                                title: TOMBI_DIRECTIVE_TITLE.to_string(),
                                description: TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                                range: directive_range,
                            }));
                        }
                        return None;
                    }
                    TombiCommentDirective::Content(TombiDirectiveContent {
                        content,
                        position_in_content,
                        content_range,
                    }) => {
                        let toml_version =
                            tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
                        // Parse the directive content as TOML
                        let (directive_ast, _) =
                            tombi_parser::parse(&content, toml_version).into_root_and_errors();

                        // Get hover information from the directive AST
                        if let Some((keys, range)) = get_hover_keys_with_range(
                            &directive_ast,
                            position_in_content,
                            toml_version,
                        )
                        .await
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

                            let document_schema =
                                tombi_comment_directive::root_comment_directive_document_schema()
                                    .await;

                            let schema_store = tombi_comment_directive::schema_store().await;
                            // Try to use the source schema if available, otherwise fall back to tombi schema
                            let schema_context = tombi_schema_store::SchemaContext {
                                toml_version,
                                root_schema: Some(&document_schema),
                                sub_schema_url_map: None,
                                store: &schema_store,
                            };

                            return get_hover_content(
                                &directive_ast
                                    .into_document_tree_and_errors(toml_version)
                                    .tree,
                                position_in_content,
                                &keys,
                                &schema_context,
                            )
                            .await
                            .map(|mut content| {
                                content.range = adjusted_range;
                                HoverContent::Value(content)
                            });
                        }
                    }
                }
            }
        }
    }
    None
}
