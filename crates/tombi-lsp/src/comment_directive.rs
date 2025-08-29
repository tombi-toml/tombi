use tombi_ast::{SchemaDocumentCommentDirective, TombiDocumentCommentDirective};

pub const DOCUMENT_SCHEMA_DIRECTIVE_TITLE: &str = "Schema Document Directive";
pub const DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION: &str =
    "Specify the Schema URL/Path for the document.";

pub const DOCUMENT_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Document Directive";
pub const DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION: &str =
    "Directives that apply only to this document.";

pub const VALUE_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Value Directive";
pub const VALUE_TOMBI_DIRECTIVE_DESCRIPTION: &str = "Directives that apply only to this value.";

pub fn get_schema_document_comment_directive(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    source_path: Option<&std::path::Path>,
) -> Option<SchemaDocumentCommentDirective> {
    if let Some(schema_document_comment_directive) =
        root.schema_document_comment_directive(source_path)
    {
        if schema_document_comment_directive
            .directive_range
            .contains(position)
            || schema_document_comment_directive
                .uri_range
                .contains(position)
        {
            return Some(schema_document_comment_directive);
        }
    }
    None
}

pub fn get_tombi_document_comment_directive(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<TombiDocumentCommentDirective> {
    if let Some(comment_directives) = root.tombi_document_comment_directives() {
        for comment_directive in comment_directives {
            if comment_directive.directive_range.contains(position)
                || comment_directive.content_range.contains(position)
            {
                return Some(comment_directive);
            }
        }
    }
    None
}
