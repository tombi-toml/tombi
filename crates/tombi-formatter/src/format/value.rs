mod array;
mod boolean;
mod date_time;
mod float;
mod inline_table;
mod integer;
mod string;

use itertools::Itertools;
use std::fmt::Write;

use tombi_syntax::SyntaxToken;

use crate::Format;

impl Format for tombi_ast::Value {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Array(it) => it.format(f),
            Self::BasicString(it) => it.format(f),
            Self::Boolean(it) => it.format(f),
            Self::Float(it) => it.format(f),
            Self::InlineTable(it) => it.format(f),
            Self::IntegerBin(it) => it.format(f),
            Self::IntegerDec(it) => it.format(f),
            Self::IntegerHex(it) => it.format(f),
            Self::IntegerOct(it) => it.format(f),
            Self::LiteralString(it) => it.format(f),
            Self::LocalDate(it) => it.format(f),
            Self::LocalDateTime(it) => it.format(f),
            Self::LocalTime(it) => it.format(f),
            Self::MultiLineBasicString(it) => it.format(f),
            Self::MultiLineLiteralString(it) => it.format(f),
            Self::OffsetDateTime(it) => it.format(f),
        }
    }
}

trait LiteralNode {
    fn token(&self) -> Option<SyntaxToken>;
}

impl<T> Format for T
where
    T: LiteralNode + tombi_ast::AstNode,
{
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        self.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        write!(f, "{}", self.token().unwrap())?;

        if let Some(comment) = self.trailing_comment() {
            comment.format(f)?;
        }

        Ok(())
    }
}
