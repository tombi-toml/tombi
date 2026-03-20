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

            if let Ok(uri) = uri_text.parse::<tombi_uri::SchemaUri>() {
                let is_relative_file_domain =
                    uri.scheme() == "file" && matches!(uri.host_str(), Some(".") | Some(".."));

                if is_relative_file_domain && let Some(source_path) = source_path {
                    let mut schema_file_path = std::path::PathBuf::from(format!(
                        "{}{}",
                        uri.host_str().unwrap_or_default(),
                        percent_decode_uri_path(uri.path())
                    ));
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
                            .map(|mut schema_uri| {
                                schema_uri.set_fragment(uri.fragment());
                                schema_uri
                            })
                            .map_err(|_| uri_text.to_string()),
                        uri_range,
                    })
                } else {
                    Some(SchemaDocumentCommentDirective {
                        directive_range,
                        uri: Ok(uri),
                        uri_range,
                    })
                }
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

fn percent_decode_uri_path(path: &str) -> String {
    let mut result = Vec::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let mut hex_chars = String::new();
            for _ in 0..2 {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_hexdigit() {
                        hex_chars.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if hex_chars.len() == 2
                && let Ok(byte) = u8::from_str_radix(&hex_chars, 16)
            {
                result.push(byte);
                continue;
            }

            result.push(b'%');
            result.extend(hex_chars.bytes());
        } else {
            let mut buffer = [0; 4];
            result.extend(ch.encode_utf8(&mut buffer).bytes());
        }
    }

    String::from_utf8_lossy(&result).into_owned()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tombi_uri::SchemaUri;

    use super::percent_decode_uri_path;

    #[test]
    fn percent_decode_uri_path_decodes_path_bytes() {
        assert_eq!(percent_decode_uri_path("/foo%20bar/schema.json"), "/foo bar/schema.json");
    }

    #[test]
    fn relative_file_schema_path_joins_after_percent_decode() {
        let source_dir = std::path::Path::new("/tmp/source dir");
        let schema_file_path =
            std::path::PathBuf::from(format!(".{}", percent_decode_uri_path("/schemas/schema%20file.json")));

        assert_eq!(
            source_dir.join(schema_file_path),
            std::path::Path::new("/tmp/source dir/./schemas/schema file.json")
        );
    }

    #[test]
    fn schema_uri_fragment_parse_still_works() {
        let uri = SchemaUri::from_str("file://./schema.json#/definitions/TableValue").unwrap();
        assert_eq!(uri.fragment(), Some("/definitions/TableValue"));
    }
}

impl AsRef<Comment> for Comment {
    fn as_ref(&self) -> &Comment {
        self
    }
}
