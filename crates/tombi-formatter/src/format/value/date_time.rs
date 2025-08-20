use itertools::Itertools;
use std::fmt::Write;

use tombi_ast::AstNode;

use super::LiteralNode;
use crate::format::Format;

macro_rules! impl_date_time_format {
    (impl Format for $type:ty;) => {
        impl Format for $type {
            fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
                self.leading_comments().collect_vec().format(f)?;

                let token = self.token().unwrap();
                let mut text = token.text().to_string();
                if let Some(delimiter) = f.date_time_delimiter() {
                    text.replace_range(10..11, &delimiter.to_string());
                }

                f.write_indent()?;
                write!(f, "{}", text)?;

                if let Some(comment) = self.trailing_comment() {
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
    use crate::test_format;

    test_format! {
        #[test]
        fn offset_datetime_key_value1("odt1 = 1979-05-27T07:32:00Z") -> Ok(source);
    }

    test_format! {
        #[test]
        fn offset_datetime_key_value2("odt2 = 1979-05-27T00:32:00-07:00") -> Ok(source);
    }

    test_format! {
        #[test]
        fn offset_datetime_key_value3("odt3 = 1979-05-27T00:32:00.999999-07:00") -> Ok(source);
    }

    test_format! {
        #[test]
        fn offset_datetime_key_value4("odt4 = 1979-05-27 00:32:00.999999-07:00") -> Ok("odt4 = 1979-05-27T00:32:00.999999-07:00");
    }

    test_format! {
        #[test]
        fn local_datetime_key_value1("ldt1 = 1979-05-27T07:32:00") -> Ok(source);
    }

    test_format! {
        #[test]
        fn local_datetime_key_value2("ldt2 = 1979-05-27T00:32:00.999999") -> Ok(source);
    }

    test_format! {
        #[test]
        fn local_datetime_key_value3("ldt3 = 1979-05-27 00:32:00.999999") -> Ok("ldt3 = 1979-05-27T00:32:00.999999");
    }

    test_format! {
        #[test]
        fn valid_local_date_key_value("ld1 = 1979-05-27") -> Ok(source);
    }

    test_format! {
        #[test]
        fn valid_local_time_key_value1("lt1 = 07:32:00") -> Ok(source);
    }

    test_format! {
        #[test]
        fn valid_local_time_key_value2("lt2 = 00:32:00.999999") -> Ok(source);
    }

    test_format! {
        #[test]
        fn retain_pico_seconds("lt2 = 00:00:00.999999999999") -> Ok(source);
    }
}
