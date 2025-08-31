use crate::{
    comment_directive::TombiValueCommentDirective, AstToken, Comment,
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective,
};

impl Comment {
    /// Returns the schema directive in the document header.
    ///
    /// ```toml
    /// #:schema "https://example.com/schema.json"
    /// ```
    pub fn get_document_schema_directive(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<SchemaDocumentCommentDirective> {
        let comment_text = self.to_string();
        if let Some(mut uri_text) = comment_text.strip_prefix("#:schema ") {
            let original_len = uri_text.len();
            uri_text = uri_text.trim_start_matches(' ');
            let space_count = (original_len - uri_text.len()) as u32;
            uri_text = uri_text.trim();

            let comment_range = self.syntax().range();
            let directive_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, 1),
                tombi_text::Position::new(
                    comment_range.end.line,
                    8, // "#:schema" length
                ),
            );
            let uri_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, 9 + space_count),
                tombi_text::Position::new(
                    comment_range.end.line,
                    9 + space_count + uri_text.len() as tombi_text::Column,
                ),
            );

            if let Ok(uri) = uri_text.parse::<tombi_uri::SchemaUri>() {
                Some(SchemaDocumentCommentDirective {
                    directive_range,
                    uri: Ok(uri),
                    uri_range,
                })
            } else if let Some(source_dir_path) = source_path {
                let mut schema_file_path = std::path::PathBuf::from(uri_text);
                if let Some(parent) = source_dir_path.parent() {
                    schema_file_path = parent.join(schema_file_path);
                }
                if let Ok(canonicalized_file_path) = schema_file_path.canonicalize() {
                    schema_file_path = canonicalized_file_path
                }

                Some(SchemaDocumentCommentDirective {
                    directive_range,
                    uri: tombi_uri::SchemaUri::from_file_path(&schema_file_path)
                        .map_err(|_| uri_text.to_string()),
                    uri_range,
                })
            } else {
                Some(SchemaDocumentCommentDirective {
                    directive_range,
                    uri: Err(uri_text.to_string()),
                    uri_range,
                })
            }
        } else {
            None
        }
    }

    /// Returns the tombi directives in the document header.
    ///
    /// ```toml
    /// #:tombi toml-version = "v1.0.0"
    /// ```
    pub fn get_tombi_document_directive(&self) -> Option<TombiDocumentCommentDirective> {
        let comment_text = self.syntax().text();
        if let Some(content) = comment_text.strip_prefix("#:tombi ") {
            let comment_range = self.syntax().range();
            let directive_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, comment_range.start.column + 1),
                tombi_text::Position::new(
                    comment_range.start.line,
                    comment_range.start.column + 7, // "#:tombi" length
                ),
            );

            let content_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, comment_range.start.column + 8),
                comment_range.end,
            );

            Some(TombiDocumentCommentDirective {
                content: content.to_string(),
                content_range,
                directive_range,
            })
        } else {
            None
        }
    }

    /// Returns the tombi value directive in the document header.
    ///
    /// ```toml
    /// # tombi: lint.rules.const_value = "error"
    /// ```
    pub fn get_tombi_value_directive(&self) -> Option<TombiValueCommentDirective> {
        let comment_text = self.syntax().text();
        let comment_range = self.syntax().range();

        let content_with_directive = comment_text[1..].trim_start();
        if !content_with_directive.starts_with("tombi:") {
            return None;
        }
        let prefix_len = (comment_text.len() - content_with_directive.len()) as u32;
        let directive_range = tombi_text::Range::new(
            tombi_text::Position::new(
                comment_range.start.line,
                comment_range.start.column + prefix_len,
            ),
            tombi_text::Position::new(
                comment_range.start.line,
                comment_range.start.column + prefix_len + 6,
            ),
        );
        let content = &content_with_directive[6..];
        let content_range = tombi_text::Range::new(
            tombi_text::Position::new(comment_range.start.line, directive_range.end.column),
            comment_range.end,
        );

        Some(TombiValueCommentDirective {
            content: content.to_string(),
            content_range,
            directive_range,
        })
    }
}

impl AsRef<Comment> for Comment {
    fn as_ref(&self) -> &Comment {
        self
    }
}
