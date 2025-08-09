use crate::{
    ast::{Pep508Requirement, SyntaxTreeBuilder, VersionSpec},
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
