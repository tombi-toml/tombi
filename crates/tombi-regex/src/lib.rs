#[cfg(not(any(feature = "fancy-regex", feature = "regex")))]
compile_error!("tombi-regex: enable one of `fancy-regex` or `regex`.");

#[cfg(feature = "regex")]
type Inner = regex::Regex;

#[cfg(all(not(feature = "regex"), feature = "fancy-regex"))]
type Inner = fancy_regex::Regex;

#[cfg(feature = "regex")]
pub type Error = regex::Error;

#[cfg(all(not(feature = "regex"), feature = "fancy-regex"))]
pub type Error = fancy_regex::Error;

#[derive(Debug, Clone)]
pub struct Regex(Inner);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match {
    start: usize,
    end: usize,
}

impl Match {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

#[cfg(all(not(feature = "regex"), feature = "fancy-regex"))]
impl<'t> From<fancy_regex::Match<'t>> for Match {
    #[inline]
    fn from(matched: fancy_regex::Match<'t>) -> Self {
        Self {
            start: matched.start(),
            end: matched.end(),
        }
    }
}

#[cfg(feature = "regex")]
impl<'t> From<regex::Match<'t>> for Match {
    #[inline]
    fn from(matched: regex::Match<'t>) -> Self {
        Self {
            start: matched.start(),
            end: matched.end(),
        }
    }
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        Ok(Self(Inner::new(pattern)?))
    }

    #[inline]
    pub fn is_match(&self, text: &str) -> bool {
        #[cfg(all(not(feature = "regex"), feature = "fancy-regex"))]
        {
            self.0.is_match(text).unwrap_or(false)
        }

        #[cfg(feature = "regex")]
        {
            self.0.is_match(text)
        }
    }

    #[inline]
    pub fn find(&self, text: &str) -> Option<Match> {
        #[cfg(all(not(feature = "regex"), feature = "fancy-regex"))]
        {
            self.0.find(text).ok().and_then(|opt| opt.map(Match::from))
        }

        #[cfg(feature = "regex")]
        {
            self.0.find(text).map(Match::from)
        }
    }
}
