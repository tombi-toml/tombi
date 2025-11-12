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

use crate::{format::write_trailing_comment_alignment_space, types::WithAlignmentHint, Format};

impl Format for tombi_ast::Value {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<'_, tombi_ast::Value> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        match self.value {
            tombi_ast::Value::Array(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::BasicString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::Boolean(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::Float(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::InlineTable(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::IntegerBin(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::IntegerDec(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::IntegerHex(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::IntegerOct(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::LiteralString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::LocalDate(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::LocalDateTime(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::LocalTime(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::MultiLineBasicString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::MultiLineLiteralString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast::Value::OffsetDateTime(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
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
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl<T> Format for WithAlignmentHint<'_, T>
where
    T: LiteralNode + tombi_ast::AstNode,
{
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let value = self.value;
        value.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        write!(f, "{}", value.token().unwrap())?;

        if let Some(comment) = value.trailing_comment() {
            if let Some(trailing_comment_alignment_width) = self.trailing_comment_alignment_width {
                write_trailing_comment_alignment_space(f, trailing_comment_alignment_width)?;
            }
            comment.format(f)?;
        }

        Ok(())
    }
}
