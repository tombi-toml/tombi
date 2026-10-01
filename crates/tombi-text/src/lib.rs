/// This module provides types to represent text positions in tombi.
///
/// We maintain two forms of source code position information.
///
/// - [`Position`][crate::Position] represents an absolute position in terms of line and column.
/// - [`Offset`][crate::Offset] represents an absolute offset from the beginning of the text.
///
/// We also provide [`Range`] and [`Span`] to indicate text ranges.
///
/// - [`Range`][crate::Range] is a struct that represents a range of text as `(Position, Position)`.
/// - [`Span`][crate::Span] is a struct that represents a range of text as `(Offset, Offset)`.
///
/// Text positions are kept as [`Span`][crate::Span]s, and converted to [`Range`][crate::Range]s
/// with a [`LineIndex`][crate::LineIndex] only where a line and column are needed.
/// The unit of a column is chosen for each conversion by an [`EncodingKind`][crate::EncodingKind].
///
mod encoding_kind;
mod features;
mod line_ending;
mod line_index;
mod offset;
mod owned_line_index;
mod position;
mod range;
mod relative_position;
mod span;

pub use crate::line_ending::LineEnding;

type RawTextSize = u32;
pub type RawOffset = RawTextSize;
pub type RelativeOffset = RawTextSize;
pub type Line = RawTextSize;
pub type Column = RawTextSize;

/// Zero-sized type which has pointer alignment (8 on 64-bit, 4 on 32-bit).
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct PointerAlign([usize; 0]);

pub use crate::{
    encoding_kind::EncodingKind,
    line_index::{LineIndex, LineIndexCursor},
    offset::Offset,
    owned_line_index::OwnedLineIndex,
    position::Position,
    range::Range,
    relative_position::RelativePosition,
    span::Span,
};

#[cfg(feature = "lsp")]
pub use crate::features::lsp::{FromLsp, IntoLsp};

#[cfg(target_pointer_width = "16")]
compile_error!("'text' crate assumes usize >= u32 and does not work on 16-bit targets");

#[cfg(feature = "lsp")]
#[inline]
/// Converts a `Range` to a `tower_lsp::lsp_types::Range`.
///
/// Use this only for a range whose columns are already counted in the encoding of the client,
/// or for a file without a [`LineIndex`], at the cost of accuracy.
///
pub fn convert_range_to_lsp(range: Range) -> tower_lsp::lsp_types::Range {
    tower_lsp::lsp_types::Range::new(
        tower_lsp::lsp_types::Position::new(range.start.line, range.start.column),
        tower_lsp::lsp_types::Position::new(range.end.line, range.end.column),
    )
}
