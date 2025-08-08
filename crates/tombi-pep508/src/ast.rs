use crate::{lexer, Error, ErrorKind, Lexed, SyntaxKind, Token};
use tombi_rg_tree::Language;

// Data structures for parsed PEP 508 requirements
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

// Re-export AST types
pub use crate::language::Pep508Language;
pub type SyntaxNode = tombi_rg_tree::RedNode<Pep508Language>;
pub type SyntaxToken = tombi_rg_tree::RedToken<Pep508Language>;
pub type SyntaxElement = tombi_rg_tree::RedElement<Pep508Language>;
pub type SyntaxNodePtr = tombi_rg_tree::RedNodePtr<Pep508Language>;
pub type SyntaxNodeChildren = tombi_rg_tree::RedNodeChildren<Pep508Language>;
pub type SyntaxElementChildren = tombi_rg_tree::RedElementChildren<Pep508Language>;
pub type PreorderWithTokens = tombi_rg_tree::RedPreorderWithTokens<Pep508Language>;

/// Parse a PEP 508 requirement string into an AST
pub fn parse(source: &str) -> (SyntaxNode, Vec<Error>) {
    let parser = Parser::new(source);
    parser.parse_ast()
}

/// Syntax tree builder for PEP 508 AST
#[derive(Debug)]
pub struct SyntaxTreeBuilder<E> {
    inner: tombi_rg_tree::GreenNodeBuilder<'static>,
    errors: Vec<E>,
}

impl<E> SyntaxTreeBuilder<E> {
    pub fn finish(self) -> (tombi_rg_tree::GreenNode, Vec<E>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn token(&mut self, kind: crate::SyntaxKind, text: &str) {
        let kind = Pep508Language::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: crate::SyntaxKind) {
        let kind = Pep508Language::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: E) {
        self.errors.push(error);
    }
}

impl<E> Default for SyntaxTreeBuilder<E> {
    fn default() -> SyntaxTreeBuilder<E> {
        SyntaxTreeBuilder {
            inner: tombi_rg_tree::GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }
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
    pub fn parse_ast(mut self) -> (SyntaxNode, Vec<Error>) {
        let mut builder = SyntaxTreeBuilder::default();
        builder.start_node(SyntaxKind::ROOT);
        self.parse_requirement_ast(&mut builder);
        builder.finish_node();

        let (green, errors) = builder.finish();
        let node = SyntaxNode::new_root(green);
        (node, errors)
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

    // AST building methods
    fn parse_requirement_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::REQUIREMENT);

        // Skip leading trivia
        self.skip_trivia_ast(builder);

        // Parse package name
        self.parse_package_name_ast(builder);
        self.skip_trivia_ast(builder);

        // Check for extras
        if self.peek_kind() == Some(SyntaxKind::BRACKET_START) {
            self.parse_extras_ast(builder);
            self.skip_trivia_ast(builder);
        }

        // Check for URL dependency (@ url)
        if self.peek_kind() == Some(SyntaxKind::AT) {
            self.consume_token_ast(builder); // consume '@'
            self.skip_trivia_ast(builder);
            self.parse_url_ast(builder);
        } else if self.has_version_operator() {
            // Check for version spec
            self.parse_version_spec_ast(builder);
            self.skip_trivia_ast(builder);
        }

        // Check for marker
        if self.peek_kind() == Some(SyntaxKind::SEMICOLON) {
            self.consume_token_ast(builder); // consume ';'
            self.skip_trivia_ast(builder);
            self.parse_marker_ast(builder);
        }

        // Consume any remaining tokens as invalid
        while !self.is_at_end() {
            self.consume_token_ast(builder);
        }

        builder.finish_node();
    }

    fn parse_package_name_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::PACKAGE_NAME);

        if self.peek_kind() == Some(SyntaxKind::IDENTIFIER) {
            self.consume_token_ast(builder);
        } else {
            let span = self
                .peek()
                .map(|t| t.span())
                .unwrap_or_else(|| tombi_text::Span::new(0.into(), 0.into()));
            let range = tombi_text::Range::new(
                tombi_text::Position::new(0, 0),
                tombi_text::Position::new(0, 0),
            );
            builder.error(Error::new(ErrorKind::UnexpectedEndOfInput, (span, range)));
        }

        builder.finish_node();
    }

    fn parse_extras_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::EXTRAS_LIST);

        // Consume '['
        self.consume_token_ast(builder);
        self.skip_trivia_ast(builder);

        // Parse extras
        loop {
            if self.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                self.consume_token_ast(builder);
                break;
            }

            if self.is_at_end() {
                let span = tombi_text::Span::new(
                    (self.source.len() as u32).into(),
                    (self.source.len() as u32).into(),
                );
                let range = tombi_text::Range::new(
                    tombi_text::Position::new(0, 0),
                    tombi_text::Position::new(0, 0),
                );
                builder.error(Error::new(ErrorKind::UnexpectedEndOfInput, (span, range)));
                break;
            }

            // Parse extra name
            if self.peek_kind() == Some(SyntaxKind::IDENTIFIER) {
                self.consume_token_ast(builder);
                self.skip_trivia_ast(builder);

                if self.peek_kind() == Some(SyntaxKind::COMMA) {
                    self.consume_token_ast(builder);
                    self.skip_trivia_ast(builder);

                    // Allow trailing comma
                    if self.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                        self.consume_token_ast(builder);
                        break;
                    }
                } else if self.peek_kind() != Some(SyntaxKind::BRACKET_END) {
                    // Expected ',' or ']'
                    let span = self
                        .peek()
                        .map(|t| t.span())
                        .unwrap_or_else(|| tombi_text::Span::new(0.into(), 0.into()));
                    let range = tombi_text::Range::new(
                        tombi_text::Position::new(0, 0),
                        tombi_text::Position::new(0, 0),
                    );
                    builder.error(Error::new(ErrorKind::InvalidToken, (span, range)));
                    break;
                }
            } else {
                // Expected identifier
                let span = self
                    .peek()
                    .map(|t| t.span())
                    .unwrap_or_else(|| tombi_text::Span::new(0.into(), 0.into()));
                let range = tombi_text::Range::new(
                    tombi_text::Position::new(0, 0),
                    tombi_text::Position::new(0, 0),
                );
                builder.error(Error::new(ErrorKind::InvalidIdentifier, (span, range)));
                break;
            }
        }

        builder.finish_node();
    }

    fn parse_version_spec_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::VERSION_SPEC);

        loop {
            if !self.has_version_operator() {
                break;
            }

            builder.start_node(SyntaxKind::VERSION_CLAUSE);

            // Consume version operator
            self.consume_token_ast(builder);
            self.skip_trivia_ast(builder);

            // Consume version string
            if self.peek_kind() == Some(SyntaxKind::VERSION_STRING) {
                self.consume_token_ast(builder);
            } else {
                let span = self
                    .peek()
                    .map(|t| t.span())
                    .unwrap_or_else(|| tombi_text::Span::new(0.into(), 0.into()));
                let range = tombi_text::Range::new(
                    tombi_text::Position::new(0, 0),
                    tombi_text::Position::new(0, 0),
                );
                builder.error(Error::new(ErrorKind::InvalidVersion, (span, range)));
            }

            builder.finish_node();
            self.skip_trivia_ast(builder);

            // Check for comma (multiple version clauses)
            if self.peek_kind() == Some(SyntaxKind::COMMA) {
                self.consume_token_ast(builder);
                self.skip_trivia_ast(builder);
            } else if !self.has_version_operator() {
                break;
            }
        }

        builder.finish_node();
    }

    fn parse_url_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::URL_SPEC);

        // Collect all tokens until we hit a separator
        while let Some(token) = self.peek() {
            match token.kind() {
                SyntaxKind::SEMICOLON | SyntaxKind::EOF | SyntaxKind::WHITESPACE => break,
                _ => {
                    self.consume_token_ast(builder);
                }
            }
        }

        builder.finish_node();
    }

    fn parse_marker_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::MARKER_EXPR);

        // For now, just consume everything as the marker expression
        while !self.is_at_end() {
            self.consume_token_ast(builder);
        }

        builder.finish_node();
    }

    fn skip_trivia_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        while let Some(token) = self.peek() {
            if token.kind().is_trivia() {
                self.consume_token_ast(builder);
            } else {
                break;
            }
        }
    }

    fn consume_token_ast(&mut self, builder: &mut SyntaxTreeBuilder<Error>) {
        if let Some(token) = self.lexed.tokens.get(self.position) {
            let span = token.span();
            let text = &self.source[span.start.into()..span.end.into()];
            builder.token(token.kind(), text);
            self.position += 1;
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

    fn has_version_operator(&self) -> bool {
        self.peek_kind().map_or(false, |k| k.is_version_operator())
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind().map_or(true, |k| k == SyntaxKind::EOF)
    }
}