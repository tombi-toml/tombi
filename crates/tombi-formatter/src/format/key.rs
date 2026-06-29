use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;

use crate::{
    Format,
    format::{format_basic_string_quote_style, format_literal_string_quote_style},
    types::{AlignmentWidth, WithAlignmentHint},
};

impl Format for WithAlignmentHint<&tombi_ast::Keys> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let keys = self.value;
        let mut keys_string = keys
            .keys()
            .map(|key| match key {
                tombi_ast::Key::BareKey(it) => it.syntax().text().to_string(),
                tombi_ast::Key::BasicString(it) => format_basic_string_quote_style(
                    it.token().unwrap().text(),
                    f.string_quote_style(),
                )
                .into_owned(),
                tombi_ast::Key::LiteralString(it) => format_literal_string_quote_style(
                    it.token().unwrap().text(),
                    f.string_quote_style(),
                )
                .into_owned(),
            })
            .join(".");

        if let Some(keys_alignment_width) = self.equal_alignment_width {
            keys_string.push_str(&" ".repeat(
                (keys_alignment_width.value() - AlignmentWidth::new(&keys_string).value()) as usize,
            ));
        }

        write!(f, "{keys_string}")
    }
}

impl Format for tombi_ast::BareKey {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.syntax().text())
    }
}

impl Format for tombi_ast::Key {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::BareKey(it) => it.format(f),
            Self::BasicString(it) => it.format(f),
            Self::LiteralString(it) => it.format(f),
        }
    }
}
