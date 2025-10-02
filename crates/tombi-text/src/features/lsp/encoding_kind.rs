use unicode_segmentation::UnicodeSegmentation;

use crate::Column;

/// A kind of wide character encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum EncodingKind {
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

impl EncodingKind {
    /// Returns the number of code units it takes to encode `text` in this encoding.
    pub fn measure(&self, text: &str) -> Column {
        match self {
            EncodingKind::Utf8 => text.len() as Column,
            EncodingKind::Utf16 => text.encode_utf16().count() as Column,
            EncodingKind::Utf32 => text.chars().count() as Column,
            EncodingKind::GraphemeCluster => text.graphemes(true).count() as Column,
        }
    }
}

impl TryFrom<&tower_lsp::lsp_types::PositionEncodingKind> for EncodingKind {
    type Error = ();
    fn try_from(kind: &tower_lsp::lsp_types::PositionEncodingKind) -> Result<Self, Self::Error> {
        use tower_lsp::lsp_types::PositionEncodingKind;

        match kind {
            kind if *kind == PositionEncodingKind::UTF8 => Ok(EncodingKind::Utf8),
            kind if *kind == PositionEncodingKind::UTF16 => Ok(EncodingKind::Utf16),
            kind if *kind == PositionEncodingKind::UTF32 => Ok(EncodingKind::Utf32),
            _ => Err(()),
        }
    }
}

impl From<EncodingKind> for tower_lsp::lsp_types::PositionEncodingKind {
    fn from(encoding: EncodingKind) -> Self {
        use tower_lsp::lsp_types::PositionEncodingKind;

        match encoding {
            EncodingKind::Utf8 => PositionEncodingKind::UTF8,
            EncodingKind::Utf16 => PositionEncodingKind::UTF16,
            EncodingKind::Utf32 => PositionEncodingKind::UTF32,
            EncodingKind::GraphemeCluster => unreachable!("Cannot convert EncodingKind::GraphemeCluster to PositionEncodingKind: GraphemeCluster is not supported by LSP"),
        }
    }
}
