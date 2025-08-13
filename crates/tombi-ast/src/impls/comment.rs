use crate::{AstToken, Comment, DocumentSchemaCommentDirective, DocumentTombiCommentDirective};
use tombi_url::url_from_file_path;

impl Comment {
    /// Returns the schema directive in the document header.
    ///
    /// ```toml
    /// #:schema "https://example.com/schema.json"
    /// ```
    pub fn document_schema_directive(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<DocumentSchemaCommentDirective> {
        let comment_string = self.to_string();
        if let Some(mut url_str) = comment_string.strip_prefix("#:schema ") {
            let original_len = url_str.len();
            url_str = url_str.trim_start_matches(' ');
            let space_count = (original_len - url_str.len()) as u32;
            url_str = url_str.trim();

            let comment_range = self.syntax().range();
            let directive_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, 1),
                tombi_text::Position::new(
                    comment_range.end.line,
                    8, // "#:schema" length
                ),
            );
            let url_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, 9 + space_count),
                tombi_text::Position::new(
                    comment_range.end.line,
                    9 + space_count + url_str.len() as tombi_text::Column,
                ),
            );

            if let Ok(url) = url_str.parse::<url::Url>() {
                Some(DocumentSchemaCommentDirective {
                    directive_range,
                    url: Ok(url),
                    url_range,
                })
            } else if let Some(source_dir_path) = source_path {
                let mut schema_file_path = std::path::PathBuf::from(url_str);
                if let Some(parent) = source_dir_path.parent() {
                    schema_file_path = parent.join(schema_file_path);
                }
                if let Ok(canonicalized_file_path) = schema_file_path.canonicalize() {
                    schema_file_path = canonicalized_file_path
                }

                Some(DocumentSchemaCommentDirective {
                    directive_range,
                    url: url_from_file_path(&schema_file_path).map_err(|_| url_str.to_string()),
                    url_range,
                })
            } else {
                Some(DocumentSchemaCommentDirective {
                    directive_range,
                    url: Err(url_str.to_string()),
                    url_range,
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
    pub fn document_tombi_directive(&self) -> Option<DocumentTombiCommentDirective> {
        let comment_str = self.syntax().text();
        if let Some(content) = comment_str.strip_prefix("#:tombi ") {
            let comment_range = self.syntax().range();
            let directive_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, 1),
                tombi_text::Position::new(
                    comment_range.end.line,
                    7, // "#:tombi" length
                ),
            );

            let content_range = tombi_text::Range::new(
                tombi_text::Position::new(comment_range.start.line, 8),
                comment_range.end,
            );

            Some(DocumentTombiCommentDirective {
                content: content.to_string(),
                content_range,
                directive_range,
            })
        } else {
            None
        }
    }
}
