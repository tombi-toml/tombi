#[derive(Debug, Default)]
pub struct DocumentCommentDirectives {
    pub schema: Option<SchemaDocumentCommentDirective>,
    pub tombi: Vec<TombiDocumentCommentDirective>,
}

impl DocumentCommentDirectives {
    pub fn from_comments(
        comments: impl Iterator<Item = crate::Comment>,
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

        found.then(|| document_comment_directives)
    }
}

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
    pub uri: Result<tombi_uri::SchemaUri, String>,

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
    pub fn range(&self) -> tombi_text::Range {
        self.directive_range + self.content_range
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
    pub fn range(&self) -> tombi_text::Range {
        self.directive_range + self.content_range
    }
}
