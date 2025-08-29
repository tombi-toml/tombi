#[derive(Debug)]
pub struct SchemaDocumentCommentDirective {
    /// The range of the directive.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///  ^^^^^^^ <- This range
    /// ```
    pub directive_range: tombi_text::Range,

    /// The URI of the schema.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This URI
    /// ```
    pub uri: Result<tombi_uri::Uri, String>,

    /// The range of the URI of the schema.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This range
    /// ```
    pub uri_range: tombi_text::Range,
}

#[derive(Debug)]
pub struct TombiDocumentCommentDirective {
    /// The range of the directive.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    ///  ^^^^^^ <- This range
    /// ```
    pub directive_range: tombi_text::Range,

    /// The content of the directive.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^ <- This content
    /// ```
    pub content: String,

    /// The range of the content of the directive.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^ <- This range
    /// ```
    pub content_range: tombi_text::Range,
}

impl TombiDocumentCommentDirective {
    pub fn position_in_content(
        &self,
        position: tombi_text::Position,
    ) -> Option<tombi_text::Position> {
        if self.content_range.contains(position) {
            Some(tombi_text::Position::new(
                0,
                position.column - (self.directive_range.end.column + 1),
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TombiValueCommentDirective {
    /// The range of the directive.
    ///
    /// ```toml
    /// # tombi: lint.rules.const-value = "error"
    ///   ^^^^^^ <- This range
    /// ```
    pub directive_range: tombi_text::Range,

    /// The content of the directive.
    ///
    /// ```toml
    /// # tombi: lint.rules.const-value = "error"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This content
    /// ```
    pub content: String,

    /// The range of the content of the directive.
    ///
    /// ```toml
    /// # tombi: lint.rules.const-value = "error"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This range
    /// ```
    pub content_range: tombi_text::Range,
}

impl TombiValueCommentDirective {
    pub fn position_in_content(
        &self,
        position: tombi_text::Position,
    ) -> Option<tombi_text::Position> {
        if self.content_range.contains(position) {
            Some(tombi_text::Position::new(
                0,
                position.column - (self.directive_range.end.column + 1),
            ))
        } else {
            None
        }
    }
}
