#[derive(Debug, Default)]
pub struct DocumentCommentDirectives {
    pub schema: Option<SchemaDocumentCommentDirective>,
    pub tombi: Vec<TombiDocumentCommentDirective>,
}

impl DocumentCommentDirectives {
    pub fn from_comments<'a>(
        comments: impl Iterator<Item = crate::Comment<'a>>,
        source_path: Option<&std::path::Path>,
    ) -> Option<Self> {
        let mut document_comment_directives = DocumentCommentDirectives::default();
        let mut found = false;
        for comment in comments {
            if let Some(schema_directive) = comment.get_document_schema_directive(source_path) {
                found = true;
                document_comment_directives.schema = Some(schema_directive);
            }
            if let Some(tombi_directive) = comment.get_tombi_document_directive() {
                found = true;
                document_comment_directives.tombi.push(tombi_directive);
            }
        }

        found.then_some(document_comment_directives)
    }
}

#[derive(Debug)]
pub struct SchemaDocumentCommentDirective {
    /// The span of the directive.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///  ^^^^^^^ <- This span
    /// ```
    pub directive_span: tombi_text::Span,

    /// The URI of the schema.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This URI
    /// ```
    pub uri: Result<tombi_uri::SchemaUri, String>,

    /// The span of the URI of the schema.
    ///
    /// ```toml
    /// #:schema https://example.com/schema.json
    ///          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This span
    /// ```
    pub uri_span: tombi_text::Span,
}

#[derive(Debug)]
pub struct TombiDocumentCommentDirective {
    /// The span of the directive.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    ///  ^^^^^^ <- This span
    /// ```
    pub directive_span: tombi_text::Span,

    /// The content of the directive.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^ <- This content
    /// ```
    pub content: String,

    /// The span of the content of the directive.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^ <- This span
    /// ```
    pub content_span: tombi_text::Span,
}

impl TombiDocumentCommentDirective {
    pub fn span(&self) -> tombi_text::Span {
        self.directive_span + self.content_span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TombiValueCommentDirective {
    /// The span of the directive.
    ///
    /// ```toml
    /// # tombi: lint.rules.const-value = "error"
    ///   ^^^^^^ <- This span
    /// ```
    pub directive_span: tombi_text::Span,

    /// The content of the directive.
    ///
    /// ```toml
    /// # tombi: lint.rules.const-value = "error"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This content
    /// ```
    pub content: String,

    /// The span of the content of the directive.
    ///
    /// ```toml
    /// # tombi: lint.rules.const-value = "error"
    ///         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ <- This span
    /// ```
    pub content_span: tombi_text::Span,
}

impl TombiValueCommentDirective {
    pub fn span(&self) -> tombi_text::Span {
        self.directive_span + self.content_span
    }
}
