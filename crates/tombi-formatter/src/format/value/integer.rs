use super::LiteralNode;

impl LiteralNode for tombi_ast::IntegerBin {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

impl LiteralNode for tombi_ast::IntegerHex {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

impl LiteralNode for tombi_ast::IntegerDec {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

impl LiteralNode for tombi_ast::IntegerOct {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_format, Formatter};

    test_format! {
        #[tokio::test]
        async fn integer_bin_key_value1("bin1 = 0b11010110") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_bin_key_value2("bin2 = 0b1101_0110") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_oct_key_value1("oct1 = 0o01234567") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_oct_key_value2("oct2 = 0o755") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_hex_key_value1("hex1 = 0xDEADBEEF") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_hex_key_value2("hex2 = 0xdeadbeef") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_hex_key_value3("hex3 = 0xdead_beef") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value1("int1 = +99") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value2("int2 = 42") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value3("int3 = 0") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value4("int4 = -17") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value5("int5 = 1_000") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value6("int6 = 5_349_221") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value7("int7 = 53_49_221") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn integer_dec_key_value8("int8 = 1_2_3_4_5") -> Ok(source)
    }
}
