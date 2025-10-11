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
/// The biggest difference from Rust Analyzer's Red-Green Tree is that we preserve two representations,
/// [`Position`][crate::Position] and [`Offset`][crate::Offset], in the tree.
/// This increases the memory size of the tree,
/// but makes it much easier to implement features that work with the tree.
///
mod features;
mod offset;
mod position;
mod range;
mod relative_position;
mod span;

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
    offset::Offset, position::Position, range::Range, relative_position::RelativePosition,
    span::Span,
};

#[cfg(feature = "lsp")]
pub use crate::features::lsp::{EncodingKind, FromLsp, IntoLsp, LineIndex};

#[cfg(target_pointer_width = "16")]
compile_error!("'text' crate assumes usize >= u32 and does not work on 16-bit targets");

#[cfg(feature = "lsp")]
#[inline]
/// Converts a `Range` to a `tower_lsp::lsp_types::Range`.
///
/// Ideally, we should use `line_index` and consider `EncodingKind` when converting positions.
/// However, only TOML files maintain a `line_index`. Other file types do not support this.
/// For simplicity, and at the cost of accuracy, this function forcibly converts to the LSP type.
///
pub fn convert_range_to_lsp(range: Range) -> tower_lsp::lsp_types::Range {
    tower_lsp::lsp_types::Range::new(
        tower_lsp::lsp_types::Position::new(range.start.line, range.start.column),
        tower_lsp::lsp_types::Position::new(range.end.line, range.end.column),
    )
}
