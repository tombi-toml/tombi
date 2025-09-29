use unicode_segmentation::UnicodeSegmentation;

use crate::Column;

/// A kind of wide character encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum WideEncoding {
    /// UTF-8.
    Utf8,

    /// UTF-16.
    #[default]
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

impl TryFrom<&tower_lsp::lsp_types::PositionEncodingKind> for WideEncoding {
    type Error = ();
    fn try_from(kind: &tower_lsp::lsp_types::PositionEncodingKind) -> Result<Self, Self::Error> {
        use tower_lsp::lsp_types::PositionEncodingKind;

        match kind {
            kind if *kind == PositionEncodingKind::UTF8 => Ok(WideEncoding::Utf8),
            kind if *kind == PositionEncodingKind::UTF16 => Ok(WideEncoding::Utf16),
            kind if *kind == PositionEncodingKind::UTF32 => Ok(WideEncoding::Utf32),
            _ => Err(()),
        }
    }
}

impl From<WideEncoding> for tower_lsp::lsp_types::PositionEncodingKind {
    fn from(encoding: WideEncoding) -> Self {
        use tower_lsp::lsp_types::PositionEncodingKind;

        match encoding {
            WideEncoding::Utf8 => PositionEncodingKind::UTF8,
            WideEncoding::Utf16 => PositionEncodingKind::UTF16,
            WideEncoding::Utf32 => PositionEncodingKind::UTF32,
            WideEncoding::GraphemeCluster => unreachable!("Cannot convert WideEncoding::GraphemeCluster to PositionEncodingKind: GraphemeCluster is not supported by LSP"),
        }
    }
}
