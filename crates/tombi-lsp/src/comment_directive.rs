use tombi_ast_syntax::{
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective, TombiValueCommentDirective,
};
use tombi_comment_directive::{
    TombiCommentDirectiveImpl,
    value::{
        ArrayCommonFormatRules, ArrayCommonLintRules, ArrayOfTableCommonFormatRules,
        ArrayOfTableCommonLintRules, InlineTableCommonFormatRules, InlineTableCommonLintRules,
        TombiGroupBoundaryDirectiveContent, TombiKeyArrayOfTableDirectiveContent,
        TombiKeyTableDirectiveContent, TombiRootTableDirectiveContent, TombiTableDirectiveContent,
        TombiValueDirectiveContent, WithKeyFormatRules, WithKeyLintRules, WithKeyTableLintRules,
    },
};
use tombi_document_tree_syntax::{ArrayKind, TableKind};
use tombi_schema_store::Accessor;

pub const DOCUMENT_SCHEMA_DIRECTIVE_TITLE: &str = "Schema Document Directive";
pub const DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION: &str = r#"
Specify the Schema URL/Path for this document.

See the [docs](https://tombi-toml.github.io/tombi/docs/comment-directive/#document-comment-directive) for more details.
"#;

pub const DOCUMENT_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Document Directive";
pub const DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION: &str = r#"
Directives that apply only to this document.

See the [docs](https://tombi-toml.github.io/tombi/docs/comment-directive/#document-comment-directive) for more details.
"#;

pub const VALUE_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Value Directive";
pub const VALUE_TOMBI_DIRECTIVE_DESCRIPTION: &str = r#"Directives that apply only to this value.

See the [docs](https://tombi-toml.github.io/tombi/docs/comment-directive/#value-comment-directive) for more details.
"#;

#[derive(Debug, Clone)]
pub enum CommentDirectiveContext<T> {
    Directive {
        directive_span: tombi_text::Span,
    },
    Content {
        content: T,
        content_span: tombi_text::Span,
        offset_in_content: tombi_text::Offset,
    },
}

pub trait GetCommentDirectiveContext<T> {
    fn get_context(&self, offset: tombi_text::Offset) -> Option<CommentDirectiveContext<T>>;
}

impl GetCommentDirectiveContext<Result<tombi_uri::SchemaUri, String>>
    for SchemaDocumentCommentDirective
{
    fn get_context(
        &self,
        offset: tombi_text::Offset,
    ) -> Option<CommentDirectiveContext<Result<tombi_uri::SchemaUri, String>>> {
        if self.uri_span.contains_inclusive(offset) {
            Some(CommentDirectiveContext::Content {
                content: self.uri.to_owned(),
                content_span: self.uri_span,
                offset_in_content: tombi_text::Offset::new(
                    u32::from(offset).saturating_sub(u32::from(self.directive_span.end) + 1),
                ),
            })
        } else if self.directive_span.contains_inclusive(offset) {
            Some(CommentDirectiveContext::Directive {
                directive_span: self.directive_span,
            })
        } else {
            None
        }
    }
}

impl GetCommentDirectiveContext<String> for TombiDocumentCommentDirective {
    fn get_context(&self, offset: tombi_text::Offset) -> Option<CommentDirectiveContext<String>> {
        if self.content_span.contains_inclusive(offset) {
            Some(CommentDirectiveContext::Content {
                content: self.content.clone(),
                content_span: self.content_span,
                offset_in_content: tombi_text::Offset::new(
                    u32::from(offset).saturating_sub(u32::from(self.directive_span.end) + 1),
                ),
            })
        } else if self.directive_span.contains_inclusive(offset) {
            Some(CommentDirectiveContext::Directive {
                directive_span: self.directive_span,
            })
        } else {
            None
        }
    }
}

impl GetCommentDirectiveContext<String> for TombiValueCommentDirective {
    fn get_context(&self, offset: tombi_text::Offset) -> Option<CommentDirectiveContext<String>> {
        if self.content_span.contains_inclusive(offset) {
            Some(CommentDirectiveContext::Content {
                content: self.content.clone(),
                content_span: self.content_span,
                offset_in_content: tombi_text::Offset::new(
                    u32::from(offset).saturating_sub(u32::from(self.directive_span.end)),
                ),
            })
        } else if self.directive_span.contains_inclusive(offset) {
            Some(CommentDirectiveContext::Directive {
                directive_span: self.directive_span,
            })
        } else {
            None
        }
    }
}

impl GetCommentDirectiveContext<String> for CommentDirectiveContext<String> {
    fn get_context(&self, _offset: tombi_text::Offset) -> Option<CommentDirectiveContext<String>> {
        Some(self.clone())
    }
}

impl GetCommentDirectiveContext<String> for Vec<TombiDocumentCommentDirective> {
    fn get_context(&self, offset: tombi_text::Offset) -> Option<CommentDirectiveContext<String>> {
        for comment_directive in self {
            if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                return Some(comment_directive_context);
            }
        }
        None
    }
}

pub fn get_key_value_comment_directive_content_and_schema_uri<'a, FormatRules, LintRules>(
    comment_directives: Option<
        impl Iterator<Item = &'a tombi_ast_syntax::TombiValueCommentDirective> + 'a,
    >,
    offset: tombi_text::Offset,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)>
where
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyFormatRules<FormatRules>, WithKeyLintRules<LintRules>>:
        TombiCommentDirectiveImpl,
{
    let comment_directives = comment_directives?;

    for comment_directive in comment_directives {
        if let Some(comment_directive_context) = comment_directive.get_context(offset) {
            let schema_uri = if let Some(tombi_schema_store::Accessor::Index(_)) = accessors.last()
            {
                TombiValueDirectiveContent::<FormatRules, LintRules>::comment_directive_schema_url()
            } else {
                TombiValueDirectiveContent::<
                    WithKeyFormatRules<FormatRules>,
                    WithKeyLintRules<LintRules>,
                >::comment_directive_schema_url()
            };
            return Some((comment_directive_context, schema_uri));
        }
    }
    None
}

pub fn get_key_table_value_comment_directive_content_and_schema_uri<'a, FormatRules, LintRules>(
    comment_directives: Option<
        impl Iterator<Item = &'a tombi_ast_syntax::TombiValueCommentDirective> + 'a,
    >,
    offset: tombi_text::Offset,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)>
where
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyFormatRules<FormatRules>, WithKeyTableLintRules<LintRules>>:
        TombiCommentDirectiveImpl,
{
    let comment_directives = comment_directives?;

    for comment_directive in comment_directives {
        if let Some(comment_directive_context) = comment_directive.get_context(offset) {
            let schema_uri = if let Some(tombi_schema_store::Accessor::Index(_)) = accessors.last()
            {
                TombiValueDirectiveContent::<FormatRules, LintRules>::comment_directive_schema_url()
            } else {
                TombiValueDirectiveContent::<
                    WithKeyFormatRules<FormatRules>,
                    WithKeyTableLintRules<LintRules>,
                >::comment_directive_schema_url()
            };
            return Some((comment_directive_context, schema_uri));
        }
    }
    None
}

pub fn get_array_comment_directive_content_with_schema_uri(
    array: &tombi_document_tree_syntax::Array<'_>,
    offset: tombi_text::Offset,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)> {
    match array.kind() {
        ArrayKind::Array => {
            if let Some((comment_directive, schema_uri)) =
                get_key_table_value_comment_directive_content_and_schema_uri::<
                    ArrayCommonFormatRules,
                    ArrayCommonLintRules,
                >(array.header_comment_directives(), offset, accessors)
            {
                return Some((comment_directive, schema_uri));
            }

            if let Some(body_comment_directives) = array.body_comment_directives() {
                for comment_directive in body_comment_directives {
                    if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                        let schema_uri = TombiValueDirectiveContent::<
                            ArrayCommonFormatRules,
                            ArrayCommonLintRules,
                        >::comment_directive_schema_url();

                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
        ArrayKind::ArrayOfTable | ArrayKind::ParentArrayOfTable => {
            if let Some((comment_directive, schema_uri)) =
                get_key_value_comment_directive_content_and_schema_uri::<
                    ArrayOfTableCommonFormatRules,
                    ArrayOfTableCommonLintRules,
                >(array.header_comment_directives(), offset, accessors)
            {
                return Some((comment_directive, schema_uri));
            }
        }
    }

    if let Some(group_boundary_comment_directives) = array.group_boundary_comment_directives() {
        for comment_directive in group_boundary_comment_directives {
            if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                let schema_uri = TombiGroupBoundaryDirectiveContent::comment_directive_schema_url();
                return Some((comment_directive_context, schema_uri));
            }
        }
    }

    None
}

pub fn get_table_comment_directive_content_with_schema_uri(
    table: &tombi_document_tree_syntax::Table<'_>,
    offset: tombi_text::Offset,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)> {
    match table.kind() {
        TableKind::InlineTable { .. } => {
            if let Some((comment_directive, schema_uri)) =
                get_key_value_comment_directive_content_and_schema_uri::<
                    InlineTableCommonFormatRules,
                    InlineTableCommonLintRules,
                >(table.comment_directives(), offset, accessors)
            {
                return Some((comment_directive, schema_uri));
            }
        }
        TableKind::Table => {
            if let Some(header_comment_directives) = table.header_comment_directives() {
                for comment_directive in header_comment_directives {
                    if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                        let schema_uri = if matches!(accessors.last(), Some(Accessor::Index(_))) {
                            TombiKeyArrayOfTableDirectiveContent::comment_directive_schema_url()
                        } else {
                            TombiKeyTableDirectiveContent::comment_directive_schema_url()
                        };

                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }

            if let Some(body_comment_directives) = table.body_comment_directives() {
                for comment_directive in body_comment_directives {
                    if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                        let schema_uri = TombiTableDirectiveContent::comment_directive_schema_url();

                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
        TableKind::ParentTable => {
            if let Some(header_comment_directives) = table.header_comment_directives() {
                for comment_directive in header_comment_directives {
                    if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                        let schema_uri = if matches!(accessors.last(), Some(Accessor::Index(_))) {
                            TombiKeyArrayOfTableDirectiveContent::comment_directive_schema_url()
                        } else {
                            TombiKeyTableDirectiveContent::comment_directive_schema_url()
                        };

                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
        TableKind::KeyValue | TableKind::ParentKey => {}
        TableKind::Root => {
            if let Some(comment_directives) = table.comment_directives() {
                for comment_directive in comment_directives {
                    if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                        let schema_uri =
                            TombiRootTableDirectiveContent::comment_directive_schema_url();
                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
    }

    if let Some(group_boundary_comment_directives) = table.group_boundary_comment_directives() {
        for comment_directive in group_boundary_comment_directives {
            if let Some(comment_directive_context) = comment_directive.get_context(offset) {
                let schema_uri = TombiGroupBoundaryDirectiveContent::comment_directive_schema_url();
                return Some((comment_directive_context, schema_uri));
            }
        }
    }

    None
}
