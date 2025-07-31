use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;

use crate::Format;

impl Format for tombi_ast::Keys {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let keys = self
            .keys()
            .map(|key| key.syntax().text().to_string())
            .collect_vec()
            .join(".");

        write!(f, "{keys}")
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
