impl<'t> crate::Value<'t> {
    pub fn span(&self) -> tombi_text::Span {
        match self {
            Self::Boolean(boolean) => boolean.span(),
            Self::IntegerBin(integer) => integer.span(),
            Self::IntegerOct(integer) => integer.span(),
            Self::IntegerDec(integer) => integer.span(),
            Self::IntegerHex(integer) => integer.span(),
            Self::Float(float) => float.span(),
            Self::BasicString(string) => string.span(),
            Self::LiteralString(string) => string.span(),
            Self::MultiLineBasicString(string) => string.span(),
            Self::MultiLineLiteralString(string) => string.span(),
            Self::OffsetDateTime(datetime) => datetime.span(),
            Self::LocalDateTime(datetime) => datetime.span(),
            Self::LocalDate(date) => date.span(),
            Self::LocalTime(time) => time.span(),
            Self::Array(array) => array.span(),
            Self::InlineTable(table) => table.span(),
        }
    }

    pub fn token_span(&self) -> tombi_text::Span {
        match self {
            Self::Boolean(boolean) => boolean.token().unwrap().span(),
            Self::IntegerBin(integer) => integer.token().unwrap().span(),
            Self::IntegerOct(integer) => integer.token().unwrap().span(),
            Self::IntegerDec(integer) => integer.token().unwrap().span(),
            Self::IntegerHex(integer) => integer.token().unwrap().span(),
            Self::Float(float) => float.token().unwrap().span(),
            Self::BasicString(string) => string.token().unwrap().span(),
            Self::LiteralString(string) => string.token().unwrap().span(),
            Self::MultiLineBasicString(string) => string.token().unwrap().span(),
            Self::MultiLineLiteralString(string) => string.token().unwrap().span(),
            Self::OffsetDateTime(datetime) => datetime.token().unwrap().span(),
            Self::LocalDateTime(datetime) => datetime.token().unwrap().span(),
            Self::LocalDate(date) => date.token().unwrap().span(),
            Self::LocalTime(time) => time.token().unwrap().span(),
            Self::Array(array) => {
                array.bracket_start().unwrap().span() + array.bracket_end().unwrap().span()
            }
            Self::InlineTable(table) => {
                table.brace_start().unwrap().span() + table.brace_end().unwrap().span()
            }
        }
    }
}
