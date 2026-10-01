use itertools::Itertools;
use std::fmt::Write;

use tombi_ast_syntax::AstNode;

use super::LiteralNode;
use crate::{
    format::{
        Format, format_basic_string_quote_style, format_literal_string_quote_style,
        write_trailing_comment_alignment_space,
    },
    types::WithAlignmentHint,
};

impl<'t> Format for tombi_ast_syntax::BasicString<'t> {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl<'t> Format for WithAlignmentHint<&tombi_ast_syntax::BasicString<'t>> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let value = self.value;
        value.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        let token = value.token().unwrap();
        let text = format_basic_string_quote_style(token.text(), f.string_quote_style());
        write!(f, "{text}")?;

        if let Some(comment) = value.trailing_comment() {
            if let Some(trailing_comment_alignment_width) = self.trailing_comment_alignment_width {
                write_trailing_comment_alignment_space(f, trailing_comment_alignment_width)?;
            }
            comment.format(f)?;
        }

        Ok(())
    }
}

impl<'t> Format for tombi_ast_syntax::LiteralString<'t> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl<'t> Format for WithAlignmentHint<&tombi_ast_syntax::LiteralString<'t>> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let value = self.value;
        value.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        let token = value.token().unwrap();
        let text = format_literal_string_quote_style(token.text(), f.string_quote_style());
        write!(f, "{text}")?;

        if let Some(comment) = value.trailing_comment() {
            if let Some(trailing_comment_alignment_width) = self.trailing_comment_alignment_width {
                write_trailing_comment_alignment_space(f, trailing_comment_alignment_width)?;
            }
            comment.format(f)?;
        }

        Ok(())
    }
}
impl<'t> LiteralNode<'t> for tombi_ast_syntax::MultiLineBasicString<'t> {
    fn token(&self) -> Option<tombi_ast_syntax::SyntaxToken<'t>> {
        self.token()
    }
}

impl<'t> LiteralNode<'t> for tombi_ast_syntax::MultiLineLiteralString<'t> {
    fn token(&self) -> Option<tombi_ast_syntax::SyntaxToken<'t>> {
        self.token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Formatter, test_format};
    use tombi_config::{StringQuoteStyle, format::FormatRules};

    test_format! {
        #[tokio::test]
        async fn basic_string_value1(r#"key = "value""#) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn basic_string_value2(r#"key    = "value""#) -> Ok(r#"key = "value""#)
    }

    test_format! {
        #[tokio::test]
        async fn basic_string_value_quote_style_single1(
            r#"key = "value""#,
            FormatOptions {
                rules: Some(FormatRules {
                    string_quote_style: Some(StringQuoteStyle::Single),
                    ..Default::default()
                }),
            }
        ) -> Ok(r#"key = 'value'"#)
    }

    test_format! {
        #[tokio::test]
        async fn basic_string_value_quote_style_single2(
            r#"key = "'value'""#,
            FormatOptions {
                rules: Some(FormatRules {
                    string_quote_style: Some(StringQuoteStyle::Single),
                    ..Default::default()
                }),
            }
        ) -> Ok(source)
    }
}
