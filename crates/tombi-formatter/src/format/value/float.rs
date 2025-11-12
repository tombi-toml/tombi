use super::LiteralNode;

impl LiteralNode for tombi_ast::Float {
    fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        self.token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_format, Formatter};

    test_format! {
        #[tokio::test]
        async fn float_key_value1("key = +1.0") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value2("key = 3.1415") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value3("key = -0.01") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value4("key = 5e+22") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value5("key = 1e06") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value6("key = -2E-2") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value7("key = 6.626e-34") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn float_key_value8("key = 224_617.445_991_228") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn invalid_key_value1("invalid_float_1 = .7") -> Err(_)
    }

    test_format! {
        #[tokio::test]
        async fn invalid_key_value2("invalid_float_2 = 7.") -> Err(_)
    }

    test_format! {
        #[tokio::test]
        async fn invalid_key_value3("invalid_float_3 = 3.e+20") -> Err(_)
    }
}
