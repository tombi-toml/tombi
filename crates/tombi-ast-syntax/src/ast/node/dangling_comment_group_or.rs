use crate::{AstNode, DanglingCommentGroup};

/// A group of consecutive items or dangling comments separated by empty lines.
///
/// This is a logical grouping concept rather than a concrete syntax node type.
/// DanglingCommentGroupOr represents either:
/// - A standalone dangling comments group
/// - A sequence of items with their associated dangling comments
#[derive(Debug, Clone)]
pub enum DanglingCommentGroupOr<'t, T> {
    /// Standalone dangling comments group
    DanglingCommentGroup(DanglingCommentGroup<'t>),
    /// Sequence of items with their associated dangling comments
    ItemGroup(T),
}

impl<'t, T> DanglingCommentGroupOr<'t, T> {
    #[inline]
    pub fn into_dangling_comment_group(self) -> Option<DanglingCommentGroup<'t>> {
        match self {
            DanglingCommentGroupOr::DanglingCommentGroup(dangling_comment_group) => {
                Some(dangling_comment_group)
            }
            _ => None,
        }
    }

    #[inline]
    pub fn into_item_group(self) -> Option<T> {
        match self {
            DanglingCommentGroupOr::ItemGroup(item) => Some(item),
            _ => None,
        }
    }
}

impl<'t, T: AstNode<'t>> AstNode<'t> for DanglingCommentGroupOr<'t, T> {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool {
        DanglingCommentGroup::can_cast(kind) || T::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: tombi_ast_syntax::SyntaxNode<'t>) -> Option<Self> {
        if let Some(dangling_comment_group) = DanglingCommentGroup::cast(syntax) {
            Some(DanglingCommentGroupOr::DanglingCommentGroup(
                dangling_comment_group,
            ))
        } else {
            T::cast(syntax).map(DanglingCommentGroupOr::ItemGroup)
        }
    }

    #[inline]
    fn syntax(&self) -> &tombi_ast_syntax::SyntaxNode<'t> {
        match self {
            DanglingCommentGroupOr::DanglingCommentGroup(dangling_comment_group) => {
                dangling_comment_group.syntax()
            }
            DanglingCommentGroupOr::ItemGroup(item_group) => item_group.syntax(),
        }
    }
}
