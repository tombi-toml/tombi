mod array;
mod boolean;
mod date_time;
mod float;
mod inline_table;
mod integer;
mod string;

use tombi_future::Boxable;

use crate::Lint;

impl Lint for tombi_ast::Value {
    fn lint<'a: 'b, 'b>(&'a self, l: &'a mut crate::Linter<'_>) -> tombi_future::BoxFuture<'b, ()> {
        async move {
            match self {
                Self::Boolean(value) => value.lint(l).await,
                Self::IntegerBin(value) => value.lint(l).await,
                Self::IntegerOct(value) => value.lint(l).await,
                Self::IntegerDec(value) => value.lint(l).await,
                Self::IntegerHex(value) => value.lint(l).await,
                Self::Float(value) => value.lint(l).await,
                Self::BasicString(value) => value.lint(l).await,
                Self::LiteralString(value) => value.lint(l).await,
                Self::MultiLineBasicString(value) => value.lint(l).await,
                Self::MultiLineLiteralString(value) => value.lint(l).await,
                Self::OffsetDateTime(value) => value.lint(l).await,
                Self::LocalDateTime(value) => value.lint(l).await,
                Self::LocalDate(value) => value.lint(l).await,
                Self::LocalTime(value) => value.lint(l).await,
                Self::Array(value) => value.lint(l).await,
                Self::InlineTable(value) => value.lint(l).await,
            }
        }
        .boxed()
    }
}
