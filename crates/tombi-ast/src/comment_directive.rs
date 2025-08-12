#[derive(Debug)]
pub struct SchemaCommentDirective {
    /// The range of the directive.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///  ^^^^^^^ <- This range
    /// ```
    pub directive_range: tombi_text::Range,

    /// The URL of the schema.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This URL
    /// ```
    pub url: Result<url::Url, String>,

    /// The range of the URL of the schema.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This range
    /// ```
    pub url_range: tombi_text::Range,
}

#[derive(Debug)]
pub struct TombiCommentDirective {
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
