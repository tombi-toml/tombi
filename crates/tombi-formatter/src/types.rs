mod alignment_width;

pub use alignment_width::AlignmentWidth;

#[derive(Debug)]
pub struct WithAlignmentHint<T> {
    pub value: T,
    pub equal_alignment_width: Option<AlignmentWidth>,
    pub trailing_comment_alignment_width: Option<AlignmentWidth>,
}

impl<T> WithAlignmentHint<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            equal_alignment_width: None,
            trailing_comment_alignment_width: None,
        }
    }

    #[inline]
    pub fn new_with_equal_alignment_width(
        value: T,
        equal_alignment_width: Option<AlignmentWidth>,
    ) -> Self {
        Self {
            value,
            equal_alignment_width,
            trailing_comment_alignment_width: None,
        }
    }

    #[inline]
    pub fn new_with_trailing_comment_alignment_width(
        value: T,
        trailing_comment_alignment_width: Option<AlignmentWidth>,
    ) -> Self {
        Self {
            value,
            equal_alignment_width: None,
            trailing_comment_alignment_width,
        }
    }

    #[inline]
    pub fn new_with_dangling_comment_group_or(
        group: tombi_ast_syntax::DanglingCommentGroupOr<T>,
        equal_alignment_width: Option<AlignmentWidth>,
        trailing_comment_alignment_width: Option<AlignmentWidth>,
    ) -> tombi_ast_syntax::DanglingCommentGroupOr<Self> {
        match group {
            tombi_ast_syntax::DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                tombi_ast_syntax::DanglingCommentGroupOr::DanglingCommentGroup(comment_group)
            }
            tombi_ast_syntax::DanglingCommentGroupOr::ItemGroup(item_group) => {
                tombi_ast_syntax::DanglingCommentGroupOr::ItemGroup(Self {
                    value: item_group,
                    equal_alignment_width,
                    trailing_comment_alignment_width,
                })
            }
        }
    }
}

impl<T: tombi_ast_syntax::AstNode> tombi_ast_syntax::AstNode for WithAlignmentHint<T> {
    #[inline]
    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool
    where
        Self: Sized,
    {
        T::can_cast(kind)
    }

    #[inline]
    fn cast(syntax: tombi_ast_syntax::SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        T::cast(syntax).map(Self::new)
    }

    #[inline]
    fn syntax(&self) -> &tombi_ast_syntax::SyntaxNode {
        self.value.syntax()
    }
}
