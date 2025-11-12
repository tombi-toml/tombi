use unicode_segmentation::UnicodeSegmentation;

/// The width of the alignment.
///
/// This struct stores the maximum length of the alignment.
/// excluding `key_value_equal_space`.
///
/// The length is measured using `GraphemeCluster`, which counts the number of user-perceived
/// characters, not bytes or codepoints.
///
/// ```toml
/// key1      = "value"
/// key2.key3 = "value"
/// ^^^^^^^^^  <- this shows the width
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AlignmentWidth(u32);

impl AlignmentWidth {
    #[inline]
    pub fn new(text: &str) -> Self {
        Self(
            text.split('\n')
                .map(|line| UnicodeSegmentation::graphemes(line, true).count() as u32)
                .max()
                .unwrap(),
        )
    }

    #[inline]
    pub fn value(&self) -> u32 {
        self.0
    }
}
