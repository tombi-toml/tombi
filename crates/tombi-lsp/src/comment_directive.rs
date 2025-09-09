use tombi_ast::{
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective, TombiValueCommentDirective,
};
use tombi_comment_directive::{
    value::{
        ArrayCommonRules, ArrayOfTableCommonRules, InlineTableCommonRules,
        KeyArrayOfTableCommonRules, KeyTableCommonRules, RootTableCommonRules, TableCommonRules,
        TombiValueDirectiveContent, WithKeyRules, WithKeyTableRules,
    },
    TombiCommentDirectiveImpl,
};
use tombi_document_tree::{ArrayKind, TableKind};
use tombi_schema_store::Accessor;

pub const DOCUMENT_SCHEMA_DIRECTIVE_TITLE: &str = "Schema Document Directive";
pub const DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION: &str =
    "Specify the Schema URL/Path for the document.";

pub const DOCUMENT_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Document Directive";
pub const DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION: &str =
    "Directives that apply only to this document.";

pub const VALUE_TOMBI_DIRECTIVE_TITLE: &str = "Tombi Value Directive";
pub const VALUE_TOMBI_DIRECTIVE_DESCRIPTION: &str = "Directives that apply only to this value.";

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

pub fn get_key_value_comment_directive_content_and_schema_uri<Rules>(
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    position: tombi_text::Position,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)>
where
    TombiValueDirectiveContent<Rules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyRules<Rules>>: TombiCommentDirectiveImpl,
{
    if let Some(comment_directive) = comment_directives {
        for comment_directive in comment_directive {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                let schema_uri = if let Some(tombi_schema_store::Accessor::Index(_)) =
                    accessors.last()
                {
                    TombiValueDirectiveContent::<Rules>::comment_directive_schema_url()
                } else {
                    TombiValueDirectiveContent::<WithKeyRules<Rules>>::comment_directive_schema_url(
                    )
                };
                return Some((comment_directive_context, schema_uri));
            }
        }
    }
    None
}

pub fn get_key_table_value_comment_directive_content_and_schema_uri<Rules>(
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    position: tombi_text::Position,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)>
where
    TombiValueDirectiveContent<Rules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyTableRules<Rules>>: TombiCommentDirectiveImpl,
{
    if let Some(comment_directive) = comment_directives {
        for comment_directive in comment_directive {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                let schema_uri = if let Some(tombi_schema_store::Accessor::Index(_)) =
                    accessors.last()
                {
                    TombiValueDirectiveContent::<Rules>::comment_directive_schema_url()
                } else {
                    TombiValueDirectiveContent::<WithKeyTableRules<Rules>>::comment_directive_schema_url(
                    )
                };
                return Some((comment_directive_context, schema_uri));
            }
        }
    }
    None
}

pub fn get_array_comment_directive_content_with_schema_uri(
    array: &tombi_document_tree::Array,
    position: tombi_text::Position,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)> {
    if let Some((comment_directive, schema_uri)) = match array.kind() {
        ArrayKind::Array => get_key_table_value_comment_directive_content_and_schema_uri::<
            ArrayCommonRules,
        >(array.comment_directives(), position, accessors),
        ArrayKind::ArrayOfTable | ArrayKind::ParentArrayOfTable => {
            get_key_value_comment_directive_content_and_schema_uri::<ArrayOfTableCommonRules>(
                array.comment_directives(),
                position,
                accessors,
            )
        }
    } {
        return Some((comment_directive, schema_uri));
    }

    if let Some(comment_directive) = array.inner_comment_directives() {
        for comment_directive in comment_directive {
            if let Some(comment_directive_context) = comment_directive.get_context(position) {
                let schema_uri =
                    TombiValueDirectiveContent::<ArrayCommonRules>::comment_directive_schema_url();
                return Some((comment_directive_context, schema_uri));
            }
        }
    }

    None
}

pub fn get_table_comment_directive_content_with_schema_uri(
    table: &tombi_document_tree::Table,
    position: tombi_text::Position,
    accessors: &[tombi_schema_store::Accessor],
) -> Option<(CommentDirectiveContext<String>, tombi_uri::SchemaUri)> {
    match table.kind() {
        TableKind::InlineTable { .. } => {
            if let Some((comment_directive, schema_uri)) =
                get_key_value_comment_directive_content_and_schema_uri::<InlineTableCommonRules>(
                    table.comment_directives(),
                    position,
                    accessors,
                )
            {
                return Some((comment_directive, schema_uri));
            }
            if let Some(comment_directive) = table.inner_comment_directives() {
                for comment_directive in comment_directive {
                    if let Some(comment_directive_context) = comment_directive.get_context(position)
                    {
                        let schema_uri =
                    TombiValueDirectiveContent::<InlineTableCommonRules>::comment_directive_schema_url();
                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
        TableKind::Table | TableKind::ParentTable => {
            if let Some(comment_directive) = table.comment_directives() {
                for comment_directive in comment_directive {
                    if let Some(comment_directive_context) = comment_directive.get_context(position)
                    {
                        let schema_uri = if matches!(accessors.last(), Some(Accessor::Index(_))) {
                            TombiValueDirectiveContent::<KeyArrayOfTableCommonRules>::comment_directive_schema_url()
                        } else {
                            TombiValueDirectiveContent::<KeyTableCommonRules>::comment_directive_schema_url()
                        };

                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
            if let Some(comment_directive) = table.inner_comment_directives() {
                for comment_directive in comment_directive {
                    if let Some(comment_directive_context) = comment_directive.get_context(position)
                    {
                        let schema_uri = TombiValueDirectiveContent::<TableCommonRules>::comment_directive_schema_url();

                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
        TableKind::KeyValue | TableKind::ParentKey => {}
        TableKind::Root => {
            if let Some(comment_directive) = table.inner_comment_directives() {
                for comment_directive in comment_directive {
                    if let Some(comment_directive_context) = comment_directive.get_context(position)
                    {
                        let schema_uri =
                    TombiValueDirectiveContent::<RootTableCommonRules>::comment_directive_schema_url();
                        return Some((comment_directive_context, schema_uri));
                    }
                }
            }
        }
    }

    None
}
