//! See [`Input`].

use crate::SyntaxKind::{self, *};

#[allow(non_camel_case_types)]
type bits = u64;

/// Input for the parser -- a sequence of tokens.
///
/// As of now, parser doesn't have access to the *text* of the tokens, and makes
/// decisions based solely on their classification. Unlike `LexerToken`, the
/// `Tokens` doesn't include whitespace and comments. Main input to the parser.
///
/// Struct of arrays internally, but this shouldn't really matter.
#[derive(Debug, Default)]
pub struct Input {
    tokens: Vec<lexer::Token>,
    joints: Vec<bits>,
}

impl Input {
    pub fn new(lexed: &lexer::Lexed) -> Input {
        let _p = tracing::info_span!("Lexer<'a, SyntaxKind>::to_input").entered();

        let mut res = Input::default();
        let mut was_joint = false;
        for token in &lexed.tokens {
            let kind = token.kind();
            if kind.is_trivia() {
                was_joint = false
            } else {
                if was_joint {
                    res.was_joint();
                }
                res.push(*token);
                was_joint = true;
            }
        }
        res
    }

    fn bit_index(&self, n: usize) -> (usize, usize) {
        let idx = n / (bits::BITS as usize);
        let b_idx = n % (bits::BITS as usize);
        (idx, b_idx)
    }
}

/// impl used by callers to create `Tokens`.
impl Input {
    #[inline]
    fn push(&mut self, token: lexer::Token) {
        let idx = self.len();
        if idx % (bits::BITS as usize) == 0 {
            self.joints.push(0);
        }
        self.tokens.push(token);
    }

    /// Sets jointness for the last token we've pushed.
    ///
    /// This is a separate API rather than an argument to the `push` to make it
    /// convenient both for textual and mbe tokens. With text, you know whether
    /// the *previous* token was joint, with mbe, you know whether the *current*
    /// one is joint. This API allows for styles of usage:
    #[inline]
    fn was_joint(&mut self) {
        let n = self.len() - 1;
        let (idx, b_idx) = self.bit_index(n);
        self.joints[idx] |= 1 << b_idx;
    }
}

/// pub(crate) impl used by the parser to consume `Tokens`.
impl Input {
    pub(crate) fn token(&self, idx: usize) -> lexer::Token {
        self.tokens.get(idx).unwrap_or(&lexer::Token::eof()).clone()
    }

    pub(crate) fn kind(&self, idx: usize) -> SyntaxKind {
        self.tokens.get(idx).map_or(EOF, |t| t.kind())
    }

    pub(crate) fn range(&self, idx: usize) -> text::Range {
        self.tokens.get(idx).unwrap_or(&lexer::Token::eof()).range()
    }

    pub(crate) fn is_joint(&self, n: usize) -> bool {
        let (idx, b_idx) = self.bit_index(n);
        self.joints[idx] & 1 << b_idx != 0
    }

    pub(crate) fn len(&self) -> usize {
        self.tokens.len()
    }
}
