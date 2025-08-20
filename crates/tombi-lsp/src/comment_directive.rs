use tombi_ast::SchemaDocumentCommentDirective;

pub const DOCUMENT_SCHEMA_DIRECTIVE_TITLE: &str = "DocumentSchema Directive";
pub const DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION: &str =
    "Specify the Schema URL/Path for the document.";

pub const DOCUMENT_TOMBI_DIRECTIVE_TITLE: &str = "Document Tombi Directive";
pub const DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION: &str =
    "Directives that apply only to this document.";

#[derive(Debug)]
pub enum TombiDocumentCommentDirective {
    Directive(DocumentTombiDirective),
    Content(DocumentTombiDirectiveContent),
}

#[derive(Debug)]
pub struct DocumentTombiDirective {
    pub directive_range: tombi_text::Range,
}

#[derive(Debug)]
pub struct DocumentTombiDirectiveContent {
    /// Directive content.
    ///
    /// ```tombi
    /// #:tombi toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^ <- This content.
    /// ```
    pub content: String,

    /// Position based on the directive content.
    ///
    /// ```tombi
    /// #:tombi toml-version█= "v1.0.0"
    ///         |----------->| <- This position.
    /// ```
    pub position_in_content: tombi_text::Position,

    /// Content range based on the Root's position.
    ///
    /// ```tombi
    /// #:tombi toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^ <- This range.
    /// ```
    pub content_range: tombi_text::Range,
}

pub fn get_schema_document_comment_directive(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<SchemaDocumentCommentDirective> {
    if let Some(schema_document_comment_directive) =
        root.schema_document_comment_directive(source_path)
    {
        if schema_document_comment_directive
            .directive_range
            .contains(position)
            || schema_document_comment_directive
                .uri_range
                .contains(position)
        {
            return Some(schema_document_comment_directive);
        }
    }
    None
}

pub fn get_tombi_document_comment_directive(
    comment: &tombi_ast::Comment,
    position: tombi_text::Position,
) -> Option<TombiDocumentCommentDirective> {
    if let Some(tombi_ast::TombiDocumentCommentDirective {
        directive_range,
        content,
        content_range,
    }) = comment.tombi_document_directive()
    {
        if directive_range.contains(position) {
            return Some(TombiDocumentCommentDirective::Directive(
                DocumentTombiDirective { directive_range },
            ));
        }
        if content_range.contains(position) {
            let position_in_content =
                tombi_text::Position::new(0, position.column - (directive_range.end.column + 1));

            return Some(TombiDocumentCommentDirective::Content(
                DocumentTombiDirectiveContent {
                    content,
                    position_in_content,
                    content_range,
                },
            ));
        }
    }
    None
}
