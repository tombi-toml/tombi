mod array_of_table;
mod comment;
mod key;
mod key_value;
mod root;
mod table;
mod value;

use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::{AstChildren, AstNode};
use tombi_config::TomlVersion;
use tombi_syntax::SyntaxKind::{LINE_BREAK, WHITESPACE};

use crate::types::AlignmentWidth;

pub trait Format {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error>;
}

fn write_trailing_comment_alignment_space(
    f: &mut crate::Formatter,
    trailing_comment_alignment_width: AlignmentWidth,
) -> Result<(), std::fmt::Error> {
    let spaces =
        (trailing_comment_alignment_width.value() as usize).saturating_sub(f.current_line_width());
    write!(f, "{}", " ".repeat(spaces))?;
    Ok(())
}

fn filter_map_unique_keys<'a>(
    header_keys: AstChildren<tombi_ast::Key>,
    parent_header_keys: impl Iterator<Item = AstChildren<tombi_ast::Key>> + 'a,
    toml_version: TomlVersion,
) -> impl Iterator<Item = Vec<String>> + 'a {
    parent_header_keys
        .filter(move |keys| keys.clone().count() < header_keys.clone().count())
        .map(move |keys| keys.map(|key| key.to_raw_text(toml_version)).collect_vec())
        .unique()
}

pub(crate) fn blank_lines_before<T: AstNode>(node: &T) -> u8 {
    let mut line_break_count = 0usize;
    let mut current = node.syntax().prev_sibling_or_token();

    while let Some(element) = current {
        match element.kind() {
            WHITESPACE => current = element.prev_sibling_or_token(),
            LINE_BREAK => {
                line_break_count += 1;
                current = element.prev_sibling_or_token();
            }
            _ => break,
        }
    }

    let blank_lines = line_break_count.saturating_sub(1);
    u8::try_from(blank_lines).unwrap_or(u8::MAX)
}
