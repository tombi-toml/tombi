mod alignment_width;

pub use alignment_width::AlignmentWidth;

pub struct KeyValueWithAlignmentHint<'a, T> {
    pub value: &'a T,
    pub equal_alignment_width: Option<AlignmentWidth>,
}
