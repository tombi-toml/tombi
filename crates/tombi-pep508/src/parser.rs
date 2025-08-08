use crate::{lexer, Lexed, SyntaxKind, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Pep508Requirement {
    pub name: String,
    pub extras: Vec<String>,
    pub version_spec: Option<VersionSpec>,
    pub marker: Option<MarkerExpression>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionSpec {
    pub clauses: Vec<VersionClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionClause {
    pub operator: VersionOperator,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionOperator {
    Equal,            // ==
    NotEqual,         // !=
    LessThanEqual,    // <=
    GreaterThanEqual, // >=
    LessThan,         // <
    GreaterThan,      // >
    Compatible,       // ~=
    ArbitraryEqual,   // ===
}

impl From<SyntaxKind> for VersionOperator {
    fn from(kind: SyntaxKind) -> Self {
        match kind {
            SyntaxKind::EQ_EQ => VersionOperator::Equal,
            SyntaxKind::NOT_EQ => VersionOperator::NotEqual,
            SyntaxKind::LTE => VersionOperator::LessThanEqual,
            SyntaxKind::GTE => VersionOperator::GreaterThanEqual,
            SyntaxKind::LT => VersionOperator::LessThan,
            SyntaxKind::GT => VersionOperator::GreaterThan,
            SyntaxKind::TILDE_EQ => VersionOperator::Compatible,
            SyntaxKind::EQ_EQ_EQ => VersionOperator::ArbitraryEqual,
            _ => panic!("Invalid version operator kind: {:?}", kind),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkerExpression {
    pub expression: String,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

pub struct Parser<'a> {
    source: &'a str,
    lexed: Lexed,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let lexed = lexer::lex(source);
        Self {
            source,
            lexed,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Pep508Requirement, ParseError> {
        // Skip leading trivia
        self.skip_trivia();

        // Parse package name
        let name = self.parse_package_name()?;
        self.skip_trivia();

        // Check for extras
        let extras = if self.peek_kind() == Some(SyntaxKind::BRACKET_START) {
            self.parse_extras()?
        } else {
            Vec::new()
        };
        self.skip_trivia();

        // Check for URL dependency (@ url)
        let url = if self.peek_kind() == Some(SyntaxKind::AT) {
            self.advance(); // consume '@'
            self.skip_trivia();
            Some(self.parse_url()?)
        } else {
            None
        };

        // Check for version spec
        let version_spec = if url.is_none() && self.has_version_operator() {
            Some(self.parse_version_spec()?)
        } else {
            None
        };
        self.skip_trivia();

        // Check for marker
        let marker = if self.peek_kind() == Some(SyntaxKind::SEMICOLON) {
            self.advance(); // consume ';'
            self.skip_trivia();
            Some(self.parse_marker()?)
        } else {
            None
        };

        Ok(Pep508Requirement {
            name,
            extras,
            version_spec,
            marker,
            url,
        })
    }

    pub fn parse_partial(&mut self) -> PartialParseResult {
        self.skip_trivia();

        // Try to parse package name
        match self.parse_package_name() {
            Ok(name) => {
                self.skip_trivia();

                // Check what comes next
                if self.is_at_end() {
                    return PartialParseResult::PackageName {
                        name,
                        complete: true,
                    };
                }

                // Check for extras
                if self.peek_kind() == Some(SyntaxKind::BRACKET_START) {
                    self.advance(); // consume '['

                    // Try to parse extras list
                    let mut extras = Vec::new();
                    let mut complete = false;

                    loop {
                        self.skip_trivia();

                        if self.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                            self.advance();
                            complete = true;
                            break;
                        }

                        if self.is_at_end() {
                            break;
                        }

                        // Parse extra name
                        match self.parse_identifier() {
                            Ok(extra) => {
                                extras.push(extra);
                                self.skip_trivia();

                                if self.peek_kind() == Some(SyntaxKind::COMMA) {
                                    self.advance();
                                    self.skip_trivia();

                                    // Check if we're at the end after comma
                                    if self.is_at_end()
                                        || self.peek_kind() == Some(SyntaxKind::BRACKET_END)
                                    {
                                        // Incomplete after comma
                                        return PartialParseResult::ExtrasIncomplete {
                                            name,
                                            extras,
                                            after_comma: true,
                                        };
                                    }
                                } else if self.peek_kind() != Some(SyntaxKind::BRACKET_END)
                                    && !self.is_at_end()
                                {
                                    // Invalid character in extras
                                    break;
                                }
                            }
                            Err(_) => {
                                // Can't parse extra name
                                return PartialParseResult::ExtrasIncomplete {
                                    name,
                                    extras,
                                    after_comma: false,
                                };
                            }
                        }
                    }

                    if !complete {
                        return PartialParseResult::ExtrasIncomplete {
                            name,
                            extras,
                            after_comma: false,
                        };
                    }

                    // Continue parsing after extras
                    self.skip_trivia();
                }

                // Check for @ URL
                if self.peek_kind() == Some(SyntaxKind::AT) {
                    self.advance();
                    self.skip_trivia();

                    if self.is_at_end() {
                        return PartialParseResult::AfterAt { name };
                    }

                    // Try to parse URL
                    match self.parse_url() {
                        Ok(url) => {
                            return PartialParseResult::Complete(Pep508Requirement {
                                name,
                                extras: Vec::new(),
                                version_spec: None,
                                marker: None,
                                url: Some(url),
                            });
                        }
                        Err(_) => {
                            return PartialParseResult::UrlIncomplete { name };
                        }
                    }
                }

                // Check for version operator
                if self.has_version_operator() {
                    match self.parse_version_spec() {
                        Ok(version_spec) => {
                            self.skip_trivia();

                            // Check for marker
                            if self.peek_kind() == Some(SyntaxKind::SEMICOLON) {
                                self.advance();
                                self.skip_trivia();

                                if self.is_at_end() {
                                    return PartialParseResult::AfterSemicolon {
                                        name,
                                        version_spec: Some(version_spec),
                                    };
                                }

                                match self.parse_marker() {
                                    Ok(marker) => {
                                        return PartialParseResult::Complete(Pep508Requirement {
                                            name,
                                            extras: Vec::new(),
                                            version_spec: Some(version_spec),
                                            marker: Some(marker),
                                            url: None,
                                        });
                                    }
                                    Err(_) => {
                                        return PartialParseResult::MarkerIncomplete {
                                            name,
                                            version_spec: Some(version_spec),
                                        };
                                    }
                                }
                            }

                            return PartialParseResult::Complete(Pep508Requirement {
                                name,
                                extras: Vec::new(),
                                version_spec: Some(version_spec),
                                marker: None,
                                url: None,
                            });
                        }
                        Err(_) => {
                            return PartialParseResult::VersionIncomplete { name };
                        }
                    }
                }

                // Check for semicolon (marker)
                if self.peek_kind() == Some(SyntaxKind::SEMICOLON) {
                    self.advance();
                    self.skip_trivia();

                    if self.is_at_end() {
                        return PartialParseResult::AfterSemicolon {
                            name,
                            version_spec: None,
                        };
                    }

                    match self.parse_marker() {
                        Ok(marker) => {
                            return PartialParseResult::Complete(Pep508Requirement {
                                name,
                                extras: Vec::new(),
                                version_spec: None,
                                marker: Some(marker),
                                url: None,
                            });
                        }
                        Err(_) => {
                            return PartialParseResult::MarkerIncomplete {
                                name,
                                version_spec: None,
                            };
                        }
                    }
                }

                PartialParseResult::PackageName {
                    name,
                    complete: false,
                }
            }
            Err(_) => {
                // Can't even parse package name
                PartialParseResult::Empty
            }
        }
    }

    fn parse_package_name(&mut self) -> Result<String, ParseError> {
        let position = self.position;
        let token = self
            .expect_kind(SyntaxKind::IDENTIFIER)
            .map_err(|_| ParseError {
                message: "Expected package name".to_string(),
                position,
            })?;
        let span = token.span();
        let name = self.source[span.start.into()..span.end.into()].to_string();
        Ok(name)
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let position = self.position;
        let token = self
            .expect_kind(SyntaxKind::IDENTIFIER)
            .map_err(|_| ParseError {
                message: "Expected identifier".to_string(),
                position,
            })?;
        let span = token.span();
        let name = self.source[span.start.into()..span.end.into()].to_string();
        Ok(name)
    }

    fn parse_extras(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect_kind(SyntaxKind::BRACKET_START)?;
        self.skip_trivia();

        let mut extras = Vec::new();

        // Handle empty extras
        if self.peek_kind() == Some(SyntaxKind::BRACKET_END) {
            self.advance();
            return Ok(extras);
        }

        loop {
            let extra = self.parse_identifier()?;
            extras.push(extra);
            self.skip_trivia();

            if self.peek_kind() == Some(SyntaxKind::COMMA) {
                self.advance();
                self.skip_trivia();

                // Allow trailing comma
                if self.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                    self.advance();
                    break;
                }
            } else if self.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                self.advance();
                break;
            } else {
                return Err(ParseError {
                    message: "Expected ',' or ']' in extras".to_string(),
                    position: self.position,
                });
            }
        }

        Ok(extras)
    }

    fn has_version_operator(&self) -> bool {
        self.peek_kind().map_or(false, |k| k.is_version_operator())
    }

    fn parse_version_spec(&mut self) -> Result<VersionSpec, ParseError> {
        let mut clauses = Vec::new();

        loop {
            let position = self.position;
            let operator_token = self.advance().ok_or_else(|| ParseError {
                message: "Expected version operator".to_string(),
                position,
            })?;

            if !operator_token.kind().is_version_operator() {
                return Err(ParseError {
                    message: format!("Expected version operator, got {:?}", operator_token.kind()),
                    position: self.position,
                });
            }

            let operator = VersionOperator::from(operator_token.kind());
            self.skip_trivia();

            let version_token = self.expect_kind(SyntaxKind::VERSION_STRING)?;
            let span = version_token.span();
            let version = self.source[span.start.into()..span.end.into()].to_string();

            clauses.push(VersionClause { operator, version });
            self.skip_trivia();

            if self.peek_kind() == Some(SyntaxKind::COMMA) {
                self.advance();
                self.skip_trivia();
            } else if !self.has_version_operator() {
                break;
            }
        }

        Ok(VersionSpec { clauses })
    }

    fn parse_url(&mut self) -> Result<String, ParseError> {
        // Collect all tokens until we hit a separator
        let mut url = String::new();
        let mut start_position = None;

        while let Some(token) = self.peek() {
            match token.kind() {
                SyntaxKind::SEMICOLON | SyntaxKind::EOF | SyntaxKind::WHITESPACE => break,
                _ => {
                    if start_position.is_none() {
                        start_position = Some(token.span().start);
                    }
                    let span = token.span();
                    url.push_str(&self.source[span.start.into()..span.end.into()]);
                    self.advance();
                }
            }
        }

        if url.is_empty() {
            return Err(ParseError {
                message: "Empty URL".to_string(),
                position: self.position,
            });
        }

        Ok(url)
    }

    fn parse_marker(&mut self) -> Result<MarkerExpression, ParseError> {
        // For now, just consume everything as the marker expression
        let mut expression = String::new();
        let mut first = true;

        while let Some(token) = self.peek() {
            if token.kind() == SyntaxKind::EOF {
                break;
            }
            if token.kind() == SyntaxKind::WHITESPACE && !first {
                expression.push(' ');
                self.advance();
                continue;
            }
            first = false;
            let span = token.span();
            expression.push_str(&self.source[span.start.into()..span.end.into()]);
            self.advance();
        }

        let expression = expression.trim().to_string();
        if expression.is_empty() {
            return Err(ParseError {
                message: "Empty marker expression".to_string(),
                position: self.position,
            });
        }

        Ok(MarkerExpression { expression })
    }

    fn skip_trivia(&mut self) {
        while let Some(token) = self.peek() {
            if token.kind().is_trivia() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.lexed.tokens.get(self.position)
    }

    fn peek_kind(&self) -> Option<SyntaxKind> {
        self.peek().map(|t| t.kind())
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.lexed.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn expect_kind(&mut self, kind: SyntaxKind) -> Result<&Token, ParseError> {
        match self.peek() {
            Some(token) if token.kind() == kind => {
                let token = self.advance().unwrap();
                Ok(token)
            }
            Some(token) => Err(ParseError {
                message: format!("Expected {:?}, found {:?}", kind, token.kind()),
                position: self.position,
            }),
            None => Err(ParseError {
                message: format!("Expected {:?}, found EOF", kind),
                position: self.position,
            }),
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind().map_or(true, |k| k == SyntaxKind::EOF)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartialParseResult {
    Empty,
    PackageName {
        name: String,
        complete: bool,
    },
    ExtrasIncomplete {
        name: String,
        extras: Vec<String>,
        after_comma: bool,
    },
    AfterAt {
        name: String,
    },
    UrlIncomplete {
        name: String,
    },
    VersionIncomplete {
        name: String,
    },
    AfterSemicolon {
        name: String,
        version_spec: Option<VersionSpec>,
    },
    MarkerIncomplete {
        name: String,
        version_spec: Option<VersionSpec>,
    },
    Complete(Pep508Requirement),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_package() {
        let mut parser = Parser::new("requests");
        let req = parser.parse().unwrap();
        assert_eq!(req.name, "requests");
        assert!(req.extras.is_empty());
        assert!(req.version_spec.is_none());
        assert!(req.marker.is_none());
    }

    #[test]
    fn test_parse_with_version() {
        let mut parser = Parser::new("requests>=2.28.0");
        let req = parser.parse().unwrap();
        assert_eq!(req.name, "requests");
        assert!(req.version_spec.is_some());
        let spec = req.version_spec.unwrap();
        assert_eq!(spec.clauses.len(), 1);
        assert_eq!(spec.clauses[0].operator, VersionOperator::GreaterThanEqual);
        assert_eq!(spec.clauses[0].version, "2.28.0");
    }

    #[test]
    fn test_parse_with_extras() {
        let mut parser = Parser::new("requests[security,socks]");
        let req = parser.parse().unwrap();
        assert_eq!(req.name, "requests");
        assert_eq!(req.extras, vec!["security", "socks"]);
    }

    #[test]
    fn test_parse_with_marker() {
        let mut parser = Parser::new("requests ; python_version >= '3.8'");
        let req = parser.parse().unwrap();
        assert_eq!(req.name, "requests");
        assert!(req.marker.is_some());
    }

    #[test]
    fn test_partial_parse_incomplete_extras() {
        let mut parser = Parser::new("requests[security,");
        let result = parser.parse_partial();
        match result {
            PartialParseResult::ExtrasIncomplete {
                name,
                extras,
                after_comma,
            } => {
                assert_eq!(name, "requests");
                assert_eq!(extras, vec!["security"]);
                assert!(after_comma);
            }
            _ => panic!("Expected ExtrasIncomplete"),
        }
    }

    #[test]
    fn test_partial_parse_after_at() {
        let mut parser = Parser::new("mypackage @ ");
        let result = parser.parse_partial();
        match result {
            PartialParseResult::AfterAt { name } => {
                assert_eq!(name, "mypackage");
            }
            _ => panic!("Expected AfterAt"),
        }
    }

    #[test]
    fn test_partial_parse_after_semicolon() {
        let mut parser = Parser::new("requests ; ");
        let result = parser.parse_partial();
        match result {
            PartialParseResult::AfterSemicolon { name, version_spec } => {
                assert_eq!(name, "requests");
                assert!(version_spec.is_none());
            }
            _ => panic!("Expected AfterSemicolon"),
        }
    }
}
