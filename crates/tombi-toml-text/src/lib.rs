use tombi_toml_version::TomlVersion;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("invalid escape sequence")]
    InvalidEscapeSequence,

    #[error("invalid Unicode escape sequence")]
    InvalidUnicodeEscapeSequence,

    #[error("invalid Unicode code point")]
    InvalidUnicodeCodePoint,

    #[error("invalid newline character in input")]
    InvalidNewline,

    #[error("invalid whitespace escape sequence")]
    InvalidWhitespaceEscapeSequence,

    #[error("LineBreak allows only LF or CRLF")]
    InvalidLineBreak,

    #[error("invalid control character in input")]
    InvalidControlCharacter,

    #[error("bare key contains '+' character")]
    PlusCharacter,

    #[error("trailing backslash in input")]
    TrailingBackslash,

    #[error("\\e is allowed in TOML v1.1.0 or later")]
    EscapeCharacter,

    #[error("\\xXX is allowed in TOML v1.1.0 or later")]
    HexEscapeSequence,

    #[error("unicode key is allowed in TOML v1.1.0 or later")]
    UnicodeKey,
}

pub fn try_from_bare_key(value: &str) -> Result<String, ParseError> {
    if value.chars().any(|c| matches!(c, '+')) {
        return Err(ParseError::PlusCharacter);
    }

    // TODO: In toml `v1.1.0`, bare key unicode support was not merged and will likely be deferred for discussion in `v1.2.0`.
    //       See: https://github.com/toml-lang/toml/issues/954#issuecomment-1932268939
    if
    // toml_version >= TomlVersion::V1_1_0_Preview ||
    value.chars().all(|c| {
        matches!(
            c,
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'
            // FIXME: This code can be removed if we can handle keys of floats correctly.
            | '.'
        )
    }) {
        Ok(value.to_string())
    } else {
        Err(ParseError::UnicodeKey)
    }
}

pub fn try_from_basic_string(value: &str, toml_version: TomlVersion) -> Result<String, ParseError> {
    parse_basic_string(&value[1..value.len() - 1], toml_version, false)
}

pub fn try_from_literal_string(value: &str) -> Result<String, ParseError> {
    parse_literal_string(&value[1..value.len() - 1], false)
}

pub fn try_from_multi_line_basic_string(
    value: &str,
    toml_version: TomlVersion,
) -> Result<String, ParseError> {
    parse_basic_string(
        &value[3..value.len() - 3]
            .chars()
            .skip_while(|c| matches!(c, '\n'))
            .collect::<String>(),
        toml_version,
        true,
    )
}

pub fn try_from_multi_line_literal_string(value: &str) -> Result<String, ParseError> {
    parse_literal_string(
        &value[3..value.len() - 3]
            .chars()
            .skip_while(|c| matches!(c, '\n'))
            .collect::<String>(),
        true,
    )
}

pub fn parse_basic_string(
    input: &str,
    toml_version: TomlVersion,
    is_multi_line: bool,
) -> Result<String, ParseError> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut unicode_buf = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next_c) = chars.peek() {
                    match next_c {
                        'b' => {
                            output.push('\u{0008}');
                            chars.next();
                        }
                        'e' => {
                            if toml_version >= TomlVersion::V1_1_0_Preview {
                                output.push('\u{001B}');
                                chars.next();
                            } else {
                                return Err(ParseError::EscapeCharacter);
                            }
                        }
                        't' => {
                            output.push('\t');
                            chars.next();
                        }
                        'n' => {
                            output.push('\n');
                            chars.next();
                        }
                        'f' => {
                            output.push('\u{000C}');
                            chars.next();
                        }
                        'r' => {
                            output.push('\r');
                            chars.next();
                        }
                        '"' => {
                            output.push('\"');
                            chars.next();
                        }
                        '\\' => {
                            output.push('\\');
                            chars.next();
                        }
                        'u' => {
                            chars.next(); // consume 'u'
                            unicode_buf.clear();
                            for _ in 0..4 {
                                if let Some(hex_digit) = chars.next() {
                                    unicode_buf.push(hex_digit);
                                } else {
                                    return Err(ParseError::InvalidUnicodeEscapeSequence);
                                }
                            }
                            if let Ok(code_point) = u32::from_str_radix(&unicode_buf, 16) {
                                if let Some(unicode_char) = std::char::from_u32(code_point) {
                                    output.push(unicode_char);
                                } else {
                                    return Err(ParseError::InvalidUnicodeCodePoint);
                                }
                            } else {
                                return Err(ParseError::InvalidUnicodeEscapeSequence);
                            }
                        }
                        'U' => {
                            chars.next(); // consume 'U'
                            unicode_buf.clear();
                            for _ in 0..8 {
                                if let Some(hex_digit) = chars.next() {
                                    unicode_buf.push(hex_digit);
                                } else {
                                    return Err(ParseError::InvalidUnicodeEscapeSequence);
                                }
                            }
                            if let Ok(code_point) = u32::from_str_radix(&unicode_buf, 16) {
                                if let Some(unicode_char) = std::char::from_u32(code_point) {
                                    output.push(unicode_char);
                                } else {
                                    return Err(ParseError::InvalidUnicodeCodePoint);
                                }
                            } else {
                                return Err(ParseError::InvalidUnicodeEscapeSequence);
                            }
                        }
                        'x' => {
                            if toml_version >= TomlVersion::V1_1_0_Preview {
                                chars.next(); // consume 'x'

                                unicode_buf.clear();
                                for _ in 0..2 {
                                    if let Some(hex_digit) = chars.next() {
                                        unicode_buf.push(hex_digit);
                                    } else {
                                        return Err(ParseError::InvalidUnicodeEscapeSequence);
                                    }
                                }

                                if let Ok(code_point) = u32::from_str_radix(&unicode_buf, 16) {
                                    if let Some(unicode_char) = std::char::from_u32(code_point) {
                                        output.push(unicode_char);
                                    } else {
                                        return Err(ParseError::InvalidUnicodeCodePoint);
                                    }
                                } else {
                                    return Err(ParseError::InvalidUnicodeEscapeSequence);
                                }
                            } else {
                                return Err(ParseError::HexEscapeSequence);
                            }
                        }
                        c if c.is_whitespace() => {
                            // Skip newline characters
                            let mut has_whitespace = c == '\n';

                            chars.next();
                            while let Some(&c) = chars.peek() {
                                if c.is_whitespace() {
                                    has_whitespace = has_whitespace || c == '\n';
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            if !has_whitespace {
                                return Err(ParseError::InvalidWhitespaceEscapeSequence);
                            }
                        }
                        _ => {
                            return Err(ParseError::InvalidEscapeSequence);
                        }
                    }
                } else {
                    return Err(ParseError::TrailingBackslash);
                }
            }
            '\r' | '\n' if is_multi_line => {
                output.push(c);
                if c == '\r' {
                    if let Some(&'\n') = chars.peek() {
                        output.push(chars.next().unwrap());
                    } else {
                        return Err(ParseError::InvalidLineBreak);
                    }
                }
            }
            '\u{0000}'..='\u{0008}' | '\u{000A}'..='\u{001F}' | '\u{007F}' => {
                return Err(ParseError::InvalidControlCharacter);
            }
            _ => {
                output.push(c);
            }
        }
    }
    Ok(output)
}

pub fn parse_literal_string(input: &str, is_multi_line: bool) -> Result<String, ParseError> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\r' | '\n' if is_multi_line => {
                output.push(c);
                if c == '\r' {
                    if let Some(&'\n') = chars.peek() {
                        output.push(chars.next().unwrap());
                    } else {
                        return Err(ParseError::InvalidLineBreak);
                    }
                }
            }
            '\u{0000}'..='\u{0008}' | '\u{000A}'..='\u{001F}' | '\u{007F}' => {
                return Err(ParseError::InvalidControlCharacter);
            }
            _ => {
                output.push(c);
            }
        }
    }
    Ok(output)
}

pub fn to_basic_string(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for c in value.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\x00' => result.push_str("\\0"),
            '\x07' => result.push_str("\\a"),
            '\x08' => result.push_str("\\b"),
            '\x09' => result.push_str("\\t"),
            '\x0a' => result.push_str("\\n"),
            '\x0b' => result.push_str("\\v"),
            '\x0c' => result.push_str("\\f"),
            '\x0d' => result.push_str("\\r"),
            '\x1b' => result.push_str("\\e"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

pub fn to_literal_string(value: &str) -> String {
    format!("'{value}'")
}

pub fn to_multi_line_basic_string(value: &str) -> String {
    format!("\"\"\"\n{value}\"\"\"")
}

pub fn to_multi_line_literal_string(value: &str) -> String {
    format!("'''\n{value}'''")
}
