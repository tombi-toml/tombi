use std::fmt::Write;

use tombi_ast::AstNode;
use tombi_config::QuoteStyle;

use super::LiteralNode;
use crate::format::Format;

impl Format for tombi_ast::BasicString {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        self.leading_comments().collect::<Vec<_>>().format(f)?;

        f.write_indent()?;
        let text = self.token().unwrap().text().to_owned();
        let text = match f.quote_style() {
            QuoteStyle::Double | QuoteStyle::Preserve => text,
            QuoteStyle::Single => {
                // TODO: Only supports simple conditions, so it needs to be changed to behavior closer to black
                if text.contains("\\") || text.contains("'") {
                    text
                } else {
                    format!("'{}'", &text[1..text.len() - 1])
                }
            }
        };
        write!(f, "{text}")?;

        if let Some(comment) = self.tailing_comment() {
            comment.format(f)?;
        }

        Ok(())
    }
}

impl Format for tombi_ast::LiteralString {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        self.leading_comments().collect::<Vec<_>>().format(f)?;

        f.write_indent()?;
        let text = self.token().unwrap().text().to_owned();
        let text = match f.quote_style() {
            QuoteStyle::Single | QuoteStyle::Preserve => text,
            QuoteStyle::Double => {
                // TODO: Only supports simple conditions, so it needs to be changed to behavior closer to black
                if text.contains("\\") || text.contains("\"") {
                    text
                } else {
                    format!("\"{}\"", &text[1..text.len() - 1])
                }
            }
        };
        write!(f, "{text}")?;

        if let Some(comment) = self.tailing_comment() {
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
    use tombi_config::{QuoteStyle, TomlVersion};

    use crate::{test_format, FormatDefinitions};

    test_format! {
        #[test]
        fn basic_string_value1(r#"key = "value""#) -> Ok(source);
    }

    test_format! {
        #[test]
        fn basic_string_value2(r#"key    = "value""#) -> Ok(r#"key = "value""#);
    }

    test_format! {
        #[test]
        fn basic_string_value_quote_style_single1(
            r#"key = "value""#,
            TomlVersion::default(),
            &FormatDefinitions {
                quote_style: Some(QuoteStyle::Single),
                ..Default::default()
            }
        ) -> Ok(r#"key = 'value'"#);
    }

    test_format! {
        #[test]
        fn basic_string_value_quote_style_single2(
            r#"key = "'value'""#,
            TomlVersion::default(),
            &FormatDefinitions {
                quote_style: Some(QuoteStyle::Single),
                ..Default::default()
            }
        ) -> Ok(source);
    }
}
