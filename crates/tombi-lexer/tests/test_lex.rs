use itertools::Itertools;
use tombi_lexer::{ErrorKind::*, Token, tokenize};
use tombi_syntax::SyntaxKind::*;

macro_rules! test_lex_tokens {
    {#[test]fn $name:ident($source:expr) -> [
        $(Token($kind:expr, $text:literal),)*
    ];} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_tracing();

            let tokens = tokenize($source).collect_vec();
            let (expected, _) = [
                $(
                    ($kind, $text),
                )*
            ]
            .into_iter()
            .fold((vec![], (0, tombi_text::Position::MIN)), |(mut acc, (start_offset, start_position)), (kind, text)| {
                let text: &str = text;
                let end_offset = start_offset + (text.len() as u32);
                let end_position = start_position + tombi_text::RelativePosition::of(text);
                acc.push(
                    Ok(
                        Token::new(
                            kind,
                            (
                                (start_offset, end_offset).into(),
                                (start_position, end_position).into()
                            )
                        )
                    )
                );
                (acc, (end_offset, end_position))
            });
            pretty_assertions::assert_eq!(tokens, expected);
        }
    };
}

macro_rules! test_lex_token {
    {#[test]fn $name:ident($source:expr) -> Ok(Token($kind:expr, ($start_offset:expr, $end_offset:expr)));} => {
        #[test]
        fn $name() {
            let source = textwrap::dedent($source);
            let source = source.trim();
            let tokens = tokenize(&source).collect_vec();
            let start_position = tombi_text::Position::MIN;
            let end_position = start_position + tombi_text::RelativePosition::of(source);

            pretty_assertions::assert_eq!(
                tokens,
                [
                    Ok(
                        Token::new(
                            $kind,
                            (
                                ($start_offset, $end_offset).into(),
                                (start_position, end_position).into()
                            )
                        )
                    )
                ]
            );
        }
    };

    {#[test]fn $name:ident($source:expr) -> Err(Token($kind:expr, ($start_offset:expr, $end_offset:expr)));} => {
        #[test]
        fn $name() {
            let source = textwrap::dedent($source);
            let source = source.trim();
            let tokens = tokenize(&source).collect_vec();
            let start_position = tombi_text::Position::MIN;
            let end_position = start_position + tombi_text::RelativePosition::of(source);

            pretty_assertions::assert_eq!(
                tokens,
                [
                    Err(
                        tombi_lexer::Error::new(
                            $kind,
                            (
                                ($start_offset, $end_offset).into(),
                                (start_position, end_position).into()
                            )
                        )
                    )
                ]
            );
        }
    }
}

test_lex_tokens! {
    #[test]
    fn empty_source("") -> [];
}

test_lex_tokens! {
    #[test]
    fn comment_line_break("# This is a comment\n") -> [
        Token(COMMENT, "# This is a comment"),
        Token(LINE_BREAK, "\n"),
    ];
}

test_lex_tokens! {
    #[test]
    fn comment_line_break_crlf("# This is a comment\r\n") -> [
        Token(COMMENT, "# This is a comment"),
        Token(LINE_BREAK, "\r\n"),
    ];
}

test_lex_tokens! {
    #[test]
    fn whitespace_comment_line_break("   # This is a comment\n") -> [
        Token(WHITESPACE, "   "),
        Token(COMMENT, "# This is a comment"),
        Token(LINE_BREAK, "\n"),
    ];
}

test_lex_tokens! {
    #[test]
    fn whitespace_comment_line_break_crlf("   # This is a comment\r\n") -> [
        Token(WHITESPACE, "   "),
        Token(COMMENT, "# This is a comment"),
        Token(LINE_BREAK, "\r\n"),
    ];
}

test_lex_tokens! {
    #[test]
    fn comment_whitespace_line_break("# This is a comment  \n") -> [
        Token(COMMENT, "# This is a comment  "),
        Token(LINE_BREAK, "\n"),
    ];
}

test_lex_tokens! {
    #[test]
    fn tokens("{},.[]=") -> [
        Token(BRACE_START, "{"),
        Token(BRACE_END, "}"),
        Token(COMMA, ","),
        Token(DOT, "."),
        Token(BRACKET_START, "["),
        Token(BRACKET_END, "]"),
        Token(EQUAL, "="),
    ];
}

test_lex_tokens! {
    #[test]
    fn key_value_float_dot_key("3.14159 = \"pi\"") -> [
        Token(FLOAT, "3.14159"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"pi\""),
    ];
}

test_lex_tokens! {
    #[test]
    fn key_value_dotted_keys("apple.type = \"fruit\"") -> [
        Token(BARE_KEY, "apple"),
        Token(DOT, "."),
        Token(BARE_KEY, "type"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"fruit\""),
    ];
}

test_lex_tokens! {
    #[test]
    fn table_only_header(r#"[package]"#) -> [
        Token(BRACKET_START, "["),
        Token(BARE_KEY, "package"),
        Token(BRACKET_END, "]"),
    ];
}

test_lex_tokens! {
    #[test]
    fn table_empty_header(r#"[]"#) -> [
        Token(BRACKET_START, "["),
        Token(BRACKET_END, "]"),
    ];
}

test_lex_tokens! {
    #[test]
    fn table_hyphen_header(r#"[-]"#) -> [
        Token(BRACKET_START, "["),
        Token(BARE_KEY, "-"),
        Token(BRACKET_END, "]"),
    ];
}

test_lex_tokens! {
    #[test]
    fn table(
        textwrap::dedent(
            r#"
            [package]
            name = "toml"
            version = "0.5.8"
            "#
        ).trim()
    ) -> [
            Token(BRACKET_START, "["),
            Token(BARE_KEY, "package"),
            Token(BRACKET_END, "]"),
            Token(LINE_BREAK, "\n"),
            Token(BARE_KEY, "name"),
            Token(WHITESPACE, " "),
            Token(EQUAL, "="),
            Token(WHITESPACE, " "),
            Token(BASIC_STRING, "\"toml\""),
            Token(LINE_BREAK, "\n"),
            Token(BARE_KEY, "version"),
            Token(WHITESPACE, " "),
            Token(EQUAL, "="),
            Token(WHITESPACE, " "),
            Token(BASIC_STRING, "\"0.5.8\""),
        ];
}

test_lex_tokens! {
    #[test]
    fn inline_table("key = { key2 = \"value\" }") -> [
        Token(BARE_KEY, "key"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BRACE_START, "{"),
        Token(WHITESPACE, " "),
        Token(BARE_KEY, "key2"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"value\""),
        Token(WHITESPACE, " "),
        Token(BRACE_END, "}"),
    ];
}

test_lex_tokens! {
    #[test]
    fn array_of_table(
        textwrap::dedent(
            r#"
            [[package]]
            name = "toml"
            version = "0.5.8"

            [[package]]
            name = "json"
            version = "1.2.4"
            "#
        ).trim()
    ) -> [
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BARE_KEY, "package"),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
        Token(LINE_BREAK, "\n"),
        Token(BARE_KEY, "name"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"toml\""),
        Token(LINE_BREAK, "\n"),
        Token(BARE_KEY, "version"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"0.5.8\""),
        Token(LINE_BREAK, "\n"),
        Token(LINE_BREAK, "\n"),
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BARE_KEY, "package"),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
        Token(LINE_BREAK, "\n"),
        Token(BARE_KEY, "name"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"json\""),
        Token(LINE_BREAK, "\n"),
        Token(BARE_KEY, "version"),
        Token(WHITESPACE, " "),
        Token(EQUAL, "="),
        Token(WHITESPACE, " "),
        Token(BASIC_STRING, "\"1.2.4\""),
    ];
}

test_lex_token! {
    #[test]
    fn only_comment("# This is a comment") -> Ok(Token(COMMENT, (0, 19)));
}

test_lex_token! {
    #[test]
    fn basic_string1(r#""Hello, World!""#) -> Ok(Token(BASIC_STRING, (0, 15)));
}

test_lex_token! {
    #[test]
    fn basic_string2(r#""Hello, \"Taro\"!""#) -> Ok(Token(BASIC_STRING, (0, 18)));
}

test_lex_token! {
    #[test]
    fn multi_line_basic_string1(r#""""aaaa""""#) -> Ok(Token(MULTI_LINE_BASIC_STRING, (0, 10)));
}

test_lex_token! {
    #[test]
    fn multi_line_basic_string2(
        r#"
        """
        aaaa
        """
        "#
    ) -> Ok(Token(MULTI_LINE_BASIC_STRING, (0, 12)));
}

test_lex_token! {
    #[test]
    fn multi_line_basic_string3(r#""""Here are fifteen quotation marks: ""\"""\"""\"""\"""\".""""#) -> Ok(
        Token(MULTI_LINE_BASIC_STRING, (0, 61))
    );
}

test_lex_token! {
    #[test]
    fn multi_line_basic_string4(r#""""""""#) -> Ok(Token(MULTI_LINE_BASIC_STRING, (0, 6)));
}

test_lex_token! {
    #[test]
    fn multi_line_basic_string5(r#""""""""""#) -> Ok(Token(MULTI_LINE_BASIC_STRING, (0, 8)));
}

test_lex_token! {
    #[test]
    fn invalid_multi_line_basic_string  (r#"""""""""""#) -> Err(
        Token(InvalidMultilineBasicString, (0, 9))
    );
}

test_lex_token! {
    #[test]
    fn invalid_multi_line_basic_string2(r#""""6 quotes: """""""#) -> Err(
        Token(InvalidMultilineBasicString, (0, 19))
    );
}

test_lex_token! {
    #[test]
    fn literal_string1("'Hello, World!'") -> Ok(Token(LITERAL_STRING, (0, 15)));
}

test_lex_token! {
    #[test]
    fn literal_string2(r#"'C:\Users\nodejs\templates'"#) -> Ok(Token(LITERAL_STRING, (0, 27)));
}

test_lex_token! {
    #[test]
    fn multi_line_literal_string("'''Here are fifteen apostrophes: '''''") -> Ok(
        Token(MULTI_LINE_LITERAL_STRING, (0, 38))
    );
}

test_lex_token! {
    #[test]
    fn invalid_multi_line_literal_string("'''Here are fifteen apostrophes: ''''''") -> Err(
        Token(InvalidMultilineLiteralString, (0, 39))
    );
}

test_lex_token! {
    #[test]
    fn offset_date_time1("2021-01-01T00:00:00Z") -> Ok(Token(OFFSET_DATE_TIME, (0, 20)));
}

test_lex_token! {
    #[test]
    fn offset_date_time2("2021-01-01T00:00:00+09:00") -> Ok(Token(OFFSET_DATE_TIME, (0, 25)));
}

test_lex_token! {
    #[test]
    fn offset_date_time3("2021-01-01T00:00:00-09:00") -> Ok(Token(OFFSET_DATE_TIME, (0, 25)));
}

test_lex_token! {
    #[test]
    fn offset_date_time4("2021-01-01T00:00:00.123456Z") -> Ok(Token(OFFSET_DATE_TIME, (0, 27)));
}

test_lex_token! {
    #[test]
    fn offset_date_time5("2021-01-01T00:00:00.123456+09:00") -> Ok(Token(OFFSET_DATE_TIME, (0, 32)));
}

test_lex_token! {
    #[test]
    fn offset_date_time6("2021-01-01T00:00:00.123456-09:00") -> Ok(Token(OFFSET_DATE_TIME, (0, 32)));
}

test_lex_token! {
    #[test]
    fn local_date_time1("2021-01-01 00:00:00") -> Ok(Token(LOCAL_DATE_TIME, (0, 19)));
}

test_lex_token! {
    #[test]
    fn local_date_time2("2021-01-01 00:00:00.123456") -> Ok(Token(LOCAL_DATE_TIME, (0, 26)));
}

test_lex_token! {
    #[test]
    fn local_date_time3("2021-01-01T00:00:00") -> Ok(Token(LOCAL_DATE_TIME, (0, 19)));
}

test_lex_token! {
    #[test]
    fn local_date_time4("2021-01-01T00:00:00.123456") -> Ok(Token(LOCAL_DATE_TIME, (0, 26)));
}

test_lex_token! {
    #[test]
    fn local_date1("2021-01-01") -> Ok(Token(LOCAL_DATE, (0, 10)));
}

test_lex_token! {
    #[test]
    fn local_time1("00:00:00") -> Ok(Token(LOCAL_TIME, (0, 8)));
}

test_lex_token! {
    #[test]
    fn local_time2("00:00:00.123456") -> Ok(Token(LOCAL_TIME, (0, 15)));
}

test_lex_token! {
    #[test]
    fn local_time_without_seconds("00:00") -> Ok(Token(LOCAL_TIME, (0, 5)));
}

test_lex_token! {
    #[test]
    fn boolean1("true") -> Ok(Token(BOOLEAN, (0, 4)));
}

test_lex_token! {
    #[test]
    fn boolean2("false") -> Ok(Token(BOOLEAN, (0, 5)));
}

test_lex_token! {
    #[test]
    fn key1("key") -> Ok(Token(BARE_KEY, (0, 3)));
}

test_lex_token! {
    #[test]
    fn key2("_1234567890") -> Ok(Token(BARE_KEY, (0, 11)));
}

test_lex_token! {
    #[test]
    fn key3("key_123") -> Ok(Token(BARE_KEY, (0, 7)));
}

test_lex_token! {
    #[test]
    fn integer_dec1("0") -> Ok(Token(INTEGER_DEC, (0, 1)));
}

test_lex_token! {
    #[test]
    fn integer_dec2("1") -> Ok(Token(INTEGER_DEC, (0, 1)));
}

test_lex_token! {
    #[test]
    fn integer_dec3("1234567890") -> Ok(Token(INTEGER_DEC, (0, 10)));
}

test_lex_token! {
    #[test]
    fn integer_dec4("+1234567890") -> Ok(Token(INTEGER_DEC, (0, 11)));
}

test_lex_token! {
    #[test]
    fn integer_dec5("-1234567890") -> Ok(Token(INTEGER_DEC, (0, 11)));
}

test_lex_token! {
    #[test]
    fn integer_dec6("1_234_567_890") -> Ok(Token(INTEGER_DEC, (0, 13)));
}

test_lex_token! {
    #[test]
    fn integer_dec7("+1_234_567_890") -> Ok(Token(INTEGER_DEC, (0, 14)));
}

test_lex_token! {
    #[test]
    fn integer_dec8("-1_234_567_890") -> Ok(Token(INTEGER_DEC, (0, 14)));
}

test_lex_token! {
    #[test]
    fn invalid_integer_dec1("+_1234567890") -> Err(Token(InvalidToken, (0, 12)));
}

test_lex_token! {
    #[test]
    fn invalid_integer_dec2("-_1234567890") -> Ok(Token(BARE_KEY, (0, 12)));
}

test_lex_token! {
    #[test]
    fn integer_bin1("0b0") -> Ok(Token(INTEGER_BIN, (0, 3)));
}

test_lex_token! {
    #[test]
    fn integer_bin2("0b1") -> Ok(Token(INTEGER_BIN, (0, 3)));
}

test_lex_token! {
    #[test]
    fn integer_bin3("0b01") -> Ok(Token(INTEGER_BIN, (0, 4)));
}

test_lex_token! {
    #[test]
    fn integer_bin4("0b10") -> Ok(Token(INTEGER_BIN, (0, 4)));
}

test_lex_token! {
    #[test]
    fn integer_bin5("0b101010") -> Ok(Token(INTEGER_BIN, (0, 8)));
}

test_lex_token! {
    #[test]
    fn integer_bin6("0b_1010_10") -> Ok(Token(BARE_KEY, (0, 10)));
}

test_lex_token! {
    #[test]
    fn integer_bin7("0b10_101_010") -> Ok(Token(INTEGER_BIN, (0, 12)));
}

test_lex_token! {
    #[test]
    fn integer_oct1("0o0") -> Ok(Token(INTEGER_OCT, (0, 3)));
}

test_lex_token! {
    #[test]
    fn integer_oct2("0o1") -> Ok(Token(INTEGER_OCT, (0, 3)));
}

test_lex_token! {
    #[test]
    fn integer_oct3("0o01") -> Ok(Token(INTEGER_OCT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn integer_oct4("0o10") -> Ok(Token(INTEGER_OCT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn integer_oct5("0o1234567") -> Ok(Token(INTEGER_OCT, (0, 9)));
}

test_lex_token! {
    #[test]
    fn integer_oct6("0o_1234_567") -> Ok(Token(BARE_KEY, (0, 11)));
}

test_lex_token! {
    #[test]
    fn integer_oct7("0o12_34_567") -> Ok(Token(INTEGER_OCT, (0, 11)));
}

test_lex_token! {
    #[test]
    fn integer_hex1("0x0") -> Ok(Token(INTEGER_HEX, (0, 3)));
}

test_lex_token! {
    #[test]
    fn integer_hex2("0x1") -> Ok(Token(INTEGER_HEX, (0, 3)));
}

test_lex_token! {
    #[test]
    fn integer_hex3("0x01") -> Ok(Token(INTEGER_HEX, (0, 4)));
}

test_lex_token! {
    #[test]
    fn integer_hex4("0x10") -> Ok(Token(INTEGER_HEX, (0, 4)));
}

test_lex_token! {
    #[test]
    fn integer_hex5("0x1234567890abcdef") -> Ok(Token(INTEGER_HEX, (0, 18)));
}

test_lex_token! {
    #[test]
    fn integer_hex6("0x_1234_5678_90ab_cdef") -> Ok(Token(BARE_KEY, (0, 22)));
}

test_lex_token! {
    #[test]
    fn integer_hex7("0x12_34_5678_90ab_cdef") -> Ok(Token(INTEGER_HEX, (0, 22)));
}

test_lex_token! {
    #[test]
    fn float1("+1.0") -> Ok(Token(FLOAT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn float2("3.1415") -> Ok(Token(FLOAT, (0, 6)));
}

test_lex_token! {
    #[test]
    fn float3("-0.01") -> Ok(Token(FLOAT, (0, 5)));
}

test_lex_token! {
    #[test]
    fn float4("5e+22") -> Ok(Token(FLOAT, (0, 5)));
}

test_lex_token! {
    #[test]
    fn float5("1e06") -> Ok(Token(FLOAT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn float6("-2E-2") -> Ok(Token(FLOAT, (0, 5)));
}

test_lex_token! {
    #[test]
    fn float7("6.626e-34") -> Ok(Token(FLOAT, (0, 9)));
}

test_lex_token! {
    #[test]
    fn float8("224_617.445_991_228") -> Ok(Token(FLOAT, (0, 19)));
}

test_lex_token! {
    #[test]
    fn special_float1("inf") -> Ok(Token(FLOAT, (0, 3)));
}

test_lex_token! {
    #[test]
    fn special_float2("nan") -> Ok(Token(FLOAT, (0, 3)));
}

test_lex_token! {
    #[test]
    fn special_float3("+inf") -> Ok(Token(FLOAT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn special_float4("+nan") -> Ok(Token(FLOAT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn special_float5("-inf") -> Ok(Token(FLOAT, (0, 4)));
}

test_lex_token! {
    #[test]
    fn special_float6("-nan") -> Ok(Token(FLOAT, (0, 4)));
}
