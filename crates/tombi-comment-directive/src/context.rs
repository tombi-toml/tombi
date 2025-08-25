#[derive(Debug, Default)]
pub struct CommentContext<'a> {
    pub parent_comments: &'a [(&'a str, tombi_text::Range)],
    pub has_key: bool,
}
