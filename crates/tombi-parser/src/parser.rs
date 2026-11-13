use tombi_ast_syntax::{
    SyntaxKind::{self, *},
    T,
};

use crate::{event::Event, marker::Marker, token_set::TokenSet};

/// Maximum nesting depth for arrays and inline tables.
///
/// Bounds recursion through `Value::parse` so a deeply nested (but finite)
/// document cannot exhaust the thread stack and abort the process. Matches
/// `serde_json`'s default recursion limit.
pub(crate) const MAX_RECURSION_DEPTH: usize = 128;

#[derive(Debug)]
pub(crate) struct Parser<'t> {
    source: &'t str,
    input_tokens: &'t [tombi_lexer::Token],
    pos: usize,
    depth: usize,
    closers: Vec<SyntaxKind>,
    synthetic_tokens: Vec<tombi_lexer::Token>,
    pub(crate) events: Vec<Event>,
    errors: Vec<crate::Error>,
}

impl<'t> Parser<'t> {
    pub(crate) fn new(source: &'t str, input_tokens: &'t [tombi_lexer::Token]) -> Self {
        Self {
            source,
            input_tokens,
            pos: 0, // Start from the beginning to preserve leading trivia
            depth: 0,
            closers: Vec::new(),
            synthetic_tokens: Vec::new(),
            events: Vec::with_capacity(input_tokens.len().saturating_mul(2)),
            errors: Vec::new(),
        }
    }

    /// Current array/inline-table nesting depth.
    #[inline]
    pub(crate) fn nested_depth(&self) -> usize {
        self.depth
    }

    /// Enter a nested array/inline table. Pair with [`Self::exit_nested`].
    #[inline]
    pub(crate) fn enter_nested(&mut self) {
        self.depth += 1;
    }

    /// Leave a nested array/inline table.
    #[inline]
    pub(crate) fn exit_nested(&mut self) {
        self.depth -= 1;
    }

    /// Register the closing token of the array/inline table being parsed.
    ///
    /// Recovery code that skips invalid tokens stops at the innermost registered
    /// closer, so the enclosing container can close on it. Pair with
    /// [`Self::pop_closer`].
    #[inline]
    pub(crate) fn push_closer(&mut self, kind: SyntaxKind) {
        self.closers.push(kind);
    }

    #[inline]
    pub(crate) fn pop_closer(&mut self) {
        self.closers.pop();
    }

    /// Whether the current token closes the innermost array/inline table.
    #[inline]
    pub(crate) fn at_closer(&self) -> bool {
        self.closers.last().is_some_and(|kind| self.at(*kind))
    }

    pub(crate) fn finish(self) -> (Vec<Event>, Vec<tombi_lexer::Token>, Vec<crate::Error>) {
        debug_assert!(self.closers.is_empty(), "unbalanced closers");
        (self.events, self.synthetic_tokens, self.errors)
    }

    #[inline]
    pub(crate) fn current(&self) -> SyntaxKind {
        self.nth(0)
    }

    #[inline]
    pub(crate) fn current_span(&self) -> tombi_text::Span {
        self.input_tokens[self.pos].span()
    }

    #[inline]
    pub(crate) fn previous_span(&self) -> tombi_text::Span {
        if self.pos == 0 {
            return tombi_text::Span::default();
        }
        let mut pos = self.pos - 1;
        while pos > 0 && self.input_tokens[pos].kind().is_trivia() {
            pos -= 1;
        }
        self.input_tokens[pos].span()
    }

    fn nth_index(&self, n: usize) -> usize {
        let mut n = n + 1;
        for (pos, token) in self.input_tokens.iter().enumerate().skip(self.pos) {
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
    pub(crate) fn nth_span(&self, n: usize) -> tombi_text::Span {
        self.nth_token(n).span()
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
        let eaten = self.eat(kind);
        debug_assert!(eaten);
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
        debug_assert!(self.nth(0) == FLOAT);
        let token = self.nth_token(0);
        let text = &self.source[token.span()];

        if !text.contains('.') {
            let m = self.start();
            self.bump_remap(BARE_KEY);
            m.complete(self, BARE_KEY);
            return;
        }

        let parts: Vec<&str> = text.split('.').collect();
        debug_assert!(parts.len() == 2);

        let key1 = {
            let m = self.start();

            let token = tombi_lexer::Token::new(
                BARE_KEY,
                tombi_text::Span::new(
                    token.span().start,
                    token.span().start + tombi_text::Offset::of(parts[0]),
                ),
            );
            self.push_synthetic_token(token);

            m.complete(self, token.kind());

            token
        };
        let dot = {
            let m = self.start();

            let token = tombi_lexer::Token::new(
                T![.],
                tombi_text::Span::new(
                    key1.span().end,
                    key1.span().end + tombi_text::Offset::of("."),
                ),
            );

            self.push_synthetic_token(token);

            m.complete(self, token.kind());

            token
        };
        {
            let m = self.start();

            let token = tombi_lexer::Token::new(
                BARE_KEY,
                tombi_text::Span::new(
                    dot.span().end,
                    dot.span().end + tombi_text::Offset::of(parts[1]),
                ),
            );

            self.push_synthetic_token(token);

            m.complete(self, token.kind())
        }
        self.pos += 1;
    }

    fn do_bump(&mut self, kind: SyntaxKind, n_raw_tokens: u8) {
        let token_index = self.nth_index(0);
        self.push_event(Event::input_token(kind, token_index, n_raw_tokens));
        let nth_index = self.nth_index((n_raw_tokens) as usize);
        self.pos = nth_index;
    }

    #[inline]
    fn push_synthetic_token(&mut self, token: tombi_lexer::Token) {
        let token_index = self.synthetic_tokens.len();
        self.synthetic_tokens.push(token);
        self.push_event(Event::synthetic_token(token.kind(), token_index));
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
        self.errors.push(error);
    }
}
