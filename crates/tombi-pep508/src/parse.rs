use crate::{ast::SyntaxTreeBuilder, parser::Parser, Error, ErrorKind, SyntaxKind};

/// Trait for parsing PEP 508 components
pub trait Parse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>);
}

/// Implementation for parsing a complete PEP 508 requirement
pub struct RequirementParse;

impl Parse for RequirementParse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::ROOT);
        builder.start_node(SyntaxKind::REQUIREMENT);

        // Skip leading trivia
        skip_trivia(p, builder);

        // Parse package name
        PackageNameParse::parse(p, builder);
        skip_trivia(p, builder);

        // Check for extras
        if p.peek_kind() == Some(SyntaxKind::BRACKET_START) {
            ExtrasListParse::parse(p, builder);
            skip_trivia(p, builder);
        }

        // Check for URL dependency (@ url)
        if p.peek_kind() == Some(SyntaxKind::AT) {
            consume_token(p, builder); // consume '@'
            skip_trivia(p, builder);
            UrlSpecParse::parse(p, builder);
        } else if p.has_version_operator() {
            // Check for version spec
            VersionSpecParse::parse(p, builder);
            skip_trivia(p, builder);
        }

        // Check for marker
        if p.peek_kind() == Some(SyntaxKind::SEMICOLON) {
            consume_token(p, builder); // consume ';'
            skip_trivia(p, builder);
            MarkerExprParse::parse(p, builder);
        }

        // Consume any remaining tokens as invalid
        while !p.is_at_end() {
            consume_token(p, builder);
        }

        builder.finish_node(); // REQUIREMENT
        builder.finish_node(); // ROOT
    }
}

/// Parse a package name
pub struct PackageNameParse;

impl Parse for PackageNameParse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::PACKAGE_NAME);

        if p.peek_kind() == Some(SyntaxKind::IDENTIFIER) {
            consume_token(p, builder);
        } else {
            let span = p
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
}

/// Parse extras list
pub struct ExtrasListParse;

impl Parse for ExtrasListParse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::EXTRAS_LIST);

        // Consume '['
        consume_token(p, builder);
        skip_trivia(p, builder);

        // Parse extras
        loop {
            if p.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                consume_token(p, builder);
                break;
            }

            if p.is_at_end() {
                let span = tombi_text::Span::new(
                    (p.source_len() as u32).into(),
                    (p.source_len() as u32).into(),
                );
                let range = tombi_text::Range::new(
                    tombi_text::Position::new(0, 0),
                    tombi_text::Position::new(0, 0),
                );
                builder.error(Error::new(ErrorKind::UnexpectedEndOfInput, (span, range)));
                break;
            }

            // Parse extra name
            if p.peek_kind() == Some(SyntaxKind::IDENTIFIER) {
                consume_token(p, builder);
                skip_trivia(p, builder);

                if p.peek_kind() == Some(SyntaxKind::COMMA) {
                    consume_token(p, builder);
                    skip_trivia(p, builder);

                    // Allow trailing comma
                    if p.peek_kind() == Some(SyntaxKind::BRACKET_END) {
                        consume_token(p, builder);
                        break;
                    }
                } else if p.peek_kind() != Some(SyntaxKind::BRACKET_END) {
                    // Expected ',' or ']'
                    let span = p
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
                let span = p
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
}

/// Parse version specification
pub struct VersionSpecParse;

impl Parse for VersionSpecParse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::VERSION_SPEC);

        loop {
            if !p.has_version_operator() {
                break;
            }

            builder.start_node(SyntaxKind::VERSION_CLAUSE);

            // Consume version operator
            consume_token(p, builder);
            skip_trivia(p, builder);

            // Consume version string
            if p.peek_kind() == Some(SyntaxKind::VERSION_STRING) {
                consume_token(p, builder);
            } else {
                let span = p
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
            skip_trivia(p, builder);

            // Check for comma (multiple version clauses)
            if p.peek_kind() == Some(SyntaxKind::COMMA) {
                consume_token(p, builder);
                skip_trivia(p, builder);
            } else if !p.has_version_operator() {
                break;
            }
        }

        builder.finish_node();
    }
}

/// Parse URL specification
pub struct UrlSpecParse;

impl Parse for UrlSpecParse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::URL_SPEC);

        // Collect all tokens until we hit a separator
        while let Some(token) = p.peek() {
            match token.kind() {
                SyntaxKind::SEMICOLON | SyntaxKind::EOF | SyntaxKind::WHITESPACE => break,
                _ => {
                    consume_token(p, builder);
                }
            }
        }

        builder.finish_node();
    }
}

/// Parse marker expression
pub struct MarkerExprParse;

impl Parse for MarkerExprParse {
    fn parse(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
        builder.start_node(SyntaxKind::MARKER_EXPR);

        // For now, just consume everything as the marker expression
        while !p.is_at_end() {
            consume_token(p, builder);
        }

        builder.finish_node();
    }
}

// Helper functions
fn skip_trivia(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
    while let Some(token) = p.peek() {
        if token.kind().is_trivia() {
            consume_token(p, builder);
        } else {
            break;
        }
    }
}

fn consume_token(p: &mut Parser<'_>, builder: &mut SyntaxTreeBuilder<Error>) {
    if let Some(token) = p.current_token() {
        let span = token.span();
        let text = p.token_text(span);
        builder.token(token.kind(), text);
        p.advance();
    }
}
