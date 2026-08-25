mod array_of_table;
mod comment;
mod key;
mod key_value;
mod root;
mod table;
mod value;

use std::borrow::Cow;
use std::fmt::Write;

use itertools::Itertools;
use tombi_config::{StringQuoteStyle, TomlVersion};

use crate::types::AlignmentWidth;

pub trait Format {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error>;
}

/// Re-quote `text` to use `quote` as its delimiter, swapping the surrounding quotes.
///
/// Leaves `text` untouched when the new delimiter or an escape would alter the content.
// TODO: Only supports simple conditions, so it needs to be changed to behavior closer to black
fn requote_string(text: &str, quote: char) -> Cow<'_, str> {
    if text.contains('\\') || text.contains(quote) {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(format!("{quote}{}{quote}", &text[1..text.len() - 1]))
    }
}

fn format_basic_string_quote_style(text: &str, quote_style: StringQuoteStyle) -> Cow<'_, str> {
    match quote_style {
        StringQuoteStyle::Double | StringQuoteStyle::Preserve => Cow::Borrowed(text),
        StringQuoteStyle::Single => requote_string(text, '\''),
    }
}

fn format_literal_string_quote_style(text: &str, quote_style: StringQuoteStyle) -> Cow<'_, str> {
    match quote_style {
        StringQuoteStyle::Single | StringQuoteStyle::Preserve => Cow::Borrowed(text),
        StringQuoteStyle::Double => requote_string(text, '"'),
    }
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
    header_keys: tombi_ast_syntax::Keys,
    parent_header_keys: impl Iterator<Item = tombi_ast_syntax::Keys> + 'a,
    toml_version: TomlVersion,
) -> impl Iterator<Item = Vec<String>> + 'a {
    parent_header_keys
        .filter(move |keys| keys.keys().count() < header_keys.keys().count())
        .map(move |keys| {
            keys.keys()
                .map(|key| key.content_lossy(toml_version))
                .collect_vec()
        })
        .unique()
}
