
#[derive(Debug, Clone)]
pub struct State {
    pub chars: [bool; 256],
    pub is_star: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            chars: [false; 256],
            is_star: false,
        }
    }

    pub fn set_char(&mut self, c: u8) {
        self.chars[c as usize] = true;
    }

    pub fn set_all(&mut self) {
        self.chars = [true; 256];
    }

    pub fn matches(&self, c: u8) -> bool {
        self.chars[c as usize]
    }
}

#[derive(Debug, Clone)]
pub struct GlobPattern {
    pub states: Vec<State>,
    pub value: i64,
}

impl GlobPattern {
    pub fn new(states: Vec<State>, value: i64) -> Self {
        Self { states, value }
    }
}

#[derive(Debug)]
pub struct MultiGlob {
    patterns: Vec<GlobPattern>,
    start_states: Vec<u64>,
    compiled: bool,
}

impl MultiGlob {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            start_states: Vec::new(),
            compiled: false,
        }
    }

    pub fn add(&mut self, pattern: &str, value: i64) -> bool {
        if let Some(states) = parse_glob(pattern) {
            self.patterns.push(GlobPattern::new(states, value));
            self.compiled = false;
            true
        } else {
            false
        }
    }

    pub fn compile(&mut self) {
        if self.patterns.is_empty() {
            return;
        }

        self.patterns.sort_by(|a, b| b.value.cmp(&a.value));

        let mut all_states = Vec::new();
        for pattern in &self.patterns {
            all_states.extend_from_slice(&pattern.states);
        }

        let sz = all_states.len();
        self.start_states.resize(sz, 0);

        let mut pos = 0;
        for pattern in &self.patterns {
            let len = pattern.states.len();
            for i in 0..len {
                self.start_states[pos + i] = if i == 0 { 1u64 << (pos + i) } else { 0 };
            }
            pos += len;
        }

        self.compiled = true;
    }

    pub fn find(&self, input: &[u8]) -> Option<i64> {
        for pattern in &self.patterns {
            if self.matches_pattern(&pattern.states, input) {
                return Some(pattern.value);
            }
        }
        None
    }

    fn matches_pattern(&self, states: &[State], input: &[u8]) -> bool {
        self.match_recursive(states, 0, input, 0)
    }

    fn match_recursive(&self, states: &[State], state_idx: usize, input: &[u8], input_idx: usize) -> bool {
        if state_idx >= states.len() {
            return input_idx == input.len();
        }

        if input_idx > input.len() {
            return false;
        }

        let state = &states[state_idx];

        if state.is_star {
            if self.match_recursive(states, state_idx + 1, input, input_idx) {
                return true;
            }
            
            for i in input_idx..input.len() {
                if self.match_recursive(states, state_idx + 1, input, i + 1) {
                    return true;
                }
            }
            
            false
        } else {
            if input_idx >= input.len() {
                return false;
            }
            
            if state.matches(input[input_idx]) {
                self.match_recursive(states, state_idx + 1, input, input_idx + 1)
            } else {
                false
            }
        }
    }
}

fn parse_glob(pattern: &str) -> Option<Vec<State>> {
    let mut states: Vec<State> = Vec::new();
    let mut chars = pattern.bytes().peekable();

    while let Some(c) = chars.next() {
        let mut char_set = State::new();

        match c {
            b'*' => {
                if let Some(last_state) = states.last_mut() {
                    last_state.is_star = true;
                } else {
                    let mut star_state = State::new();
                    star_state.is_star = true;
                    states.push(star_state);
                }
                continue;
            }
            b'?' => {
                char_set.set_all();
            }
            b'\\' => {
                if let Some(&next_char) = chars.peek() {
                    char_set.set_char(next_char);
                    chars.next();
                } else {
                    return None;
                }
            }
            b'[' => {
                let mut negate = false;
                let mut bracket_chars = Vec::new();
                let mut closed = false;

                if let Some(&b'!') | Some(&b'^') = chars.peek() {
                    negate = true;
                    chars.next();
                }

                if let Some(&b']') = chars.peek() {
                    bracket_chars.push(b']');
                    chars.next();
                }

                while let Some(bracket_char) = chars.next() {
                    if bracket_char == b']' {
                        closed = true;
                        break;
                    }

                    if bracket_char == b'-' && !bracket_chars.is_empty() {
                        if let Some(&end_char) = chars.peek() {
                            if end_char != b']' {
                                let start_char = *bracket_chars.last().unwrap();
                                chars.next();
                                for ch in start_char..=end_char {
                                    bracket_chars.push(ch);
                                }
                                continue;
                            }
                        }
                    }

                    bracket_chars.push(bracket_char);
                }

                if !closed {
                    return None;
                }

                if negate {
                    char_set.set_all();
                    for ch in bracket_chars {
                        char_set.chars[ch as usize] = false;
                    }
                } else {
                    for ch in bracket_chars {
                        char_set.set_char(ch);
                    }
                }
            }
            _ => {
                char_set.set_char(c);
            }
        }

        states.push(char_set);
    }

    if states.is_empty() {
        return None;
    }

    Some(states)
}

impl Default for MultiGlob {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("hello", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"world"), None);
    }

    #[test]
    fn test_wildcard_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h*o", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"ho"), Some(1));
        assert_eq!(glob.find(b"hilo"), Some(1));
        assert_eq!(glob.find(b"hi"), None);
    }

    #[test]
    fn test_question_mark_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h?llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"hllo"), None);
    }

    #[test]
    fn test_bracket_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[aeiou]llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"hillo"), Some(1));
        assert_eq!(glob.find(b"hyllo"), None);
    }

    #[test]
    fn test_bracket_range_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[a-z]llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), Some(1));
        assert_eq!(glob.find(b"hallo"), Some(1));
        assert_eq!(glob.find(b"h9llo"), None);
    }

    #[test]
    fn test_bracket_negation_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("h[!aeiou]llo", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"hello"), None);
        assert_eq!(glob.find(b"hxllo"), Some(1));
        assert_eq!(glob.find(b"h9llo"), Some(1));
    }

    #[test]
    fn test_multiple_patterns() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("*.txt", 10));
        assert!(glob.add("test.*", 5));
        assert!(glob.add("test.txt", 20));
        glob.compile();
        
        assert_eq!(glob.find(b"test.txt"), Some(20));
        assert_eq!(glob.find(b"hello.txt"), Some(10));
        assert_eq!(glob.find(b"test.rs"), Some(5));
    }

    #[test]
    fn test_escape_pattern() {
        let mut glob = MultiGlob::new();
        assert!(glob.add("test\\*file", 1));
        glob.compile();
        
        assert_eq!(glob.find(b"test*file"), Some(1));
        assert_eq!(glob.find(b"testfile"), None);
        assert_eq!(glob.find(b"testXfile"), None);
    }
}