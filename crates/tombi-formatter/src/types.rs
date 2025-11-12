mod alignment_width;

pub use alignment_width::AlignmentWidth;

pub struct WithAlignmentHint<'a, T> {
    pub value: &'a T,
    pub equal_alignment_width: Option<AlignmentWidth>,
    pub trailing_comment_alignment_width: Option<AlignmentWidth>,
}

impl<'a, T> WithAlignmentHint<'a, T> {
    pub fn new_without_hint(value: &'a T) -> Self {
        Self {
            value,
            equal_alignment_width: None,
            trailing_comment_alignment_width: None,
        }
    }
}
