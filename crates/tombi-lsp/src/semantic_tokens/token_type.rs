use tower_lsp::lsp_types::SemanticTokenType;

macro_rules! token_types {
    (
        standard {
            $($standard:ident),*$(,)?
        }
        custom {
            $(($custom:ident, $string:literal)),*$(,)?
        }
    ) => {
        pub mod token_type {
            use super::SemanticTokenType;

            $(pub(crate) const $custom: SemanticTokenType = SemanticTokenType::new($string);)*
        }

        #[allow(clippy::upper_case_acronyms)]
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub enum TokenType {
            $($standard,)*
            $($custom),*
        }

        pub const SUPPORTED_TOKEN_TYPES: &[SemanticTokenType] = &[
            $(SemanticTokenType::$standard,)*
            $(self::token_type::$custom),*
        ];
    }
}

token_types! {
    standard {
        STRING,
        NUMBER,
        OPERATOR,
        COMMENT,
        KEYWORD,
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
