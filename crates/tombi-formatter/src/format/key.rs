use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;

use crate::{
    types::{AlignmentWidth, WithAlignmentHint},
    Format,
};

impl Format for WithAlignmentHint<'_, tombi_ast::Keys> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let keys = self.value;
        let mut keys_string = keys
            .keys()
            .map(|key| key.syntax().text().to_string())
            .collect_vec()
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
