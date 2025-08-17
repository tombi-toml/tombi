use tombi_ast::DocumentSchemaCommentDirective;

pub const DOCUMENT_SCHEMA_DIRECTIVE_TITLE: &str = "DocumentSchema Directive";
pub const DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION: &str =
    "Specify the Schema URL/Path for the document.";

pub const DOCUMENT_TOMBI_DIRECTIVE_TITLE: &str = "Document Tombi Directive";
pub const DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION: &str =
    "Directives that apply only to this document.";

#[derive(Debug)]
pub enum DocumentTombiCommentDirective {
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

pub fn get_document_schema_comment_directive(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<DocumentSchemaCommentDirective> {
    if let Some(document_schema_comment_directive) =
        root.document_schema_comment_directive(source_path)
    {
        if document_schema_comment_directive
            .directive_range
            .contains(position)
            || document_schema_comment_directive
                .uri_range
                .contains(position)
        {
            return Some(document_schema_comment_directive);
        }
    }
    None
}

pub fn get_document_tombi_comment_directive(
    comment: &tombi_ast::Comment,
    position: tombi_text::Position,
) -> Option<DocumentTombiCommentDirective> {
    if let Some(tombi_ast::DocumentTombiCommentDirective {
        directive_range,
        content,
        content_range,
    }) = comment.document_tombi_directive()
    {
        if directive_range.contains(position) {
            return Some(DocumentTombiCommentDirective::Directive(
                DocumentTombiDirective { directive_range },
            ));
        }
        if content_range.contains(position) {
            let position_in_content =
                tombi_text::Position::new(0, position.column - (directive_range.end.column + 1));

            return Some(DocumentTombiCommentDirective::Content(
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
