use itertools::Itertools;
use std::fmt::Write;

use tombi_ast::AstNode;

use super::LiteralNode;
use crate::{format::write_trailing_comment_alignment_space, types::WithAlignmentHint, Format};

macro_rules! impl_date_time_format {
    (impl Format for $type:ty;) => {
        impl Format for $type {
            #[inline]
            fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
                WithAlignmentHint::new(self).format(f)
            }
        }

        impl Format for WithAlignmentHint<'_, $type> {
            fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
                let value = self.value;
                value.leading_comments().collect_vec().format(f)?;

                let token = value.token().unwrap();
                let mut text = token.text().to_string();
                if let Some(delimiter) = f.date_time_delimiter() {
                    text.replace_range(10..11, &delimiter.to_string());
                }

                f.write_indent()?;
                write!(f, "{}", text)?;

                if let Some(comment) = value.trailing_comment() {
                    if let Some(trailing_comment_alignment_width) =
                        self.trailing_comment_alignment_width
                    {
                        write_trailing_comment_alignment_space(
                            f,
                            trailing_comment_alignment_width,
                        )?;
                    }
                    comment.format(f)?;
                }

                Ok(())
            }
        }
    };
}

impl_date_time_format! {
    impl Format for tombi_ast::OffsetDateTime;
}

impl_date_time_format! {
    impl Format for tombi_ast::LocalDateTime;
}

impl LiteralNode for tombi_ast::LocalDate {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

impl LiteralNode for tombi_ast::LocalTime {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_format, Formatter};

    test_format! {
        #[tokio::test]
        async fn offset_datetime_key_value1("odt1 = 1979-05-27T07:32:00Z") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn offset_datetime_key_value2("odt2 = 1979-05-27T00:32:00-07:00") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn offset_datetime_key_value3("odt3 = 1979-05-27T00:32:00.999999-07:00") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn offset_datetime_key_value4("odt4 = 1979-05-27 00:32:00.999999-07:00") -> Ok("odt4 = 1979-05-27T00:32:00.999999-07:00")
    }

    test_format! {
        #[tokio::test]
        async fn local_datetime_key_value1("ldt1 = 1979-05-27T07:32:00") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn local_datetime_key_value2("ldt2 = 1979-05-27T00:32:00.999999") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn local_datetime_key_value3("ldt3 = 1979-05-27 00:32:00.999999") -> Ok("ldt3 = 1979-05-27T00:32:00.999999")
    }

    test_format! {
        #[tokio::test]
        async fn valid_local_date_key_value("ld1 = 1979-05-27") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn valid_local_time_key_value1("lt1 = 07:32:00") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn valid_local_time_key_value2("lt2 = 00:32:00.999999") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn retain_pico_seconds("lt2 = 00:00:00.999999999999") -> Ok(source)
    }
}
