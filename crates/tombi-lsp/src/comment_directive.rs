use tombi_ast::SchemaCommentDirective;

pub const SCHEMA_DIRECTIVE_TITLE: &str = "Schema Directive";
pub const SCHEMA_DIRECTIVE_DESCRIPTION: &str = "Specify the Schema URL/Path for the document.";

pub const TOMBI_DIRECTIVE_TITLE: &str = "Tombi Directive";
pub const TOMBI_DIRECTIVE_DESCRIPTION: &str = "Directives that apply only to this document.";

#[derive(Debug)]
pub enum TombiCommentDirective {
    Directive(TombiDirective),
    Content(TombiDirectiveContent),
}

#[derive(Debug)]
pub struct TombiDirective {
    pub directive_range: tombi_text::Range,
}

#[derive(Debug)]
pub struct TombiDirectiveContent {
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

pub fn get_schema_comment_directive(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<SchemaCommentDirective> {
    if let Some(schema_comment_directive) = root.schema_comment_directive(None) {
        if schema_comment_directive.directive_range.contains(position)
            || schema_comment_directive.url_range.contains(position)
        {
            return Some(schema_comment_directive);
        }
    }
    None
}

pub fn get_tombi_comment_directive(
    comment: &tombi_ast::Comment,
    position: tombi_text::Position,
) -> Option<TombiCommentDirective> {
    if let Some(tombi_ast::TombiCommentDirective {
        directive_range,
        content,
        content_range,
    }) = comment.tombi_directive()
    {
        if directive_range.contains(position) {
            return Some(TombiCommentDirective::Directive(TombiDirective {
                directive_range,
            }));
        }
        if content_range.contains(position) {
            let position_in_content =
                tombi_text::Position::new(0, position.column - (directive_range.end.column + 1));

            return Some(TombiCommentDirective::Content(TombiDirectiveContent {
                content,
                position_in_content,
                content_range,
            }));
        }
    }
    None
}
