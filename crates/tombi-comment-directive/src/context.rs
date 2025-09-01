use tombi_ast::{
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective, TombiValueCommentDirective,
};

#[derive(Debug, Clone)]
pub enum CommentDirectiveContext<T> {
    Directive {
        directive_range: tombi_text::Range,
    },
    Content {
        content: T,
        content_range: tombi_text::Range,
        position_in_content: tombi_text::Position,
    },
}

pub trait GetCommentDirectiveContext<T> {
    fn get_context(&self, position: tombi_text::Position) -> Option<CommentDirectiveContext<T>>;
}

impl GetCommentDirectiveContext<Result<tombi_uri::SchemaUri, String>>
    for SchemaDocumentCommentDirective
{
    fn get_context(
        &self,
        position: tombi_text::Position,
    ) -> Option<CommentDirectiveContext<Result<tombi_uri::SchemaUri, String>>> {
        if self.uri_range.contains(position) {
            Some(CommentDirectiveContext::Content {
                content: self.uri.to_owned(),
                content_range: self.uri_range,
                position_in_content: tombi_text::Position::new(
                    0,
                    position
                        .column
                        .saturating_sub(self.directive_range.end.column + 1),
                ),
            })
        } else if self.directive_range.contains(position) {
            Some(CommentDirectiveContext::Directive {
                directive_range: self.directive_range,
            })
        } else {
            None
        }
    }
}

impl GetCommentDirectiveContext<String> for TombiDocumentCommentDirective {
    fn get_context(
        &self,
        position: tombi_text::Position,
    ) -> Option<CommentDirectiveContext<String>> {
        if self.content_range.contains(position) {
            Some(CommentDirectiveContext::Content {
                content: self.content.clone(),
                content_range: self.content_range,
                position_in_content: tombi_text::Position::new(
                    0,
                    position
                        .column
                        .saturating_sub(self.directive_range.end.column + 1),
                ),
            })
        } else if self.directive_range.contains(position) {
            Some(CommentDirectiveContext::Directive {
                directive_range: self.directive_range,
            })
        } else {
            None
        }
    }
}

impl GetCommentDirectiveContext<String> for TombiValueCommentDirective {
    fn get_context(
        &self,
        position: tombi_text::Position,
    ) -> Option<CommentDirectiveContext<String>> {
        if self.content_range.contains(position) {
            return Some(CommentDirectiveContext::Content {
                content: self.content.clone(),
                content_range: self.content_range,
                position_in_content: tombi_text::Position::new(
                    0,
                    position
                        .column
                        .saturating_sub(self.directive_range.end.column),
                ),
            });
        } else if self.directive_range.contains(position) {
            return Some(CommentDirectiveContext::Directive {
                directive_range: self.directive_range,
            });
        } else {
            None
        }
    }
}

impl GetCommentDirectiveContext<String> for CommentDirectiveContext<String> {
    fn get_context(
        &self,
        _position: tombi_text::Position,
    ) -> Option<CommentDirectiveContext<String>> {
        Some(self.clone())
    }
}

impl GetCommentDirectiveContext<String> for Vec<TombiDocumentCommentDirective> {
    fn get_context(
        &self,
        position: tombi_text::Position,
    ) -> Option<CommentDirectiveContext<String>> {
        for comment_directive in self {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                return Some(comment_directive_context);
            }
        }
        None
    }
}
