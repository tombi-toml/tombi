pub struct Cursor<'a> {
    /// Iterator over chars. Slightly faster than a &str.
    chars: std::str::Chars<'a>,
    current_char: char,
    current_offset: tombi_text::Offset,
    token_start_offset: tombi_text::Offset,
    /// The start offset of each line scanned so far.
    pub(crate) line_starts: Vec<tombi_text::Offset>,
}

pub(crate) const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        let chars = input.chars();
        let current = chars.clone().next().unwrap_or(EOF_CHAR);

        Cursor {
            chars,
            current_char: current,
            current_offset: Default::default(),
            token_start_offset: Default::default(),
            line_starts: vec![tombi_text::Offset::new(0)],
        }
    }

    #[inline]
    pub(crate) fn current(&self) -> char {
        self.current_char
    }

    pub fn peek(&self, i: usize) -> char {
        debug_assert!(i != 0, "peek(0) is invalid");

        self.chars
            .clone()
            .nth(i.saturating_sub(1))
            .unwrap_or(EOF_CHAR)
    }

    /// Checks if the character at the current position is expected.
    pub fn matches(&self, expected: &str) -> bool {
        let mut iter = expected.chars();
        if iter.next() != Some(self.current_char) {
            return false;
        }
        for (i, c) in iter.enumerate() {
            if self.peek(i + 1) != c {
                return false;
            }
        }
        true
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    #[inline]
    pub(crate) fn remaining(&self) -> &str {
        self.chars.as_str()
    }

    /// Consumes an ASCII prefix without decoding one character at a time.
    /// SIMD scanners call this after locating a token boundary.
    pub(crate) fn eat_ascii_bytes(&mut self, len: usize) {
        debug_assert!(len > 0);
        let remaining = self.chars.as_str();
        let prefix = &remaining.as_bytes()[..len];
        debug_assert!(prefix.is_ascii());
        debug_assert!(
            !prefix.contains(&b'\n'),
            "line starts are recorded only in `bump`"
        );
        let current_char = prefix[len - 1] as char;
        let rest = &remaining[len..];

        self.current_offset += tombi_text::Offset::new(len as u32);
        self.current_char = current_char;
        self.chars = rest.chars();
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        if let Some(c) = self.chars.next() {
            self.current_offset += tombi_text::Offset::new(c.len_utf8() as u32);
            if c == '\n' {
                self.line_starts.push(self.current_offset);
            }
            self.current_char = c;
            Some(c)
        } else {
            self.current_char = EOF_CHAR;
            None
        }
    }

    pub(crate) fn eat_n(&mut self, n: usize) {
        debug_assert!(n > 0);
        for _ in 0..n {
            if self.bump().is_none() {
                break;
            }
        }
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    pub(crate) fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek(1)) && !self.is_eof() {
            self.bump();
        }
    }

    #[inline]
    pub(crate) fn pop_span(&mut self) -> tombi_text::Span {
        let span = tombi_text::Span::new(self.token_start_offset, self.current_offset);
        self.token_start_offset = self.current_offset;
        span
    }
}
