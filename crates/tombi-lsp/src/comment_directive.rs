use tombi_ast::{
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective, TombiValueCommentDirective,
};

pub const DOCUMENT_SCHEMA_DIRECTIVE_TITLE: &str = "Schema Document Directive";
pub const DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION: &str =
    "Specify the Schema URL/Path for the document.";

pub const DOCUMENT_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Document Directive";
pub const DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION: &str =
    "Directives that apply only to this document.";

pub const VALUE_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Value Directive";
pub const VALUE_TOMBI_DIRECTIVE_DESCRIPTION: &str = "Directives that apply only to this value.";

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
                    position.column - (self.directive_range.end.column + 1),
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
                    position.column - (self.directive_range.end.column + 1),
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
                    position.column - (self.directive_range.end.column + 1),
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

pub fn get_schema_comment_directive_context(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<CommentDirectiveContext<Result<tombi_uri::SchemaUri, String>>> {
    root.schema_document_comment_directive(source_path)
        .and_then(|comment_directive| comment_directive.get_context(position))
}

pub fn get_tombi_document_comment_directive_context(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<CommentDirectiveContext<String>> {
    if let Some(comment_directives) = root.tombi_document_comment_directives() {
        for comment_directive in comment_directives {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                return Some(comment_directive_context);
            }
        }
    }
    None
}
