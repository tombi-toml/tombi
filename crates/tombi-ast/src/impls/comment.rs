use tombi_wasm_compat::url::url_from_file_path;
use crate::{AstToken, Comment};

impl Comment {
    pub fn schema_url(
        &self,
        source_path: Option<&std::path::Path>,
    ) -> Option<(Result<url::Url, String>, tombi_text::Range)> {
        let comment_string = self.to_string();
        if let Some(mut url_str) = comment_string.strip_prefix("#:schema ") {
            let original_len = url_str.len();
            url_str = url_str.trim_start_matches(' ');
            let space_count = (original_len - url_str.len()) as u32;
            url_str = url_str.trim();

            let mut schema_url_range = self.syntax().range();
            schema_url_range = tombi_text::Range::new(
                tombi_text::Position::new(schema_url_range.start().line(), 9 + space_count),
                tombi_text::Position::new(
                    schema_url_range.end().line(),
                    9 + space_count + url_str.len() as tombi_text::Column,
                ),
            );

            if let Ok(url) = url_str.parse::<url::Url>() {
                Some((Ok(url), schema_url_range))
            } else if let Some(source_dir_path) = source_path {
                let mut schema_file_path = std::path::PathBuf::from(url_str);
                if let Some(parent) = source_dir_path.parent() {
                    schema_file_path = parent.join(schema_file_path);
                }
                if let Ok(canonicalized_file_path) = schema_file_path.canonicalize() {
                    schema_file_path = canonicalized_file_path
                }

                Some((
                    url_from_file_path(&schema_file_path).map_err(|_| url_str.to_string()),
                    schema_url_range,
                ))
            } else {
                Some((Err(url_str.to_string()), schema_url_range))
            }
        } else {
            None
        }
    }
}
