use tombi_syntax::{SyntaxKind, SyntaxKind::*, T};

pub(crate) const TS_LINE_END: TokenSet = TokenSet::new(&[LINE_BREAK, EOF]);
pub(crate) const TS_COMMEMT_OR_LINE_END: TokenSet = TokenSet::new(&[COMMENT, LINE_BREAK, EOF]);
pub(crate) const TS_NEXT_SECTION: TokenSet = TokenSet::new(&[T!['['], T!("[["), EOF]);
pub(crate) const TS_DANGLING_COMMENTS_KINDS: TokenSet = TokenSet::new(&[COMMENT, LINE_BREAK]);
pub(crate) const TS_LEADING_COMMENTS_KINDS: TokenSet = TokenSet::new(&[COMMENT, LINE_BREAK]);
pub(crate) const TS_TAILING_COMMENT_KINDS: TokenSet = TokenSet::new(&[COMMENT]);
pub(crate) const TS_KEY_FIRST: TokenSet = TokenSet::new(&[
    // name = "Tom"
    BARE_KEY,
    // "127.0.0.1" = "value"
    BASIC_STRING,
    // 'key2' = "value"
    LITERAL_STRING,
    // 1234 = "value"
    INTEGER_DEC,
    // 3.14159 = "pi"
    FLOAT,
    // true = "value"
    BOOLEAN,
    // 2001-02-08 = "value"
    LOCAL_DATE,
]);

/// A bit-set of `SyntaxKind`s
#[derive(Clone, Copy)]
pub(crate) struct TokenSet([u64; 3]);

impl TokenSet {
    pub(crate) const fn new(kinds: &[SyntaxKind]) -> TokenSet {
        let mut res = [0; 3];
        let mut i = 0;
        while i < kinds.len() {
            let discriminant = kinds[i] as usize;
            let idx = discriminant / 64;
            res[idx] |= 1 << (discriminant % 64);
            i += 1;
        }
        TokenSet(res)
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        let discriminant = kind as usize;
        let idx = discriminant / 64;
        let mask = 1 << (discriminant % 64);
        self.0[idx] & mask != 0
    }
}

#[test]
fn token_set_works_for_tokens() {
    use crate::SyntaxKind::*;
    let ts = TokenSet::new(&[EOF, WHITESPACE]);
    assert!(ts.contains(EOF));
    assert!(ts.contains(WHITESPACE));
    assert!(!ts.contains(EQUAL));
}
