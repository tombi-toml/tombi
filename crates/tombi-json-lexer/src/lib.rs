mod cursor;
mod error;
mod lexed;
mod token;

use cursor::Cursor;
use error::ErrorKind::*;
pub use error::{Error, ErrorKind};
pub use lexed::Lexed;
pub use token::Token;
use tombi_json_syntax::{SyntaxKind, T};

macro_rules! regex {
    ($($var:ident = $re:expr);+;) => {
        $(
            static $var: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
                regex::Regex::new($re).unwrap()
            });
        )+
    };
}

regex!(
    REGEX_INTEGER_DEC = r"^-?(:?[1-9](:?[0-9])*|0)$";
    REGEX_FLOAT = r"^-?[0-9]+(:?(:?\.[0-9]+)?[eE][+-]?[0-9]+|\.[0-9]+)$";
);

#[tracing::instrument(level = "debug", skip_all)]
pub fn lex(source: &str) -> Lexed {
    let mut lexed = Lexed::default();
    let mut was_joint = false;
    let mut last_offset = tombi_text::Offset::default();
    let mut last_position = tombi_text::Position::default();

    for result in tokenize(source) {
        match result {
            Ok(token) if token.kind().is_trivia() => was_joint = false,
            _ => {
                if was_joint {
                    lexed.set_joint();
                }
                was_joint = true;
            }
        }
        let (last_span, last_range) = lexed.push_result_token(result);
        last_offset = last_span.end();
        last_position = last_range.end;
    }

    lexed.tokens.push(crate::Token::new(
        SyntaxKind::EOF,
        (
            tombi_text::Span::new(last_offset, tombi_text::Offset::new(source.len() as u32)),
            tombi_text::Range::new(
                last_position,
                last_position + tombi_text::RelativePosition::of(&source[last_offset.into()..]),
            ),
        ),
    ));

    lexed
}

pub fn tokenize(source: &str) -> impl Iterator<Item = Result<Token, crate::Error>> + '_ {
    let mut cursor = Cursor::new(source);

    std::iter::from_fn(move || {
        let token = cursor.advance_token();

        match token {
            Ok(token) => match token.kind() {
                kind if kind != SyntaxKind::EOF => Some(Ok(token)),
                _ => None,
            },
            Err(error) => Some(Err(error)),
        }
    })
}

impl Cursor<'_> {
    /// Parses a token from the input string.
    pub fn advance_token(&mut self) -> Result<Token, crate::Error> {
        if self.bump().is_none() {
            return Ok(Token::eof());
        }
        match self.current() {
            _ if self.is_whitespace() => self.whitespace(),
            _ if self.is_line_break() => self.line_break(),
            // JSON object brackets
            '{' => Ok(Token::new(T!['{'], self.pop_span_range())),
            '}' => Ok(Token::new(T!['}'], self.pop_span_range())),
            // JSON array brackets
            '[' => Ok(Token::new(T!['['], self.pop_span_range())),
            ']' => Ok(Token::new(T![']'], self.pop_span_range())),
            // JSON value separators
            ',' => Ok(Token::new(T![,], self.pop_span_range())),
            ':' => Ok(Token::new(T![:], self.pop_span_range())),
            '"' => self.string(),
            // JSON number
            '0'..='9' | '-' => self.number(),
            // JSON keywords
            't' => {
                if self.matches("true") {
                    self.eat_n(3);
                    Ok(Token::new(SyntaxKind::BOOLEAN, self.pop_span_range()))
                } else {
                    self.bump();
                    self.eat_while(|c| !is_token_separator(c));
                    Err(crate::Error::new(InvalidTrue, self.pop_span_range()))
                }
            }
            'f' => {
                if self.matches("false") {
                    self.eat_n(4);
                    Ok(Token::new(SyntaxKind::BOOLEAN, self.pop_span_range()))
                } else {
                    self.bump();
                    self.eat_while(|c| !is_token_separator(c));
                    Err(crate::Error::new(InvalidFalse, self.pop_span_range()))
                }
            }
            'n' => {
                if self.matches("null") {
                    self.eat_n(3);
                    Ok(Token::new(SyntaxKind::NULL, self.pop_span_range()))
                } else {
                    self.bump();
                    self.eat_while(|c| !is_token_separator(c));
                    Err(crate::Error::new(InvalidNull, self.pop_span_range()))
                }
            }
            _ => {
                self.bump();
                self.eat_while(|c| !is_token_separator(c));
                Err(crate::Error::new(InvalidToken, self.pop_span_range()))
            }
        }
    }

    fn is_whitespace(&self) -> bool {
        is_whitespace(self.current())
    }

    fn whitespace(&mut self) -> Result<Token, crate::Error> {
        self.eat_while(is_whitespace);
        Ok(Token::new(SyntaxKind::WHITESPACE, self.pop_span_range()))
    }

    fn is_line_break(&self) -> bool {
        is_line_break(self.current())
    }

    fn line_break(&mut self) -> Result<Token, crate::Error> {
        let c = self.current();
        assert!(matches!(c, '\r' | '\n'));
        if c == '\r' {
            if self.peek(1) == '\n' {
                self.eat_n(1);
            } else {
                return Err(crate::Error::new(InvalidLineBreak, self.pop_span_range()));
            }
        } else {
            while self.peek(1) == '\r' {
                self.eat_n(1);
            }
        }

        Ok(Token::new(SyntaxKind::LINE_BREAK, self.pop_span_range()))
    }

    fn number(&mut self) -> Result<Token, crate::Error> {
        let line = self.peek_with_current_while(|c| !is_token_separator(c));

        if let Some(m) = REGEX_FLOAT.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }
            return Ok(Token::new(SyntaxKind::NUMBER, self.pop_span_range()));
        } else if let Some(m) = REGEX_INTEGER_DEC.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }
            return Ok(Token::new(SyntaxKind::NUMBER, self.pop_span_range()));
        }

        self.eat_while(|c| !is_token_separator(c));

        Err(crate::Error::new(InvalidNumber, self.pop_span_range()))
    }

    fn string(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current() == '"');

        let mut first_error: Option<ErrorKind> = None;
        while let Some(c) = self.bump() {
            match c {
                _ if c == '"' => {
                    if let Some(error_kind) = first_error {
                        return Err(crate::Error::new(error_kind, self.pop_span_range()));
                    }

                    return Ok(Token::new(SyntaxKind::STRING, self.pop_span_range()));
                }
                '\u{0000}'..='\u{001F}' => {
                    if first_error.is_none() {
                        first_error = Some(InvalidString);
                    }
                }
                '\\' => match self.bump() {
                    Some(escape_char) => match escape_char {
                        '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {}
                        'u' => {
                            let mut valid_unicode = true;
                            for _i in 0..4 {
                                match self.bump() {
                                    Some(hex_char) if hex_char.is_ascii_hexdigit() => {}
                                    _ => {
                                        valid_unicode = false;
                                        break;
                                    }
                                }
                            }

                            if !valid_unicode && first_error.is_none() {
                                first_error = Some(InvalidString);
                            }
                        }
                        _ => {
                            if first_error.is_none() {
                                first_error = Some(InvalidString);
                            }
                        }
                    },
                    None => {
                        if first_error.is_none() {
                            first_error = Some(InvalidString);
                        }
                    }
                },
                _ => {}
            }
        }

        Err(crate::Error::new(InvalidString, self.pop_span_range()))
    }
}

#[inline]
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

#[inline]
fn is_line_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

#[inline]
fn is_token_separator(c: char) -> bool {
    is_whitespace(c)
        || is_line_break(c)
        || matches!(c, '{' | '}' | '[' | ']' | ',' | ':' | '"' | '\0')
}
