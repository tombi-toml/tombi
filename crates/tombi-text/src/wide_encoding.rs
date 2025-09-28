use unicode_segmentation::UnicodeSegmentation;

use crate::Column;

/// A kind of wide character encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WideEncoding {
    /// UTF-8.
    Utf8,

    /// UTF-16.
    Utf16,

    /// UTF-32.
    Utf32,

    /// Represents the width as seen in a text editor, counting grapheme clusters (user-perceived characters).
    GraphemeCluster,
}

impl WideEncoding {
    /// Returns the number of code units it takes to encode `text` in this encoding.
    pub fn measure(&self, text: &str) -> Column {
        match self {
            WideEncoding::Utf8 => text.len() as Column,
            WideEncoding::Utf16 => text.encode_utf16().count() as Column,
            WideEncoding::Utf32 => text.chars().count() as Column,
            WideEncoding::GraphemeCluster => text.graphemes(true).count() as Column,
        }
    }
}
