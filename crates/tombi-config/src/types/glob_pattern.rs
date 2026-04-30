#[repr(transparent)]
/// Glob pattern used by config include/exclude lists.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct GlobPattern(#[cfg_attr(feature = "jsonschema", schemars(length(min = 1)))] String);

impl GlobPattern {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    #[inline]
    pub(crate) fn as_string_slice(patterns: &[GlobPattern]) -> &[String] {
        // SAFETY: `GlobPattern` is `repr(transparent)` over `String`.
        unsafe { &*(patterns as *const [GlobPattern] as *const [String]) }
    }
}

impl std::ops::Deref for GlobPattern {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl std::convert::AsRef<str> for GlobPattern {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for GlobPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<&str> for GlobPattern {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for GlobPattern {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<GlobPattern> for String {
    fn from(value: GlobPattern) -> Self {
        value.0
    }
}
