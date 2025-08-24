use tombi_ast::AstToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment {
    text: String,
    range: tombi_text::Range,
}

impl Comment {
    pub fn new(text: impl Into<String>, range: tombi_text::Range) -> Self {
        Self {
            text: text.into(),
            range,
        }
    }

    pub fn tombi_value_directive(&self) -> Option<tombi_ast::TombiValueCommentDirective> {
        tombi_ast::tombi_value_comment_directive(&self.text, self.range)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl From<tombi_ast::Comment> for Comment {
    fn from(comment: tombi_ast::Comment) -> Self {
        Self::new(comment.text(), comment.syntax().range())
    }
}

impl From<tombi_ast::BeginDanglingComment> for Comment {
    fn from(comment: tombi_ast::BeginDanglingComment) -> Self {
        Self::new(comment.text(), comment.syntax().range())
    }
}

impl From<tombi_ast::EndDanglingComment> for Comment {
    fn from(comment: tombi_ast::EndDanglingComment) -> Self {
        Self::new(comment.text(), comment.syntax().range())
    }
}

impl From<tombi_ast::DanglingComment> for Comment {
    fn from(comment: tombi_ast::DanglingComment) -> Self {
        Self::new(comment.text(), comment.syntax().range())
    }
}

impl From<tombi_ast::LeadingComment> for Comment {
    fn from(comment: tombi_ast::LeadingComment) -> Self {
        Self::new(comment.text(), comment.syntax().range())
    }
}

impl From<tombi_ast::TrailingComment> for Comment {
    fn from(comment: tombi_ast::TrailingComment) -> Self {
        Self::new(comment.text(), comment.syntax().range())
    }
}
