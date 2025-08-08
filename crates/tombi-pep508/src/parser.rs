use crate::{
    ast::{
        MarkerExpression, ParseError, Pep508Requirement, SyntaxTreeBuilder, VersionClause,
        VersionOperator, VersionSpec,
    },
    lexer, Error, Lexed, SyntaxKind, Token,
};

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

/// Parser for PEP 508 requirements
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

    /// Parse into an AST
    pub fn parse_ast(mut self) -> (crate::ast::SyntaxNode, Vec<Error>) {
        self.parse_with::<crate::parse::RequirementParse>()
    }

    /// Parse into a Pep508Requirement structure
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

    /// Parse partial input for completion
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

    // Data extraction methods
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

    // Helper methods
    fn skip_trivia(&mut self) {
        while let Some(token) = self.peek() {
            if token.kind().is_trivia() {
                self.advance();
            } else {
                break;
            }
        }
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

    // Public methods for Parse trait
    pub fn has_version_operator(&self) -> bool {
        self.peek_kind().map_or(false, |k| k.is_version_operator())
    }

    pub fn is_at_end(&self) -> bool {
        self.peek_kind().map_or(true, |k| k == SyntaxKind::EOF)
    }

    pub fn peek(&self) -> Option<&Token> {
        self.lexed.tokens.get(self.position)
    }

    pub fn peek_kind(&self) -> Option<SyntaxKind> {
        self.peek().map(|t| t.kind())
    }

    pub fn advance(&mut self) -> Option<&Token> {
        let token = self.lexed.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.lexed.tokens.get(self.position)
    }

    pub fn token_text(&self, span: tombi_text::Span) -> &str {
        &self.source[span.start.into()..span.end.into()]
    }

    pub fn source_len(&self) -> usize {
        self.source.len()
    }

    /// Parse using the Parse trait
    pub fn parse_with<P: crate::parse::Parse>(&mut self) -> (crate::ast::SyntaxNode, Vec<Error>) {
        let mut builder = SyntaxTreeBuilder::default();
        P::parse(self, &mut builder);
        let (green, errors) = builder.finish();
        let node = crate::ast::SyntaxNode::new_root(green);
        (node, errors)
    }
}
