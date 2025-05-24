mod cursor;
mod error;
mod lexed;
mod token;

use cursor::Cursor;
use error::ErrorKind::*;
pub use error::{Error, ErrorKind};
pub use lexed::Lexed;
pub use token::Token;
use tombi_syntax::{SyntaxKind, T};

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
    REGEX_INTEGER_BIN = r"^0b[0|1](:?_?[0|1])*$";
    REGEX_INTEGER_OCT = r"^0o[0-7](:?_?[0-7])*$";
    REGEX_INTEGER_HEX = r"^0x[0-9A-Fa-f](:?_?[0-9A-Fa-f])*$";
    REGEX_INTEGER_DEC = r"^(:?[1-9](:?_?[0-9])*|0)$";
    REGEX_FLOAT = r"^[0-9_]+(:?(:?\.[0-9_]+)?[eE][+-]?[0-9_]+|\.[0-9_]+)$";
    REGEX_IS_DATE_TIME = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}";
    REGEX_OFFSET_DATE_TIME =
        r"^[0-9]{4}-[0-9]{2}-[0-9]{2}[Tt ][0-9]{2}:[0-9]{2}(?::[0-9]{2})?(?:[\.,][0-9]+)?(?:[Zz]|[+-][0-9]{2}:[0-9]{2})$";
    REGEX_LOCAL_DATE_TIME = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}[Tt ][0-9]{2}:[0-9]{2}(?::[0-9]{2})?(?:[\.,][0-9]+)?$";
    REGEX_LOCAL_DATE = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$";
    REGEX_LOCAL_TIME = r"^[0-9]{2}:[0-9]{2}(?::[0-9]{2})?(?:[\.,][0-9]+)?$";
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

pub fn lex_document_header_comments(source: &str) -> Lexed {
    let mut lexed = Lexed::default();
    let mut was_joint = false;
    let mut last_offset = tombi_text::Offset::default();
    let mut last_position = tombi_text::Position::default();

    for result in tokenize(source) {
        match result {
            Ok(token) => match token.kind() {
                SyntaxKind::COMMENT => {
                    if was_joint {
                        lexed.set_joint();
                    }
                    was_joint = true;
                    let (last_span, last_range) = lexed.push_result_token(Ok(token));
                    last_offset = last_span.end();
                    last_position = last_range.end;
                }
                SyntaxKind::LINE_BREAK | SyntaxKind::WHITESPACE => {
                    let (last_span, last_range) = lexed.push_result_token(Ok(token));
                    last_offset = last_span.end();
                    last_position = last_range.end;
                }
                _ => break,
            },
            Err(_) => break,
        }
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
    std::iter::from_fn(move || match cursor.advance_token() {
        Ok(token) => match token.kind() {
            kind if kind != SyntaxKind::EOF => Some(Ok(token)),
            _ => None,
        },
        Err(error) => Some(Err(error)),
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
            '#' => self.line_comment(),
            '"' => {
                if self.matches(r#"""""#) {
                    self.multi_line_basic_string()
                } else {
                    self.basic_string()
                }
            }
            // number
            '0'..='9' => {
                if self.is_datetime() {
                    self.datetime()
                } else if self.is_time() {
                    self.time()
                } else if self.is_number() {
                    self.number()
                } else {
                    self.key()
                }
            }
            '\'' => {
                if self.matches("'''") {
                    self.multi_line_literal_string()
                } else {
                    self.literal_string()
                }
            }
            '+' => {
                self.bump();
                if self.is_keyword("inf") || self.is_keyword("nan") {
                    self.eat_n(2);
                    Ok(Token::new(SyntaxKind::FLOAT, self.pop_span_range()))
                } else if self.current().is_ascii_digit() {
                    self.number()
                } else {
                    self.eat_while(|c| !is_token_separator_with_dot(c));
                    Err(crate::Error::new(InvalidToken, self.pop_span_range()))
                }
            }
            '-' => {
                if ["inf", "nan"].contains(&self.peeks(3).as_str())
                    && is_token_separator_with_dot(self.peek(4))
                {
                    self.eat_n(3);
                    Ok(Token::new(SyntaxKind::FLOAT, self.pop_span_range()))
                } else if self.peek(1).is_ascii_digit() {
                    self.bump();
                    self.number()
                } else {
                    self.key()
                }
            }
            '{' => Ok(Token::new(T!('{'), self.pop_span_range())),
            '}' => Ok(Token::new(T!('}'), self.pop_span_range())),
            '[' => Ok(Token::new(T!('['), self.pop_span_range())),
            ']' => Ok(Token::new(T!(']'), self.pop_span_range())),
            ',' => Ok(Token::new(T!(,), self.pop_span_range())),
            '.' => Ok(Token::new(T!(.), self.pop_span_range())),
            '=' => Ok(Token::new(T!(=), self.pop_span_range())),
            'A'..='Z' | 'a'..='z' | '_' | '\u{A0}'..='\u{10FFFF}' => {
                if self.is_keyword("inf") || self.is_keyword("nan") {
                    self.eat_n(2);
                    Ok(Token::new(SyntaxKind::FLOAT, self.pop_span_range()))
                } else if self.is_keyword("true") {
                    self.eat_n(3);
                    Ok(Token::new(SyntaxKind::BOOLEAN, self.pop_span_range()))
                } else if self.is_keyword("false") {
                    self.eat_n(4);
                    Ok(Token::new(SyntaxKind::BOOLEAN, self.pop_span_range()))
                } else {
                    self.key()
                }
            }
            _ => {
                self.eat_while(|c| !is_token_separator_with_dot(c));
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

    fn line_comment(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current() == '#');

        self.eat_while(|c| !matches!(c, '\n' | '\r'));
        Ok(Token::new(SyntaxKind::COMMENT, self.pop_span_range()))
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
        }

        Ok(Token::new(SyntaxKind::LINE_BREAK, self.pop_span_range()))
    }

    #[inline]
    fn is_keyword(&self, keyword: &str) -> bool {
        self.matches(keyword) && is_token_separator_with_dot(self.peek(keyword.len()))
    }

    fn is_datetime(&self) -> bool {
        assert!(self.current().is_ascii_digit());
        assert!("2000-01-01".len() == 10);
        REGEX_IS_DATE_TIME.is_match(&self.peeks_with_current(10))
    }

    fn datetime(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current().is_ascii_digit());

        let mut pass_local_date_time = false;
        let mut pass_local_date = false;
        let mut index = 0;

        let line = self.peek_with_current_while(|c| {
            index += 1;
            (index == 10 && c == ' ') || !is_token_separator(c)
        });
        if let Some(m) = REGEX_OFFSET_DATE_TIME.find(&line) {
            assert!(m.start() == 0);

            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }

            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(
                    SyntaxKind::OFFSET_DATE_TIME,
                    self.pop_span_range(),
                ));
            }
        } else if let Some(m) = REGEX_LOCAL_DATE_TIME.find(&line) {
            assert!(m.start() == 0);

            pass_local_date_time = true;

            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }
            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(
                    SyntaxKind::LOCAL_DATE_TIME,
                    self.pop_span_range(),
                ));
            }
        } else if let Some(m) = REGEX_LOCAL_DATE.find(&line[..10]) {
            assert!(m.start() == 0);

            pass_local_date = true;

            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }
            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(SyntaxKind::LOCAL_DATE, self.pop_span_range()));
            }
        }

        self.eat_while(|c| !is_token_separator_with_dot(c));
        if pass_local_date_time {
            Err(crate::Error::new(
                InvalidOffsetDateTime,
                self.pop_span_range(),
            ))
        } else if pass_local_date {
            Err(crate::Error::new(
                InvalidLocalDateTime,
                self.pop_span_range(),
            ))
        } else {
            Err(crate::Error::new(InvalidLocalDate, self.pop_span_range()))
        }
    }

    fn is_time(&self) -> bool {
        assert!(self.current().is_ascii_digit());
        assert!("00:00".len() == 5);
        REGEX_LOCAL_TIME.is_match(&self.peeks_with_current(5))
    }

    fn time(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current().is_ascii_digit());

        let line = self.peek_with_current_while(|c| !is_token_separator(c));
        if let Some(m) = REGEX_LOCAL_TIME.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }

            Ok(Token::new(SyntaxKind::LOCAL_TIME, self.pop_span_range()))
        } else {
            self.eat_while(|c| !is_line_break(c) && !is_whitespace(c));

            Err(crate::Error::new(InvalidLocalTime, self.pop_span_range()))
        }
    }

    fn is_number(&self) -> bool {
        let line = self.peek_with_current_while(|c| !is_token_separator(c));

        REGEX_FLOAT.is_match(&line)
            || REGEX_INTEGER_BIN.is_match(&line)
            || REGEX_INTEGER_OCT.is_match(&line)
            || REGEX_INTEGER_HEX.is_match(&line)
            || REGEX_INTEGER_DEC.is_match(&line)
    }

    fn number(&mut self) -> Result<Token, crate::Error> {
        let line = self.peek_with_current_while(|c| !is_token_separator(c));
        if let Some(m) = REGEX_FLOAT.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }

            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(SyntaxKind::FLOAT, self.pop_span_range()));
            }
        } else if let Some(m) = REGEX_INTEGER_BIN.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }

            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(SyntaxKind::INTEGER_BIN, self.pop_span_range()));
            }
        } else if let Some(m) = REGEX_INTEGER_OCT.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }

            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(SyntaxKind::INTEGER_OCT, self.pop_span_range()));
            }
        } else if let Some(m) = REGEX_INTEGER_HEX.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }
            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(SyntaxKind::INTEGER_HEX, self.pop_span_range()));
            }
        } else if let Some(m) = REGEX_INTEGER_DEC.find(&line) {
            assert!(m.start() == 0);
            if m.end() > 1 {
                self.eat_n(m.end() - 1);
            }

            if is_token_separator_with_dot(self.peek(1)) {
                return Ok(Token::new(SyntaxKind::INTEGER_DEC, self.pop_span_range()));
            }
        }

        self.eat_while(|c| !is_token_separator_with_dot(c));

        Err(crate::Error::new(InvalidNumber, self.pop_span_range()))
    }

    fn basic_string(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current() == '"');

        while let Some(c) = self.bump() {
            match c {
                _ if c == '"' => {
                    return Ok(Token::new(SyntaxKind::BASIC_STRING, self.pop_span_range()))
                }
                '\\' if matches!(self.peek(1), '"' | '\\') => {
                    self.bump();
                }
                _ if is_line_break(self.peek(1)) => break,
                _ => (),
            }
        }

        Err(crate::Error::new(InvalidBasicString, self.pop_span_range()))
    }

    fn multi_line_basic_string(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current() == '"' && self.peek(1) == '"');

        self.eat_n(2);

        let mut was_quote = false;
        while let Some(c) = self.bump() {
            match c {
                _ if !was_quote
                    && self.current() == '"'
                    && self.peek(1) == '"'
                    && self.peek(2) == '"' =>
                {
                    let last_quotes = self.peek_while(|c| c == '"');

                    self.eat_while(|c| c == '"');

                    if last_quotes.len() > 4 {
                        break;
                    }

                    return Ok(Token::new(
                        SyntaxKind::MULTI_LINE_BASIC_STRING,
                        self.pop_span_range(),
                    ));
                }
                '\\' => {
                    was_quote = true;
                }
                _ => {
                    was_quote = false;
                }
            }
        }

        Err(crate::Error::new(
            InvalidMultilineBasicString,
            self.pop_span_range(),
        ))
    }

    fn literal_string(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current() == '\'');

        while let Some(c) = self.bump() {
            match c {
                '\'' => {
                    return Ok(Token::new(
                        SyntaxKind::LITERAL_STRING,
                        self.pop_span_range(),
                    ))
                }
                _ if is_line_break(self.peek(1)) => break,
                _ => {}
            }
        }

        Err(crate::Error::new(
            InvalidLiteralString,
            self.pop_span_range(),
        ))
    }

    fn multi_line_literal_string(&mut self) -> Result<Token, crate::Error> {
        assert!(self.current() == '\'' && self.peek(1) == '\'');

        self.eat_n(2);

        while let Some(c) = self.bump() {
            match c {
                _ if self.current() == '\'' && self.peek(1) == '\'' && self.peek(2) == '\'' => {
                    let last_quotes = self.peek_while(|c| c == '\'');
                    self.eat_while(|c| c == '\'');

                    if last_quotes.len() > 4 {
                        break;
                    }

                    return Ok(Token::new(
                        SyntaxKind::MULTI_LINE_LITERAL_STRING,
                        self.pop_span_range(),
                    ));
                }
                _ => {}
            }
        }

        Err(crate::Error::new(
            InvalidMultilineLiteralString,
            self.pop_span_range(),
        ))
    }

    fn key(&mut self) -> Result<Token, crate::Error> {
        self.eat_while(|c| {
            matches!(
                c,
                // ASCII characters - a-z A-Z 0-9 - _
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' |

                // ==============================================
                // TOML v1.1.0 supports Unicode for bare keys
                // ==============================================
                //
                // TODO: The exact specification is not yet known, but it is provisionally implemented to pass toml-test.
                //       When the use of v1.1.0 is officially confirmed, it will be implemented.
                //
                // See discussion: https://github.com/toml-lang/toml/discussions/941
                // See specification: https://github.com/toml-lang/toml/blob/78fcf9dd7eab7acfbaf147c684b649477e7bdd9c/toml.abnf#L55-L64
                //

                // Superscript digits, fractions
                '\u{B2}' | '\u{B3}' | '\u{B9}' | '\u{BC}'..='\u{BE}' |

                // Non-symbol chars in Latin block
                '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{F6}' | '\u{F8}'..='\u{37D}' |

                // Exclude GREEK QUESTION MARK (which is basically a semicolon)
                '\u{37F}'..='\u{1FFF}' |

                // From General Punctuation Block, include the two tie symbols and ZWNJ, ZWJ
                '\u{200C}'..='\u{200D}' | '\u{203F}'..='\u{2040}' |

                // Include super-/subscripts, letterlike/numberlike forms, enclosed alphanumerics
                '\u{2070}'..='\u{218F}' | '\u{2460}'..='\u{24FF}' |

                // Skip arrows, math, box drawing etc
                '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' |

                // Skip surrogate block, Private Use area, intended for process-internal use
                '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' |

                // All chars outside BMP range, excluding Private Use planes
                '\u{10000}'..='\u{EFFFF}'
            )
        });
        if is_token_separator_with_dot(self.peek(1)) {
            Ok(Token::new(SyntaxKind::BARE_KEY, self.pop_span_range()))
        } else {
            self.eat_while(|c| !is_token_separator_with_dot(c));
            Err(crate::Error::new(InvalidKey, self.pop_span_range()))
        }
    }
}

#[inline]
fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

#[inline]
fn is_line_break(c: char) -> bool {
    matches!(c, '\r' | '\n')
}

#[inline]
fn is_token_separator(c: char) -> bool {
    matches!(
        c,
        '{' | '}' | '[' | ']' | ',' | '=' | ' ' | '\t' | '\r' | '\n' | '#' | '\0'
    )
}

#[inline]
fn is_token_separator_with_dot(c: char) -> bool {
    matches!(
        c,
        '{' | '}' | '[' | ']' | ',' | '.' | '=' | ' ' | '\t' | '\r' | '\n' | '#' | '\0'
    )
}
