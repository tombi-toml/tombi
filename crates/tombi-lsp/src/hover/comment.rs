use tombi_document_tree::IntoDocumentTreeAndErrors;

use crate::{
    comment_directive::{
        get_schema_document_comment_directive, get_tombi_document_comment_directive,
        DocumentTombiDirective, DocumentTombiDirectiveContent, TombiDocumentCommentDirective,
    },
    handler::get_hover_keys_with_range,
    hover::{get_hover_content, HoverContent, HoverDirectiveContent},
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
};

pub async fn get_comment_directive_hover_info(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<HoverContent> {
    if let Some(schema_comment_directive) =
        get_schema_document_comment_directive(root, position, source_path)
    {
        if schema_comment_directive.directive_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: DOCUMENT_SCHEMA_DIRECTIVE_TITLE.to_string(),
                description: DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION.to_string(),
                range: schema_comment_directive.directive_range,
            }));
        }
        if schema_comment_directive.uri_range.contains(position) {
            return Some(HoverContent::Directive(HoverDirectiveContent {
                title: "Schema URL".to_string(),
                description: "The URL/Path of the schema that applies to this document."
                    .to_string(),
                range: schema_comment_directive.uri_range,
            }));
        }
        return None;
    }

    // Check if position is in a #:tombi comment directive
    if let Some(comments) = root.get_document_header_comments() {
        for comment in comments {
            if let Some(comment_directive) =
                get_tombi_document_comment_directive(&comment, position)
            {
                match comment_directive {
                    TombiDocumentCommentDirective::Directive(DocumentTombiDirective {
                        directive_range,
                    }) => {
                        if directive_range.contains(position) {
                            return Some(HoverContent::Directive(HoverDirectiveContent {
                                title: DOCUMENT_TOMBI_DIRECTIVE_TITLE.to_string(),
                                description: DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION.to_string(),
                                range: directive_range,
                            }));
                        }
                        return None;
                    }
                    TombiDocumentCommentDirective::Content(DocumentTombiDirectiveContent {
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
                                tombi_comment_directive::document_comment_directive_document_schema()
                                    .await;

                            let schema_store = tombi_comment_directive::schema_store().await;
                            // Try to use the source schema if available, otherwise fall back to tombi schema
                            let schema_context = tombi_schema_store::SchemaContext {
                                toml_version,
                                root_schema: Some(&document_schema),
                                sub_schema_uri_map: None,
                                store: schema_store,
                                strict: None,
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
