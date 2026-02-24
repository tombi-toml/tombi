mod alignment_width;

pub use alignment_width::AlignmentWidth;

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
}
