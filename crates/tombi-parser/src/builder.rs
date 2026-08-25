use tombi_ast_syntax::SyntaxKind;

pub struct Builder<'a, F> {
    token_index: usize,
    tokens: &'a [tombi_lexer::Token],
    synthetic_tokens: &'a [tombi_lexer::Token],
    state: State,
    sink: F,
}

impl<F> std::fmt::Debug for Builder<'_, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("token_index", &self.token_index)
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Debug)]
pub enum State {
    PendingEnter,
    Normal,
    PendingExit,
}

impl<'a, F> Builder<'a, F>
where
    F: FnMut(Step),
{
    pub fn new(
        tokens: &'a [tombi_lexer::Token],
        synthetic_tokens: &'a [tombi_lexer::Token],
        sink: F,
    ) -> Self {
        Self {
            token_index: 0,
            tokens,
            synthetic_tokens,
            state: State::PendingEnter,
            sink,
        }
    }

    pub fn token(&mut self, kind: SyntaxKind, source: crate::event::TokenSource) {
        match std::mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(Step::FinishNode),
            State::Normal => (),
        }
        match source {
            crate::event::TokenSource::Input { index, count } => {
                let index = index as usize;
                self.eat_trivias_until(index);
                debug_assert_eq!(self.token_index, index);
                let end_index = index + count as usize;
                let span = tombi_text::Span::new(
                    self.tokens[index].span().start,
                    self.tokens[end_index - 1].span().end,
                );
                self.token_index = end_index;
                self.add_token(kind, span);
            }
            crate::event::TokenSource::Synthetic { index } => {
                let token = self.synthetic_tokens[index as usize];
                self.eat_trivias_before(token.span().start);
                self.add_token(kind, token.span());
                while self.token_index < self.tokens.len()
                    && self.tokens[self.token_index].span().end <= token.span().end
                {
                    self.token_index += 1;
                }
            }
        }
    }

    pub fn enter(&mut self, kind: SyntaxKind) {
        match std::mem::replace(&mut self.state, State::Normal) {
            State::PendingEnter => {
                (self.sink)(Step::StartNode { kind });
                // No need to attach trivias to previous node: there is no
                // previous node.
                return;
            }
            State::PendingExit => (self.sink)(Step::FinishNode),
            State::Normal => (),
        }

        self.eat_trivias();
        (self.sink)(Step::StartNode { kind });
    }

    pub fn exit(&mut self) {
        match std::mem::replace(&mut self.state, State::PendingExit) {
            State::PendingEnter => unreachable!(),
            State::PendingExit => (self.sink)(Step::FinishNode),
            State::Normal => (),
        }
    }

    pub fn eat_trivias(&mut self) {
        while self.token_index < self.tokens.len() {
            let kind = self.tokens[self.token_index].kind();
            if !kind.is_trivia() {
                break;
            }
            let span = self.tokens[self.token_index].span();
            self.token_index += 1;
            self.add_token(kind, span);
        }
    }

    fn eat_trivias_until(&mut self, index: usize) {
        while self.token_index < index {
            let token = self.tokens[self.token_index];
            debug_assert!(token.kind().is_trivia());
            self.token_index += 1;
            self.add_token(token.kind(), token.span());
        }
    }

    fn eat_trivias_before(&mut self, offset: tombi_text::Offset) {
        while self.token_index < self.tokens.len() {
            let token = self.tokens[self.token_index];
            if !token.kind().is_trivia() || token.span().end > offset {
                break;
            }
            self.token_index += 1;
            self.add_token(token.kind(), token.span());
        }
    }

    fn add_token(&mut self, kind: SyntaxKind, span: tombi_text::Span) {
        (self.sink)(Step::AddToken { kind, span });
    }
}

#[derive(Debug)]
pub enum Step {
    AddToken {
        kind: SyntaxKind,
        span: tombi_text::Span,
    },
    StartNode {
        kind: SyntaxKind,
    },
    FinishNode,
}

pub fn intersperse_trivia(
    tokens: &[tombi_lexer::Token],
    synthetic_tokens: &[tombi_lexer::Token],
    events: &[crate::event::Event],
    sink: impl FnMut(Step),
) {
    let mut builder = Builder::new(tokens, synthetic_tokens, sink);

    crate::event::process(events, |event| match event {
        crate::event::Step::Token { kind, source } => builder.token(kind, source),
        crate::event::Step::Enter { kind } => builder.enter(kind),
        crate::event::Step::Exit => builder.exit(),
    });

    match std::mem::replace(&mut builder.state, State::Normal) {
        State::PendingExit => {
            builder.eat_trivias();
            (builder.sink)(Step::FinishNode);
        }
        State::PendingEnter | State::Normal => unreachable!(),
    }
}
