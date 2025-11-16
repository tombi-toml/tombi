use itertools::Itertools;
use std::fmt::Write;

use tombi_ast::AstNode;
use tombi_config::StringQuoteStyle;

use super::LiteralNode;
use crate::{
    format::{write_trailing_comment_alignment_space, Format},
    types::WithAlignmentHint,
};

impl Format for tombi_ast::BasicString {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<'_, tombi_ast::BasicString> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let value = self.value;
        value.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        let text = value.token().unwrap().text().to_owned();
        let text = match f.string_quote_style() {
            StringQuoteStyle::Double | StringQuoteStyle::Preserve => text,
            StringQuoteStyle::Single => {
                // TODO: Only supports simple conditions, so it needs to be changed to behavior closer to black
                if text.contains("\\") || text.contains("'") {
                    text
                } else {
                    format!("'{}'", &text[1..text.len() - 1])
                }
            }
        };
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

impl Format for tombi_ast::LiteralString {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<'_, tombi_ast::LiteralString> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let value = self.value;
        value.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        let text = value.token().unwrap().text().to_owned();
        let text = match f.string_quote_style() {
            StringQuoteStyle::Single | StringQuoteStyle::Preserve => text,
            StringQuoteStyle::Double => {
                // TODO: Only supports simple conditions, so it needs to be changed to behavior closer to black
                if text.contains("\\") || text.contains("\"") {
                    text
                } else {
                    format!("\"{}\"", &text[1..text.len() - 1])
                }
            }
        };
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
impl LiteralNode for tombi_ast::MultiLineBasicString {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

impl LiteralNode for tombi_ast::MultiLineLiteralString {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_format, Formatter};
    use tombi_config::{format::FormatRules, FormatOptions, StringQuoteStyle};

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
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        string_quote_style: Some(StringQuoteStyle::Single),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(r#"key = 'value'"#)
    }

    test_format! {
        #[tokio::test]
        async fn basic_string_value_quote_style_single2(
            r#"key = "'value'""#,
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        string_quote_style: Some(StringQuoteStyle::Single),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(source)
    }
}
