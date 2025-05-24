pub struct Cursor<'a> {
    /// Iterator over chars. Slightly faster than a &str.
    chars: std::str::Chars<'a>,
    current_char: char,
    current_offset: tombi_text::Offset,
    current_position: tombi_text::Position,
    token_start_offset: tombi_text::Offset,
    token_start_position: tombi_text::Position,
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
            current_position: Default::default(),
            token_start_offset: Default::default(),
            token_start_position: Default::default(),
        }
    }

    #[inline]
    pub(crate) fn current(&self) -> char {
        self.current_char
    }

    pub fn peek(&self, i: usize) -> char {
        assert!(i != 0, "peek(0) is invalid");

        

        self
            .chars
            .clone()
            .nth(i.saturating_sub(1))
            .unwrap_or(EOF_CHAR)
    }

    pub fn peek_with_current_while(&self, mut predicate: impl FnMut(char) -> bool) -> String {
        let iter = self.chars.clone();
        let mut s = String::new();
        s.push(self.current_char);
        for c in iter {
            if predicate(c) {
                s.push(c);
            } else {
                break;
            }
        }
        s
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

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        if let Some(c) = self.chars.next() {
            self.current_offset += tombi_text::Offset::new(c.len_utf8() as u32);
            self.current_position += tombi_text::RelativePosition::from(c);
            self.current_char = c;
            Some(c)
        } else {
            self.current_char = EOF_CHAR;
            None
        }
    }

    pub(crate) fn eat_n(&mut self, n: usize) {
        assert!(n > 0);
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
    pub(crate) fn pop_span_range(&mut self) -> (tombi_text::Span, tombi_text::Range) {
        let start_offset = self.token_start_offset;
        let end_offset = self.current_offset;
        let start_position = self.token_start_position;
        let end_position = self.current_position;

        self.token_start_offset = self.current_offset;
        self.token_start_position = self.current_position;

        (
            tombi_text::Span::new(start_offset, end_offset),
            tombi_text::Range::new(start_position, end_position),
        )
    }
}
