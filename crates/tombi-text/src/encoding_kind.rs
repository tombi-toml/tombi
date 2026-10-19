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
        // Every encoding counts an ASCII character as one unit.
        if matches!(self, EncodingKind::Utf8) || text.is_ascii() {
            return text.len() as Column;
        }
        match self {
            EncodingKind::Utf8 => unreachable!(),
            EncodingKind::Utf16 => text.encode_utf16().count() as Column,
            EncodingKind::Utf32 => text.chars().count() as Column,
            EncodingKind::GraphemeCluster => text.graphemes(true).count() as Column,
        }
    }

    /// Returns the byte length of the longest prefix of `text` that is at most `units` long.
    ///
    /// The prefix never splits a unit of this encoding.
    pub fn prefix_len(&self, text: &str, units: Column) -> usize {
        let units = units as usize;
        if text.is_ascii() {
            return units.min(text.len());
        }
        match self {
            EncodingKind::Utf8 => {
                let mut len = units.min(text.len());
                while !text.is_char_boundary(len) {
                    len -= 1;
                }
                len
            }
            EncodingKind::Utf16 => prefix_len_by(text.chars(), units, char::len_utf16),
            EncodingKind::Utf32 => prefix_len_by(text.chars(), units, |_| 1),
            EncodingKind::GraphemeCluster => {
                let mut len = 0;
                for grapheme in text.graphemes(true).take(units) {
                    len += grapheme.len();
                }
                len
            }
        }
    }
}

fn prefix_len_by(
    chars: std::str::Chars<'_>,
    units: usize,
    char_units: impl Fn(char) -> usize,
) -> usize {
    let mut consumed = 0;
    let mut len = 0;
    for c in chars {
        consumed += char_units(c);
        if consumed > units {
            break;
        }
        len += c.len_utf8();
    }
    len
}

#[cfg(feature = "lsp")]
impl TryFrom<&tower_lsp_server::gen_lsp_types::PositionEncodingKind> for EncodingKind {
    type Error = ();
    fn try_from(
        kind: &tower_lsp_server::gen_lsp_types::PositionEncodingKind,
    ) -> Result<Self, Self::Error> {
        use tower_lsp_server::gen_lsp_types::PositionEncodingKind;

        match kind {
            kind if *kind == PositionEncodingKind::UTF8 => Ok(EncodingKind::Utf8),
            kind if *kind == PositionEncodingKind::UTF16 => Ok(EncodingKind::Utf16),
            kind if *kind == PositionEncodingKind::UTF32 => Ok(EncodingKind::Utf32),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "lsp")]
impl From<EncodingKind> for tower_lsp_server::gen_lsp_types::PositionEncodingKind {
    fn from(encoding: EncodingKind) -> Self {
        use tower_lsp_server::gen_lsp_types::PositionEncodingKind;

        match encoding {
            EncodingKind::Utf8 => PositionEncodingKind::UTF8,
            EncodingKind::Utf16 => PositionEncodingKind::UTF16,
            EncodingKind::Utf32 => PositionEncodingKind::UTF32,
            EncodingKind::GraphemeCluster => unreachable!(
                "Cannot convert EncodingKind::GraphemeCluster to PositionEncodingKind: GraphemeCluster is not supported by LSP"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EncodingKind;

    #[test]
    fn measure_counts_units_of_each_encoding() {
        let text = "a🦅é👨‍👩‍👧";
        assert_eq!(EncodingKind::Utf8.measure(text), 25);
        assert_eq!(EncodingKind::Utf16.measure(text), 12);
        assert_eq!(EncodingKind::Utf32.measure(text), 8);
        assert_eq!(EncodingKind::GraphemeCluster.measure(text), 4);
    }

    #[test]
    fn prefix_len_does_not_split_a_unit() {
        let text = "a🦅b";
        assert_eq!(EncodingKind::Utf8.prefix_len(text, 3), 1);
        assert_eq!(EncodingKind::Utf16.prefix_len(text, 2), 1);
        assert_eq!(EncodingKind::Utf16.prefix_len(text, 3), 5);
        assert_eq!(EncodingKind::Utf32.prefix_len(text, 2), 5);
        assert_eq!(EncodingKind::GraphemeCluster.prefix_len(text, 10), 6);
    }
}
