use crate::{AstNode, DanglingCommentGroup};

/// A group of consecutive items or dangling comments separated by empty lines.
///
/// This is a logical grouping concept rather than a concrete syntax node type.
/// DanglingCommentGroupOr represents either:
/// - A standalone dangling comments group
/// - A sequence of items with their associated dangling comments
#[derive(Debug, Clone)]
pub enum DanglingCommentGroupOr<T> {
    /// Standalone dangling comments group
    DanglingCommentGroup(DanglingCommentGroup),
    /// Sequence of items with their associated dangling comments
    ItemGroup(T),
}

impl<T: AstNode> AstNode for DanglingCommentGroupOr<T> {
    #[inline]
    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool {
        DanglingCommentGroup::can_cast(kind) || T::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: tombi_syntax::SyntaxNode) -> Option<Self> {
        if let Some(dangling_comment_group) = DanglingCommentGroup::cast(syntax.clone()) {
            Some(DanglingCommentGroupOr::DanglingCommentGroup(
                dangling_comment_group,
            ))
        } else if let Some(item) = T::cast(syntax) {
            Some(DanglingCommentGroupOr::ItemGroup(item))
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &tombi_syntax::SyntaxNode {
        match self {
            DanglingCommentGroupOr::DanglingCommentGroup(dangling_comment_group) => {
                dangling_comment_group.syntax()
            }
            DanglingCommentGroupOr::ItemGroup(item_group) => item_group.syntax(),
        }
    }
}
