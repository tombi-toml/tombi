use tombi_ast::AstToken;

#[allow(dead_code)]
pub enum TombiCommentDirective {
    Directive(TombiDirective),
    Content(TombiDirectiveContent),
}

#[allow(dead_code)]
pub struct TombiDirective {
    pub directive_range: tombi_text::Range,
}

pub struct TombiDirectiveContent {
    /// Directive content.
    ///
    /// ```tombi
    /// # tombi: toml-version = "v1.0.0"
    ///          ^^^^^^^^^^^^^^^^^^^^^^^ <- This content.
    /// ```
    pub content: String,

    /// Position based on the directive content.
    ///
    /// ```tombi
    /// # tombi: toml-version█= "v1.0.0"
    ///          |----------->| <- This position.
    /// ```
    pub position_in_content: tombi_text::Position,

    /// Content range based on the Root's position.
    ///
    /// ```tombi
    /// # tombi: toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^ <- This range.
    /// ```
    pub content_range: tombi_text::Range,
}

pub fn get_tombi_comment_directive(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<TombiCommentDirective> {
    if let Some(comments) = root.get_document_header_comments() {
        for comment in comments {
            let comment_range = comment.syntax().range();

            if comment_range.contains(position) {
                let comment_text = comment.syntax().text();
                if comment_text.starts_with('#') {
                    if let Some(colon_pos) = comment_text.find(':') {
                        if comment_text[1..colon_pos].trim_start() == "tombi" {
                            if position.column < comment_range.start.column + colon_pos as u32 + 1 {
                                let mut directive_range = comment_range;
                                directive_range.start.column += 1;
                                directive_range.end.column += colon_pos as u32;

                                return Some(TombiCommentDirective::Directive(TombiDirective {
                                    directive_range,
                                }));
                            }
                            let directive_content = &comment_text[colon_pos + 1..];

                            // Calculate position within the directive content
                            let mut position_in_content = position;
                            position_in_content.line = 0;
                            position_in_content.column -= colon_pos as u32 + 1;

                            // Calculate the directive range
                            let mut content_range = comment_range;
                            content_range.start.column += colon_pos as u32 + 1;

                            return Some(TombiCommentDirective::Content(TombiDirectiveContent {
                                content: directive_content.to_string(),
                                position_in_content,
                                content_range,
                            }));
                        }
                    }
                }
            }
        }
    }
    None
}
