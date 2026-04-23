use crate::{
    AstToken, Comment, SchemaDocumentCommentDirective, TombiDocumentCommentDirective,
    comment_directive::TombiValueCommentDirective,
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

            if let Some(source_path) = source_path
                && is_relative_file_schema_uri_text(uri_text)
            {
                Some(SchemaDocumentCommentDirective {
                    directive_range,
                    uri: resolve_relative_file_schema_uri(uri_text, source_path),
                    uri_range,
                })
            } else if let Ok(uri) = uri_text.parse::<tombi_uri::SchemaUri>() {
                Some(SchemaDocumentCommentDirective {
                    directive_range,
                    uri: Ok(uri),
                    uri_range,
                })
            } else if let Some(source_path) = source_path {
                let mut schema_file_path = std::path::PathBuf::from(uri_text);
                if schema_file_path.is_relative()
                    && let Some(source_dir_path) = source_path.parent()
                {
                    schema_file_path = source_dir_path.join(schema_file_path);
                }
                if let Ok(canonicalized_file_path) = schema_file_path.canonicalize() {
                    schema_file_path = canonicalized_file_path;
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

fn is_relative_file_schema_uri_text(uri_text: &str) -> bool {
    let Some(path_and_fragment) = uri_text.strip_prefix("file://") else {
        return false;
    };
    let path = path_and_fragment
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(path_and_fragment);

    matches!(path, "." | "..") || path.starts_with("./") || path.starts_with("../")
}

fn resolve_relative_file_schema_uri(
    uri_text: &str,
    source_path: &std::path::Path,
) -> Result<tombi_uri::SchemaUri, String> {
    if let Some(path_and_fragment) = uri_text.strip_prefix("file://")
        && let Some(source_dir_path) = source_path.parent()
        && let Ok(mut base_dir_uri) = tombi_uri::SchemaUri::from_file_path(source_dir_path)
    {
        if !base_dir_uri.path().ends_with('/') {
            let path = format!("{}/", base_dir_uri.path());
            base_dir_uri.set_path(&path);
        }

        base_dir_uri
            .join(path_and_fragment)
            .map(tombi_uri::SchemaUri::from)
            .map_err(|_| uri_text.to_string())
    } else {
        Err(uri_text.to_string())
    }
}

impl AsRef<Comment> for Comment {
    fn as_ref(&self) -> &Comment {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use tombi_uri::SchemaUri;

    use super::resolve_relative_file_schema_uri;

    #[test]
    fn relative_file_schema_path_decodes_path_bytes() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("source dir/tombi.toml");
        let resolved_uri =
            resolve_relative_file_schema_uri("file://./schemas/schema%20file.json", &source_path)
                .unwrap();

        assert_eq!(
            resolved_uri.to_file_path().unwrap(),
            temp_dir.path().join("source dir/schemas/schema file.json")
        );
    }

    #[test]
    fn parent_relative_file_schema_path_preserves_all_parent_segments() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("workspace/project/nested/tombi.toml");
        let schema_path = temp_dir.path().join("schemas/schema.json");

        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(schema_path.parent().unwrap()).unwrap();
        std::fs::write(&source_path, "").unwrap();
        std::fs::write(&schema_path, "{}").unwrap();

        let resolved_uri =
            resolve_relative_file_schema_uri("file://../../../schemas/schema.json", &source_path)
                .unwrap();

        assert_eq!(
            resolved_uri.to_file_path().unwrap().canonicalize().unwrap(),
            schema_path.canonicalize().unwrap()
        );
    }

    #[test]
    fn dot_prefixed_parent_relative_file_schema_path_preserves_all_parent_segments() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("workspace/project/nested/tombi.toml");
        let schema_path = temp_dir.path().join("workspace/schemas/schema.json");

        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(schema_path.parent().unwrap()).unwrap();
        std::fs::write(&source_path, "").unwrap();
        std::fs::write(&schema_path, "{}").unwrap();

        let resolved_uri =
            resolve_relative_file_schema_uri("file://./../../schemas/schema.json", &source_path)
                .unwrap();

        assert_eq!(
            resolved_uri.to_file_path().unwrap().canonicalize().unwrap(),
            schema_path.canonicalize().unwrap()
        );
    }

    #[test]
    fn schema_uri_fragment_parse_still_works() {
        let uri = SchemaUri::from_str("file://./schema.json#/definitions/TableValue").unwrap();
        assert_eq!(uri.fragment(), Some("/definitions/TableValue"));
    }
}
