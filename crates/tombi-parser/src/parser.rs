use tombi_syntax::{
    SyntaxKind::{self, *},
    T,
};

use crate::{marker::Marker, token_set::TokenSet, Event};

#[derive(Debug)]
pub(crate) struct Parser<'t> {
    source: &'t str,
    input_tokens: &'t [tombi_lexer::Token],
    pos: usize,
    pub toml_version: tombi_config::TomlVersion,
    pub tokens: Vec<tombi_lexer::Token>,
    pub(crate) events: Vec<crate::Event>,
}

impl<'t> Parser<'t> {
    pub(crate) fn new(
        source: &'t str,
        toml_version: Option<tombi_config::TomlVersion>,
        input_tokens: &'t [tombi_lexer::Token],
    ) -> Self {
        Self {
            source,
            input_tokens,
            toml_version: toml_version.unwrap_or_default(),
            pos: input_tokens
                .iter()
                .enumerate()
                .find(|(_, token)| !token.kind().is_trivia())
                .map(|(i, _)| i)
                .unwrap_or_default(),
            tokens: Vec::new(),
            events: Vec::new(),
        }
    }

    pub(crate) fn finish(mut self) -> (Vec<tombi_lexer::Token>, Vec<crate::Event>) {
        for i in self.pos..self.input_tokens.len() {
            self.tokens.push(self.input_tokens[i]);
        }
        (self.tokens, self.events)
    }

    #[inline]
    pub(crate) fn current(&self) -> SyntaxKind {
        self.nth(0)
    }

    #[inline]
    pub(crate) fn current_range(&self) -> tombi_text::Range {
        self.input_tokens[self.pos].range()
    }

    #[inline]
    pub(crate) fn previous_range(&self) -> tombi_text::Range {
        if self.pos == 0 {
            return tombi_text::Range::default();
        }
        let mut pos = self.pos - 1;
        while pos > 0 && self.input_tokens[pos].kind().is_trivia() {
            pos -= 1;
        }
        self.input_tokens[pos].range()
    }

    fn nth_index(&self, n: usize) -> usize {
        let mut n = n + 1;
        for pos in self.pos..self.input_tokens.len() {
            let token = self.input_tokens[pos];
            if !token.kind().is_trivia() {
                n -= 1;
                if n == 0 {
                    return pos;
                }
            }
        }
        self.input_tokens.len()
    }

    fn nth_token(&self, n: usize) -> tombi_lexer::Token {
        let pos = self.nth_index(n);
        if pos == self.input_tokens.len() {
            return tombi_lexer::Token::eof();
        }

        self.input_tokens[pos]
    }

    #[inline]
    pub(crate) fn nth(&self, n: usize) -> SyntaxKind {
        self.nth_token(n).kind()
    }

    /// Checks if the current token is `kind`.
    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.nth_at(0, kind)
    }

    pub(crate) fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        match kind {
            T!["[["] => self.at_composite2(n, T!['['], T!['[']),
            T!["]]"] => self.at_composite2(n, T![']'], T![']']),
            _ => self.nth(n) == kind,
        }
    }

    #[inline]
    pub(crate) fn nth_range(&self, n: usize) -> tombi_text::Range {
        self.nth_token(n).range()
    }

    fn is_joint(&self, n: usize) -> bool {
        let index = self.nth_index(n);
        if index + 1 >= self.input_tokens.len() {
            return false;
        }

        !self.input_tokens[index + 1].kind().is_trivia()
    }

    fn at_composite2(&self, n: usize, k1: SyntaxKind, k2: SyntaxKind) -> bool {
        self.nth(n) == k1 && self.nth(n + 1) == k2 && self.is_joint(n)
    }

    /// Consume the next token if `kind` matches.
    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }

        self.do_bump_kind(kind);

        true
    }

    pub(crate) fn eat_ts(&mut self, kinds: TokenSet) -> bool {
        let kind = self.current();
        if !kinds.contains(kind) {
            return false;
        }

        self.do_bump_kind(kind);

        true
    }

    /// Checks if the current token is in `kinds`.
    pub(crate) fn at_ts(&self, kinds: TokenSet) -> bool {
        kinds.contains(self.current())
    }

    /// Checks if the `n`-th token is in `kinds`.
    pub(crate) fn nth_at_ts(&self, n: usize, kinds: TokenSet) -> bool {
        kinds.contains(self.nth(n))
    }

    /// Starts a new node in the syntax tree. All nodes and tokens
    /// consumed between the `start` and the corresponding `Marker::complete`
    /// belong to the same node.
    pub(crate) fn start(&mut self) -> Marker {
        let event_index = self.events.len() as u32;
        self.push_event(Event::tombstone());
        Marker::new(event_index)
    }

    /// Consume the next token. Panics if the parser isn't currently at `kind`.
    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    /// Advances the parser by one token
    pub(crate) fn bump_any(&mut self) {
        let kind = self.nth(0);
        if kind == EOF {
            return;
        }
        self.do_bump(kind, 1);
    }

    /// Advances the parser by one token, remapping its kind.
    /// This is useful to create contextual keywords from
    /// identifiers. For example, the lexer creates a `union`
    /// *identifier* token, but the parser remaps it to the
    /// `union` keyword, and keyword is what ends up in the
    /// final tree.
    pub(crate) fn bump_remap(&mut self, kind: SyntaxKind) {
        if self.nth(0) == EOF {
            // FIXME: panic!?
            return;
        }
        self.do_bump(kind, 1);
    }

    pub(crate) fn bump_float_key(&mut self) {
        assert!(self.nth(0) == FLOAT);
        let token = self.nth_token(0);
        let text = &self.source[token.span()];

        if !text.contains('.') {
            let m = self.start();
            self.bump_remap(BARE_KEY);
            m.complete(self, BARE_KEY);
            return;
        }

        let parts: Vec<&str> = text.split('.').collect();
        assert!(parts.len() == 2);

        let key1 = {
            let m = self.start();

            let token = tombi_lexer::Token::new(
                BARE_KEY,
                (
                    tombi_text::Span::new(
                        token.span().start,
                        token.span().start + tombi_text::Offset::of(parts[0]),
                    ),
                    tombi_text::Range::new(
                        token.range().start,
                        token.range().start + tombi_text::RelativePosition::of(parts[0]),
                    ),
                ),
            );
            self.tokens.push(token);
            self.push_event(Event::Token {
                kind: token.kind(),
                n_raw_tokens: 1,
            });

            m.complete(self, token.kind());

            token
        };
        let dot = {
            let m = self.start();

            let token = tombi_lexer::Token::new(
                T![.],
                (
                    tombi_text::Span::new(
                        key1.span().end,
                        key1.span().end + tombi_text::Offset::of("."),
                    ),
                    tombi_text::Range::new(
                        key1.range().end,
                        key1.range().end + tombi_text::RelativePosition::of("."),
                    ),
                ),
            );

            self.tokens.push(token);

            self.push_event(Event::Token {
                kind: token.kind(),
                n_raw_tokens: 1,
            });

            m.complete(self, token.kind());

            token
        };
        {
            let m = self.start();

            let token = tombi_lexer::Token::new(
                BARE_KEY,
                (
                    tombi_text::Span::new(
                        dot.span().end,
                        dot.span().end + tombi_text::Offset::of(parts[1]),
                    ),
                    tombi_text::Range::new(
                        dot.range().end,
                        dot.range().end + tombi_text::RelativePosition::of(parts[1]),
                    ),
                ),
            );

            self.tokens.push(token);

            self.push_event(Event::Token {
                kind: token.kind(),
                n_raw_tokens: 1,
            });

            m.complete(self, token.kind())
        }
        self.pos += 1;
    }

    fn do_bump(&mut self, kind: SyntaxKind, n_raw_tokens: u8) {
        self.push_event(Event::Token { kind, n_raw_tokens });
        let nth_index = self.nth_index((n_raw_tokens) as usize);
        self.tokens.extend(&self.input_tokens[self.pos..nth_index]);

        self.pos = nth_index;
    }

    fn do_bump_kind(&mut self, kind: SyntaxKind) {
        let n_raw_tokens = match kind {
            T!["[["] | T!["]]"] => 2,
            _ => 1,
        };

        self.do_bump(kind, n_raw_tokens);
    }

    #[inline]
    pub(crate) fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    #[inline]
    pub(crate) fn invalid_token(&mut self) {
        self.start().complete(self, INVALID_TOKEN);
    }

    /// Emit error with the `message`
    #[inline]
    pub(crate) fn error(&mut self, error: crate::Error) {
        self.push_event(Event::Error { error });
    }
}
