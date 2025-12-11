// This file contains tests for the JSON lexer.
// The tests are written in a style similar to the TOML lexer tests,
// using macros to define test cases in a declarative way.

use itertools::Itertools;
use tombi_json_lexer::{ErrorKind, Token, tokenize};
use tombi_json_syntax::SyntaxKind::*;

macro_rules! test_tokens {
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

macro_rules! test_token {
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
                        tombi_json_lexer::Error::new(
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

// Basic token tests
test_tokens! {
    #[test]
    fn empty_source("") -> [];
}

test_tokens! {
    #[test]
    fn basic_tokens("{},[]:") -> [
        Token(BRACE_START, "{"),
        Token(BRACE_END, "}"),
        Token(COMMA, ","),
        Token(BRACKET_START, "["),
        Token(BRACKET_END, "]"),
        Token(COLON, ":"),
    ];
}

test_tokens! {
    #[test]
    fn array_with_numbers("[1, 2.5, 3]") -> [
        Token(BRACKET_START, "["),
        Token(NUMBER, "1"),
        Token(COMMA, ","),
        Token(WHITESPACE, " "),
        Token(NUMBER, "2.5"),
        Token(COMMA, ","),
        Token(WHITESPACE, " "),
        Token(NUMBER, "3"),
        Token(BRACKET_END, "]"),
    ];
}

test_tokens! {
    #[test]
    fn exponential_numbers_array("[1e2, 3.14e-2, 2.5e+10]") -> [
        Token(BRACKET_START, "["),
        Token(NUMBER, "1e2"),
        Token(COMMA, ","),
        Token(WHITESPACE, " "),
        Token(NUMBER, "3.14e-2"),
        Token(COMMA, ","),
        Token(WHITESPACE, " "),
        Token(NUMBER, "2.5e+10"),
        Token(BRACKET_END, "]"),
    ];
}

test_tokens! {
    #[test]
    fn nested_objects(r#"{"outer":{"inner":42}}"#) -> [
        Token(BRACE_START, "{"),
        Token(STRING, r#""outer""#),
        Token(COLON, ":"),
        Token(BRACE_START, "{"),
        Token(STRING, r#""inner""#),
        Token(COLON, ":"),
        Token(NUMBER, "42"),
        Token(BRACE_END, "}"),
        Token(BRACE_END, "}"),
    ];
}

// Complex JSON test with structure validation
test_tokens! {
    #[test]
    fn simple_json(r#"{
  "name": "John",
  "age": 30,
  "isAdmin": false,
  "address": null,
  "skills": ["programming", "design"]
}
"#) -> [
        Token(BRACE_START, "{"),
        Token(LINE_BREAK, "\n"),
        Token(WHITESPACE, "  "),
        Token(STRING, r#""name""#),
        Token(COLON, ":"),
        Token(WHITESPACE, " "),
        Token(STRING, r#""John""#),
        Token(COMMA, ","),
        Token(LINE_BREAK, "\n"),
        Token(WHITESPACE, "  "),
        Token(STRING, r#""age""#),
        Token(COLON, ":"),
        Token(WHITESPACE, " "),
        Token(NUMBER, "30"),
        Token(COMMA, ","),
        Token(LINE_BREAK, "\n"),
        Token(WHITESPACE, "  "),
        Token(STRING, r#""isAdmin""#),
        Token(COLON, ":"),
        Token(WHITESPACE, " "),
        Token(BOOLEAN, "false"),
        Token(COMMA, ","),
        Token(LINE_BREAK, "\n"),
        Token(WHITESPACE, "  "),
        Token(STRING, r#""address""#),
        Token(COLON, ":"),
        Token(WHITESPACE, " "),
        Token(NULL, "null"),
        Token(COMMA, ","),
        Token(LINE_BREAK, "\n"),
        Token(WHITESPACE, "  "),
        Token(STRING, r#""skills""#),
        Token(COLON, ":"),
        Token(WHITESPACE, " "),
        Token(BRACKET_START, "["),
        Token(STRING, r#""programming""#),
        Token(COMMA, ","),
        Token(WHITESPACE, " "),
        Token(STRING, r#""design""#),
        Token(BRACKET_END, "]"),
        Token(LINE_BREAK, "\n"),
        Token(BRACE_END, "}"),
        Token(LINE_BREAK, "\n"),
    ];
}

test_tokens! {
    #[test]
    fn complex_structure(r#"{"array":[1,2,3],"object":{"key":"value"}}"#) -> [
        Token(BRACE_START, "{"),
        Token(STRING, "\"array\""),
        Token(COLON, ":"),
        Token(BRACKET_START, "["),
        Token(NUMBER, "1"),
        Token(COMMA, ","),
        Token(NUMBER, "2"),
        Token(COMMA, ","),
        Token(NUMBER, "3"),
        Token(BRACKET_END, "]"),
        Token(COMMA, ","),
        Token(STRING, "\"object\""),
        Token(COLON, ":"),
        Token(BRACE_START, "{"),
        Token(STRING, "\"key\""),
        Token(COLON, ":"),
        Token(STRING, "\"value\""),
        Token(BRACE_END, "}"),
        Token(BRACE_END, "}"),
    ];
}

// Tests for single tokens - Numbers
test_token! {
    #[test]
    fn number_integer("42") -> Ok(Token(NUMBER, (0, 2)));
}

test_token! {
    #[test]
    fn number_negative_integer("-42") -> Ok(Token(NUMBER, (0, 3)));
}

test_token! {
    #[test]
    fn number_float("3.14") -> Ok(Token(NUMBER, (0, 4)));
}

test_token! {
    #[test]
    fn number_exponential("2.5e+10") -> Ok(Token(NUMBER, (0, 7)));
}

test_token! {
    #[test]
    fn number_exponential_uppercase("1.2E-3") -> Ok(Token(NUMBER, (0, 6)));
}

test_token! {
    #[test]
    fn number_exponential_no_sign("1e2") -> Ok(Token(NUMBER, (0, 3)));
}

test_token! {
    #[test]
    fn number_zero("0") -> Ok(Token(NUMBER, (0, 1)));
}

test_token! {
    #[test]
    fn number_decimal_point_leading_zero("0.123") -> Ok(Token(NUMBER, (0, 5)));
}

// Tests for single tokens - Strings
test_token! {
    #[test]
    fn string_simple(r#""hello""#) -> Ok(Token(STRING, (0, 7)));
}

test_token! {
    #[test]
    fn string_with_escaped_quotes(r#""escape\"quotes""#) -> Ok(Token(STRING, (0, 16)));
}

test_token! {
    #[test]
    fn string_empty(r#""""#) -> Ok(Token(STRING, (0, 2)));
}

test_token! {
    #[test]
    fn string_with_unicode(r#""\u00A9""#) -> Ok(Token(STRING, (0, 8)));
}

// Tests for single tokens - Other primitives
test_token! {
    #[test]
    fn boolean_true("true") -> Ok(Token(BOOLEAN, (0, 4)));
}

test_token! {
    #[test]
    fn boolean_false("false") -> Ok(Token(BOOLEAN, (0, 5)));
}

test_token! {
    #[test]
    fn null_value("null") -> Ok(Token(NULL, (0, 4)));
}

// Error test cases
test_token! {
    #[test]
    fn error_unterminated_string(r#""hello"#) -> Err(Token(ErrorKind::InvalidString, (0, 6)));
}

test_token! {
    #[test]
    fn error_invalid_number("01") -> Err(Token(ErrorKind::InvalidNumber, (0, 2)));
}

test_token! {
    #[test]
    fn string_with_unrecognized_escape(r#""\z""#) -> Err(Token(ErrorKind::InvalidString, (0, 4)));
}

// Additional tests for JSON specification edge cases

// Tests for various escape sequences in strings
test_token! {
    #[test]
    fn string_with_common_escapes(r#""\n\t\r\b\f""#) -> Ok(Token(STRING, (0, 12)));
}

test_token! {
    #[test]
    fn string_with_escaped_solidus(r#""\/""#) -> Ok(Token(STRING, (0, 4)));
}

test_token! {
    #[test]
    fn string_with_escaped_backslash(r#""\\""#) -> Ok(Token(STRING, (0, 4)));
}

// Tests for Unicode escapes
test_token! {
    #[test]
    fn string_with_unicode_emoji(r#""\uD83D\uDE00""#) -> Ok(Token(STRING, (0, 14)));
}

test_token! {
    #[test]
    fn string_with_unicode_surrogate_pair(r#""\uD834\uDD1E""#) -> Ok(Token(STRING, (0, 14)));
}

// Tests for whitespace handling
test_tokens! {
    #[test]
    fn whitespace_between_tokens("{ \t\n\r }") -> [
        Token(BRACE_START, "{"),
        Token(WHITESPACE, " \t"),
        Token(LINE_BREAK, "\n\r"),
        Token(WHITESPACE, " "),
        Token(BRACE_END, "}"),
    ];
}

// Error cases for invalid JSON constructs
test_token! {
    #[test]
    fn error_unescaped_control_char("\"\u{0001}\"") -> Err(Token(ErrorKind::InvalidString, (0, 3)));
}

test_token! {
    #[test]
    fn error_invalid_unicode_escape(r#""\uXYZA""#) -> Err(Token(ErrorKind::InvalidString, (0, 8)));
}

test_token! {
    #[test]
    fn error_incomplete_unicode_escape(r#""\u123""#) -> Err(Token(ErrorKind::InvalidString, (0, 7)));
}

// Numbers with special edge cases
test_token! {
    #[test]
    fn number_negative_zero("-0") -> Ok(Token(NUMBER, (0, 2)));
}

test_token! {
    #[test]
    fn number_fractional_no_integer("0.123") -> Ok(Token(NUMBER, (0, 5)));
}

test_token! {
    #[test]
    fn error_plus_prefix("+10") -> Err(Token(ErrorKind::InvalidToken, (0, 3)));
}

test_token! {
    #[test]
    fn error_number_trailing_decimal("10.") -> Err(Token(ErrorKind::InvalidNumber, (0, 3)));
}

test_tokens! {
    #[test]
    fn nested_arrays_deep("[[[[[[]]]]]]") -> [
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BRACKET_START, "["),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
        Token(BRACKET_END, "]"),
    ];
}

// Test for JSON with all primitive types
test_tokens! {
    #[test]
    fn all_primitive_types(r#"{"string":"value","number":42,"float":3.14,"bool":true,"null":null}"#) -> [
        Token(BRACE_START, "{"),
        Token(STRING, r#""string""#),
        Token(COLON, ":"),
        Token(STRING, r#""value""#),
        Token(COMMA, ","),
        Token(STRING, r#""number""#),
        Token(COLON, ":"),
        Token(NUMBER, "42"),
        Token(COMMA, ","),
        Token(STRING, r#""float""#),
        Token(COLON, ":"),
        Token(NUMBER, "3.14"),
        Token(COMMA, ","),
        Token(STRING, r#""bool""#),
        Token(COLON, ":"),
        Token(BOOLEAN, "true"),
        Token(COMMA, ","),
        Token(STRING, r#""null""#),
        Token(COLON, ":"),
        Token(NULL, "null"),
        Token(BRACE_END, "}"),
    ];
}
