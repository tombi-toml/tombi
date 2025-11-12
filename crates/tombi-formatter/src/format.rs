mod array_of_table;
mod comment;
mod key;
mod key_value;
mod root;
mod table;
mod value;

use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstChildren;
use tombi_config::TomlVersion;

use crate::types::AlignmentWidth;

pub trait Format {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error>;
}

fn write_trailing_comment_alignment_space(
    f: &mut crate::Formatter,
    trailing_comment_alignment_width: AlignmentWidth,
) -> Result<(), std::fmt::Error> {
    write!(
        f,
        "{}",
        " ".repeat(trailing_comment_alignment_width.value() as usize - f.current_line_width())
    )?;
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
