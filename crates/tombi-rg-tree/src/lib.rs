//! A generic library for lossless syntax trees.
//! See `examples/s_expressions.rs` for a tutorial.
#![forbid(
    // missing_debug_implementations,
    unconditional_recursion,
    future_incompatible,
    // missing_docs,
)]
#![deny(unsafe_code)]

#[allow(unsafe_code)]
pub mod cursor;
#[allow(unsafe_code)]
mod green;
mod language;
mod red;

mod syntax_text;
mod utility_types;

#[allow(unsafe_code)]
mod arc;
mod cow_mut;
#[allow(unsafe_code)]
mod sll;

use crate::utility_types::WalkEvent;
pub use crate::{
    green::{
        Checkpoint, Children, GreenNode, GreenNodeBuilder, GreenNodeData, GreenToken,
        GreenTokenData, NodeCache, SyntaxKind,
    },
    language::Language,
    red::{
        RedElement, RedElementChildren, RedNode, RedNodeChildren, RedNodePtr,
        RedPreorderWithTokens, RedToken,
    },
    syntax_text::SyntaxText,
    utility_types::{Direction, NodeOrToken, TokenAtOffset},
};
