macro_rules! token_types {
    (
        standard {
            $(($standard:ident, $standard_string:literal)),*$(,)?
        }
        custom {
            $(($custom:ident, $string:literal)),*$(,)?
        }
    ) => {
        #[allow(clippy::upper_case_acronyms)]
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub enum TokenType {
            $($standard,)*
            $($custom),*
        }

        /// Semantic token type names advertised in the server's legend.
        pub const SUPPORTED_TOKEN_TYPES: &[&str] = &[
            $($standard_string,)*
            $($string),*
        ];
    }
}

token_types! {
    standard {
        (STRING, "string"),
        (NUMBER, "number"),
        (OPERATOR, "operator"),
        (COMMENT, "comment"),
        (KEYWORD, "keyword"),
    }
    custom {
        (TABLE, "table"),
        (KEY, "key"),
        (BOOLEAN, "boolean"),
        // NOTE: "datetime" does not exist, so we will use "regexp" instead.
        (OFFSET_DATE_TIME, "offsetDateTime"),
        (LOCAL_DATE_TIME, "localDateTime"),
        (LOCAL_DATE, "localDate"),
        (LOCAL_TIME, "localTime"),
    }
}
