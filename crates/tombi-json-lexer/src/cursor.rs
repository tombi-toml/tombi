/// A byte cursor over the source.
///
/// Every byte that decides a JSON token boundary is ASCII, so the lexer scans bytes
/// instead of decoding chars. Non-ASCII bytes only appear inside strings or invalid tokens,
/// where they are skipped as a whole, so the offsets always stay on char boundaries.
pub struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    token_start_offset: usize,
    /// The start offset of each line scanned so far.
    pub(crate) line_starts: Vec<tombi_text::Offset>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            bytes: input.as_bytes(),
            offset: 0,
            token_start_offset: 0,
            line_starts: vec![tombi_text::Offset::new(0)],
        }
    }

    /// Returns the byte at the cursor, without consuming it.
    #[inline]
    pub(crate) fn first(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }

    /// Returns the bytes from the cursor to the end of the source.
    #[inline]
    pub(crate) fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }

    /// Consumes `len` bytes that contain no line break.
    #[inline]
    pub(crate) fn eat_bytes(&mut self, len: usize) {
        debug_assert!(
            !self.remaining()[..len].contains(&b'\n'),
            "line breaks must be consumed by `eat_line_break`"
        );
        self.offset += len;
    }

    /// Consumes a `\n` byte and records the start of the next line.
    #[inline]
    pub(crate) fn eat_line_break(&mut self) {
        debug_assert_eq!(self.first(), Some(b'\n'));
        self.offset += 1;
        self.line_starts
            .push(tombi_text::Offset::new(self.offset as u32));
    }

    /// Consumes bytes while `predicate` returns true or until the end of file is reached.
    ///
    /// `predicate` must reject `\n`, so that line starts are recorded only in `eat_line_break`.
    #[inline]
    pub(crate) fn eat_while(&mut self, predicate: impl Fn(u8) -> bool) {
        let len = self
            .remaining()
            .iter()
            .position(|&byte| !predicate(byte))
            .unwrap_or(self.bytes.len() - self.offset);
        self.eat_bytes(len);
    }

    /// Skips whitespace and line breaks, without building their tokens.
    #[inline]
    pub(crate) fn skip_trivia(&mut self) {
        while let Some(byte) = self.first() {
            match byte {
                b' ' | b'\t' | b'\r' => self.offset += 1,
                b'\n' => self.eat_line_break(),
                _ => break,
            }
        }
        self.token_start_offset = self.offset;
    }

    #[inline]
    pub(crate) fn source_len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub(crate) fn pop_span(&mut self) -> tombi_text::Span {
        let span = tombi_text::Span::new(
            tombi_text::Offset::new(self.token_start_offset as u32),
            tombi_text::Offset::new(self.offset as u32),
        );
        self.token_start_offset = self.offset;
        span
    }
}
