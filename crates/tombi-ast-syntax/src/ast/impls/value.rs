impl crate::Value {
    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::Boolean(boolean) => boolean.range(),
            Self::IntegerBin(integer) => integer.range(),
            Self::IntegerOct(integer) => integer.range(),
            Self::IntegerDec(integer) => integer.range(),
            Self::IntegerHex(integer) => integer.range(),
            Self::Float(float) => float.range(),
            Self::BasicString(string) => string.range(),
            Self::LiteralString(string) => string.range(),
            Self::MultiLineBasicString(string) => string.range(),
            Self::MultiLineLiteralString(string) => string.range(),
            Self::OffsetDateTime(datetime) => datetime.range(),
            Self::LocalDateTime(datetime) => datetime.range(),
            Self::LocalDate(date) => date.range(),
            Self::LocalTime(time) => time.range(),
            Self::Array(array) => array.range(),
            Self::InlineTable(table) => table.range(),
        }
    }

    pub fn token_range(&self) -> tombi_text::Range {
        match self {
            Self::Boolean(boolean) => boolean.token().unwrap().range(),
            Self::IntegerBin(integer) => integer.token().unwrap().range(),
            Self::IntegerOct(integer) => integer.token().unwrap().range(),
            Self::IntegerDec(integer) => integer.token().unwrap().range(),
            Self::IntegerHex(integer) => integer.token().unwrap().range(),
            Self::Float(float) => float.token().unwrap().range(),
            Self::BasicString(string) => string.token().unwrap().range(),
            Self::LiteralString(string) => string.token().unwrap().range(),
            Self::MultiLineBasicString(string) => string.token().unwrap().range(),
            Self::MultiLineLiteralString(string) => string.token().unwrap().range(),
            Self::OffsetDateTime(datetime) => datetime.token().unwrap().range(),
            Self::LocalDateTime(datetime) => datetime.token().unwrap().range(),
            Self::LocalDate(date) => date.token().unwrap().range(),
            Self::LocalTime(time) => time.token().unwrap().range(),
            Self::Array(array) => {
                array.bracket_start().unwrap().range() + array.bracket_end().unwrap().range()
            }
            Self::InlineTable(table) => {
                table.brace_start().unwrap().range() + table.brace_end().unwrap().range()
            }
        }
    }
}
