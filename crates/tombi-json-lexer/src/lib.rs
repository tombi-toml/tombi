mod cursor;
mod error;
mod lexed;
mod scanner;
mod token;

use cursor::Cursor;
use error::ErrorKind::*;
pub use error::{Error, ErrorKind};
pub use lexed::Lexed;
pub use token::Token;
use tombi_json_syntax::{SyntaxKind, T};

pub fn lex(source: &str) -> Lexed {
    let mut lexed = Lexed::default();
    let mut last_offset = tombi_text::Offset::default();

    let mut cursor = Cursor::new(source);
    for result in tokenize_with(&mut cursor) {
        last_offset = lexed.push_result_token(result).end;
    }

    lexed.line_starts = std::mem::take(&mut cursor.line_starts);
    lexed.tokens.push(crate::Token::new(
        SyntaxKind::EOF,
        tombi_text::Span::new(last_offset, tombi_text::Offset::new(source.len() as u32)),
    ));

    lexed
}

/// A lexer that yields the non-trivia tokens of a source one at a time.
///
/// A parser that skips trivia can use it instead of [`lex`], so that the tokens are not
/// collected into a vector first.
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }

    /// Returns the next non-trivia token.
    ///
    /// An invalid token is returned as [`SyntaxKind::INVALID_TOKEN`], as in [`Lexed::tokens`].
    /// After the end of the source, it keeps returning an [`SyntaxKind::EOF`] token.
    pub fn next_token(&mut self) -> Token {
        self.cursor.skip_trivia();
        match self.cursor.advance_token() {
            Ok(token) if token.is_eof() => {
                let end = tombi_text::Offset::new(self.cursor.source_len() as u32);
                Token::new(SyntaxKind::EOF, tombi_text::Span::new(end, end))
            }
            Ok(token) => token,
            Err(error) => Token::new(SyntaxKind::INVALID_TOKEN, error.span()),
        }
    }

    /// Returns the start offset of each line scanned so far.
    ///
    /// They cover the whole source once [`Lexer::next_token`] has returned [`SyntaxKind::EOF`].
    pub fn into_line_starts(self) -> Vec<tombi_text::Offset> {
        self.cursor.line_starts
    }
}

pub fn tokenize(source: &str) -> impl Iterator<Item = Result<Token, crate::Error>> + '_ {
    let mut cursor = Cursor::new(source);
    std::iter::from_fn(move || next_token(&mut cursor))
}

fn tokenize_with<'a, 'b>(
    cursor: &'b mut Cursor<'a>,
) -> impl Iterator<Item = Result<Token, crate::Error>> + 'b {
    std::iter::from_fn(move || next_token(cursor))
}

#[inline]
fn next_token(cursor: &mut Cursor<'_>) -> Option<Result<Token, crate::Error>> {
    match cursor.advance_token() {
        Ok(token) => match token.kind() {
            kind if kind != SyntaxKind::EOF => Some(Ok(token)),
            _ => None,
        },
        Err(error) => Some(Err(error)),
    }
}

impl Cursor<'_> {
    /// Parses a token from the input string.
    pub fn advance_token(&mut self) -> Result<Token, crate::Error> {
        let Some(byte) = self.first() else {
            return Ok(Token::eof());
        };
        match byte {
            b' ' | b'\t' => self.whitespace(),
            b'\n' | b'\r' => self.line_break(),
            // JSON object brackets
            b'{' => self.punctuation(T!['{']),
            b'}' => self.punctuation(T!['}']),
            // JSON array brackets
            b'[' => self.punctuation(T!['[']),
            b']' => self.punctuation(T![']']),
            // JSON value separators
            b',' => self.punctuation(T![,]),
            b':' => self.punctuation(T![:]),
            b'"' => self.string(),
            // JSON number
            b'0'..=b'9' | b'-' => self.number(),
            // JSON keywords
            b't' => self.keyword(b"true", SyntaxKind::BOOLEAN, InvalidTrue),
            b'f' => self.keyword(b"false", SyntaxKind::BOOLEAN, InvalidFalse),
            b'n' => self.keyword(b"null", SyntaxKind::NULL, InvalidNull),
            _ => self.invalid_token(InvalidToken),
        }
    }

    #[inline]
    fn punctuation(&mut self, kind: SyntaxKind) -> Result<Token, crate::Error> {
        self.eat_bytes(1);
        Ok(Token::new(kind, self.pop_span()))
    }

    fn whitespace(&mut self) -> Result<Token, crate::Error> {
        self.eat_while(is_whitespace);
        Ok(Token::new(SyntaxKind::WHITESPACE, self.pop_span()))
    }

    fn line_break(&mut self) -> Result<Token, crate::Error> {
        if self.first() == Some(b'\r') {
            self.eat_bytes(1);
            if self.first() != Some(b'\n') {
                return Ok(Token::new(SyntaxKind::WHITESPACE, self.pop_span()));
            }
        }
        self.eat_line_break();
        Ok(Token::new(SyntaxKind::LINE_BREAK, self.pop_span()))
    }

    fn keyword(
        &mut self,
        keyword: &[u8],
        kind: SyntaxKind,
        error_kind: ErrorKind,
    ) -> Result<Token, crate::Error> {
        if self.remaining().starts_with(keyword) {
            self.eat_bytes(keyword.len());
            Ok(Token::new(kind, self.pop_span()))
        } else {
            self.invalid_token(error_kind)
        }
    }

    /// Consumes the current char and the following chars up to a token separator.
    fn invalid_token(&mut self, error_kind: ErrorKind) -> Result<Token, crate::Error> {
        // A non-ASCII char continues with non-separator bytes, so it is consumed as a whole.
        self.eat_bytes(1);
        self.eat_while(|byte| !is_token_separator(byte));
        Err(crate::Error::new(error_kind, self.pop_span()))
    }

    fn number(&mut self) -> Result<Token, crate::Error> {
        if let Some(len) = json_number_len(self.remaining()) {
            self.eat_bytes(len);
            return Ok(Token::new(SyntaxKind::NUMBER, self.pop_span()));
        }

        self.invalid_token(InvalidNumber)
    }

    fn string(&mut self) -> Result<Token, crate::Error> {
        debug_assert_eq!(self.first(), Some(b'"'));
        self.eat_bytes(1);

        let mut is_valid = true;
        let mut contains_escape = false;
        loop {
            self.eat_bytes(scanner::ordinary_string_prefix(self.remaining()));
            match self.first() {
                Some(b'"') => {
                    self.eat_bytes(1);
                    if !is_valid {
                        return Err(crate::Error::new(InvalidString, self.pop_span()));
                    }
                    return Ok(Token::new_string(contains_escape, self.pop_span()));
                }
                Some(b'\\') => {
                    contains_escape = true;
                    self.eat_bytes(1);
                    is_valid &= self.escape();
                }
                // An unescaped line break is invalid, but keeps the line starts complete.
                Some(b'\n') => {
                    is_valid = false;
                    self.eat_line_break();
                }
                Some(byte) => {
                    debug_assert!(byte < 0x20, "the scanner stops only at special bytes");
                    is_valid = false;
                    self.eat_bytes(1);
                }
                None => return Err(crate::Error::new(InvalidString, self.pop_span())),
            }
        }
    }

    /// Consumes an escape sequence after `\\`, and returns whether it is valid.
    fn escape(&mut self) -> bool {
        match self.first() {
            Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                self.eat_bytes(1);
                true
            }
            Some(b'u') => {
                self.eat_bytes(1);
                let len = self
                    .remaining()
                    .iter()
                    .take(4)
                    .take_while(|byte| byte.is_ascii_hexdigit())
                    .count();
                self.eat_bytes(len);
                len == 4
            }
            // Leave the byte to the string loop, so that a quote still closes the string
            // and a line break is still recorded.
            _ => false,
        }
    }
}

/// Returns the length of the JSON number at the start of `bytes`.
#[inline]
fn json_number_len(bytes: &[u8]) -> Option<usize> {
    let mut index = usize::from(bytes.first() == Some(&b'-'));

    let integer_first = *bytes.get(index)?;
    if !integer_first.is_ascii_digit() {
        return None;
    }
    index += 1;

    let mut integer_len = 1;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
        integer_len += 1;
    }

    let mut has_fraction_or_exponent = false;
    if bytes.get(index) == Some(&b'.') {
        has_fraction_or_exponent = true;
        index += 1;
        if !bytes.get(index).is_some_and(u8::is_ascii_digit) {
            return None;
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        has_fraction_or_exponent = true;
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if !bytes.get(index).is_some_and(u8::is_ascii_digit) {
            return None;
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
    }

    // Keep compatibility with the previous regexes: leading zeroes are
    // rejected for integers but were accepted for fractions and exponents.
    if integer_first == b'0' && integer_len > 1 && !has_fraction_or_exponent {
        return None;
    }

    match bytes.get(index) {
        None => Some(index),
        Some(&next) if is_token_separator(next) => Some(index),
        Some(_) => None,
    }
}

#[inline]
fn is_whitespace(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

#[inline]
fn is_token_separator(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'}' | b'[' | b']' | b',' | b':' | b'"' | b'\0'
    )
}
