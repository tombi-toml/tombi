mod array;
mod boolean;
mod date_time;
mod float;
mod inline_table;
mod integer;
mod string;

use itertools::Itertools;
use std::fmt::Write;

use tombi_ast_syntax::SyntaxToken;

use crate::{Format, format::write_trailing_comment_alignment_space, types::WithAlignmentHint};

impl<'t> Format for tombi_ast_syntax::Value<'t> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl<'t> Format for WithAlignmentHint<&tombi_ast_syntax::Value<'t>> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        match self.value {
            tombi_ast_syntax::Value::Array(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::BasicString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::Boolean(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::Float(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::InlineTable(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::IntegerBin(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::IntegerDec(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::IntegerHex(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::IntegerOct(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::LiteralString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::LocalDate(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::LocalDateTime(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::LocalTime(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::MultiLineBasicString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::MultiLineLiteralString(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
            tombi_ast_syntax::Value::OffsetDateTime(value) => WithAlignmentHint {
                value,
                equal_alignment_width: self.equal_alignment_width,
                trailing_comment_alignment_width: self.trailing_comment_alignment_width,
            }
            .format(f),
        }
    }
}

trait LiteralNode<'t> {
    fn token(&self) -> Option<SyntaxToken<'t>>;
}

impl<'t, T> Format for T
where
    T: LiteralNode<'t> + tombi_ast_syntax::AstNode<'t>,
{
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl<'t, T> Format for WithAlignmentHint<&T>
where
    T: LiteralNode<'t> + tombi_ast_syntax::AstNode<'t>,
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
