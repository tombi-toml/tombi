use crate::cursor::Cursor;
use crate::error::{Error, ErrorKind};
use crate::lexed::Lexed;
use crate::syntax_kind::SyntaxKind;
use crate::token::Token;

#[tracing::instrument(level = "debug", skip_all)]
pub fn lex(source: &str) -> Lexed {
    let mut lexed = Lexed::default();
    let mut last_offset = tombi_text::Offset::default();
    let mut last_position = tombi_text::Position::default();

    for result in tokenize(source) {
        let (last_span, last_range) = lexed.push_result_token(result);
        last_offset = last_span.end;
        last_position = last_range.end;
    }

    lexed.tokens.push(Token::new(
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

pub fn tokenize(source: &str) -> impl Iterator<Item = Result<Token, Error>> + '_ {
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
    pub fn advance_token(&mut self) -> Result<Token, Error> {
        if self.bump().is_none() {
            return Ok(Token::eof());
        }
        
        match self.current() {
            // Whitespace
            _ if self.is_whitespace() => self.whitespace(),
            
            // Single character tokens
            '[' => Ok(Token::new(SyntaxKind::BRACKET_START, self.pop_span_range())),
            ']' => Ok(Token::new(SyntaxKind::BRACKET_END, self.pop_span_range())),
            '(' => Ok(Token::new(SyntaxKind::PAREN_START, self.pop_span_range())),
            ')' => Ok(Token::new(SyntaxKind::PAREN_END, self.pop_span_range())),
            ',' => Ok(Token::new(SyntaxKind::COMMA, self.pop_span_range())),
            ';' => Ok(Token::new(SyntaxKind::SEMICOLON, self.pop_span_range())),
            '@' => Ok(Token::new(SyntaxKind::AT, self.pop_span_range())),
            
            // Operators
            '=' => {
                if self.peek(1) == '=' {
                    self.bump();
                    if self.peek(1) == '=' {
                        self.bump();
                        Ok(Token::new(SyntaxKind::EQ_EQ_EQ, self.pop_span_range()))
                    } else {
                        Ok(Token::new(SyntaxKind::EQ_EQ, self.pop_span_range()))
                    }
                } else {
                    self.bump();
                    Err(Error::new(ErrorKind::InvalidOperator, self.pop_span_range()))
                }
            }
            '!' => {
                if self.peek(1) == '=' {
                    self.bump();
                    Ok(Token::new(SyntaxKind::NOT_EQ, self.pop_span_range()))
                } else {
                    self.bump();
                    Err(Error::new(ErrorKind::InvalidOperator, self.pop_span_range()))
                }
            }
            '<' => {
                if self.peek(1) == '=' {
                    self.bump();
                    Ok(Token::new(SyntaxKind::LTE, self.pop_span_range()))
                } else {
                    Ok(Token::new(SyntaxKind::LT, self.pop_span_range()))
                }
            }
            '>' => {
                if self.peek(1) == '=' {
                    self.bump();
                    Ok(Token::new(SyntaxKind::GTE, self.pop_span_range()))
                } else {
                    Ok(Token::new(SyntaxKind::GT, self.pop_span_range()))
                }
            }
            '~' => {
                if self.peek(1) == '=' {
                    self.bump();
                    Ok(Token::new(SyntaxKind::TILDE_EQ, self.pop_span_range()))
                } else {
                    self.bump();
                    Err(Error::new(ErrorKind::InvalidOperator, self.pop_span_range()))
                }
            }
            
            // Strings
            '"' | '\'' => self.string(),
            
            // Identifiers, keywords, version strings
            _ if self.current().is_ascii_alphabetic() || self.current() == '_' => {
                self.identifier_or_keyword()
            }
            
            // Version strings
            _ if self.current().is_ascii_digit() => self.version_string(),
            
            // Unknown
            _ => {
                self.bump();
                self.eat_while(|c| !is_token_separator(c));
                Err(Error::new(ErrorKind::InvalidToken, self.pop_span_range()))
            }
        }
    }

    fn is_whitespace(&self) -> bool {
        is_whitespace(self.current())
    }

    fn whitespace(&mut self) -> Result<Token, Error> {
        self.eat_while(is_whitespace);
        Ok(Token::new(SyntaxKind::WHITESPACE, self.pop_span_range()))
    }

    fn string(&mut self) -> Result<Token, Error> {
        let quote_char = self.current();
        assert!(quote_char == '"' || quote_char == '\'');

        while let Some(c) = self.bump() {
            if c == quote_char {
                return Ok(Token::new(SyntaxKind::STRING, self.pop_span_range()));
            }
            if c == '\\' {
                // Skip escaped character
                self.bump();
            }
        }

        Err(Error::new(ErrorKind::UnterminatedString, self.pop_span_range()))
    }

    fn identifier_or_keyword(&mut self) -> Result<Token, Error> {
        let start = self.peek_with_current_while(|c| {
            c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
        });

        // Consume the identifier
        if start.len() > 1 {
            self.eat_n(start.len() - 1);
        }

        let kind = match start.as_str() {
            "and" => SyntaxKind::AND,
            "or" => SyntaxKind::OR,
            "in" => SyntaxKind::IN,
            "not" => SyntaxKind::NOT,
            "python_version" => SyntaxKind::PYTHON_VERSION,
            "python_full_version" => SyntaxKind::PYTHON_FULL_VERSION,
            "os_name" => SyntaxKind::OS_NAME,
            "sys_platform" => SyntaxKind::SYS_PLATFORM,
            "platform_release" => SyntaxKind::PLATFORM_RELEASE,
            "platform_system" => SyntaxKind::PLATFORM_SYSTEM,
            "platform_version" => SyntaxKind::PLATFORM_VERSION,
            "platform_machine" => SyntaxKind::PLATFORM_MACHINE,
            "platform_python_implementation" => SyntaxKind::PLATFORM_PYTHON_IMPLEMENTATION,
            "implementation_name" => SyntaxKind::IMPLEMENTATION_NAME,
            "implementation_version" => SyntaxKind::IMPLEMENTATION_VERSION,
            "extra" => SyntaxKind::EXTRA,
            _ => SyntaxKind::IDENTIFIER,
        };

        Ok(Token::new(kind, self.pop_span_range()))
    }

    fn version_string(&mut self) -> Result<Token, Error> {
        // Consume version-like characters (digits, dots, letters, etc.)
        self.eat_while(|c| {
            c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '*' || c == '+' || c == '!'
        });
        
        Ok(Token::new(SyntaxKind::VERSION_STRING, self.pop_span_range()))
    }
}

#[inline]
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

#[inline]
fn is_token_separator(c: char) -> bool {
    is_whitespace(c)
        || matches!(c, '[' | ']' | '(' | ')' | ',' | ';' | '@' | '"' | '\'' | '\0')
}