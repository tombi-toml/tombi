use itertools::Itertools;

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("\"{key}\" is required")]
    KeyRequired { key: String },

    #[error("\"{key}\" is not allowed")]
    KeyNotAllowed { key: String },

    #[error("Expected a value of type {expected}, but found {actual}")]
    TypeMismatch {
        expected: tombi_schema_store::ValueType,
        actual: tombi_document_tree::ValueType,
    },

    #[error("The value must be const value \"{expected}\", but found \"{actual}\"")]
    Const { expected: String, actual: String },

    #[error("The value must be one of [{}], but found {actual}", .expected.join(", "))]
    Enumerate {
        expected: Vec<String>,
        actual: String,
    },

    #[error("The value must be > {maximum}, but found {actual}")]
    MaximumInteger { maximum: i64, actual: i64 },

    #[error("The value must be < {minimum}, but found {actual}")]
    MinimumInteger { minimum: i64, actual: i64 },

    #[error("The value must be ≥ {maximum}, but found {actual}")]
    ExclusiveMaximumInteger { maximum: i64, actual: i64 },

    #[error("The value must be ≤ {minimum}, but found {actual}")]
    ExclusiveMinimumInteger { minimum: i64, actual: i64 },

    #[error("The value {actual} is not a multiple of {multiple_of}")]
    MultipleOfInteger { multiple_of: i64, actual: i64 },

    #[error("The value must be > {maximum}, but found {actual}")]
    MaximumFloat { maximum: f64, actual: f64 },

    #[error("The value must be < {minimum}, but found {actual}")]
    MinimumFloat { minimum: f64, actual: f64 },

    #[error("The value must be ≥ {maximum}, but found {actual}")]
    ExclusiveMaximumFloat { maximum: f64, actual: f64 },

    #[error("The value must be ≤ {minimum}, but found {actual}")]
    ExclusiveMinimumFloat { minimum: f64, actual: f64 },

    #[error("The value {actual} is not a multiple of {multiple_of}")]
    MultipleOfFloat { multiple_of: f64, actual: f64 },

    #[error("The length must be ≤ {maximum}, but found {actual}")]
    MaximumLength { maximum: usize, actual: usize },

    #[error("The length must be ≥ {minimum}, but found {actual}")]
    MinimumLength { minimum: usize, actual: usize },

    #[error("\"{actual}\" does not match the pattern \"{pattern}\"")]
    Pattern { pattern: String, actual: String },

    #[error("Array must contain at most {max_items} items, but found {actual}")]
    MaxItems { max_items: usize, actual: usize },

    #[error("Array must contain at least {min_items} items, but found {actual}")]
    MinItems { min_items: usize, actual: usize },

    #[error("Table must contain at most {max_properties} properties, but found {actual}")]
    MaxProperties {
        max_properties: usize,
        actual: usize,
    },

    #[error("Table must contain at least {min_properties} properties, but found {actual}")]
    MinProperties {
        min_properties: usize,
        actual: usize,
    },

    #[error("Key must match the pattern \"{patterns}\"")]
    PatternProperty { patterns: Patterns },
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: tombi_text::Range,
}

impl Error {
    #[inline]
    pub fn code(&self) -> &'static str {
        match self.kind {
            ErrorKind::KeyRequired { .. } => "key-required",
            ErrorKind::KeyNotAllowed { .. } => "key-not-allowed",
            ErrorKind::TypeMismatch { .. } => "type-mismatch",
            ErrorKind::Const { .. } => "const",
            ErrorKind::Enumerate { .. } => "enumerate",
            ErrorKind::MaximumInteger { .. } => "maximum-integer",
            ErrorKind::MinimumInteger { .. } => "minimum-integer",
            ErrorKind::ExclusiveMaximumInteger { .. } => "exclusive-maximum-integer",
            ErrorKind::ExclusiveMinimumInteger { .. } => "exclusive-minimum-integer",
            ErrorKind::MultipleOfInteger { .. } => "multiple-of-integer",
            ErrorKind::MaximumFloat { .. } => "maximum-float",
            ErrorKind::MinimumFloat { .. } => "minimum-float",
            ErrorKind::ExclusiveMaximumFloat { .. } => "exclusive-maximum-float",
            ErrorKind::ExclusiveMinimumFloat { .. } => "exclusive-minimum-float",
            ErrorKind::MultipleOfFloat { .. } => "multiple-of-float",
            ErrorKind::MaximumLength { .. } => "maximum-length",
            ErrorKind::MinimumLength { .. } => "minimum-length",
            ErrorKind::Pattern { .. } => "pattern",
            ErrorKind::MaxItems { .. } => "max-items",
            ErrorKind::MinItems { .. } => "min-items",
            ErrorKind::MaxProperties { .. } => "max-properties",
            ErrorKind::MinProperties { .. } => "min-properties",
            ErrorKind::PatternProperty { .. } => "pattern-property",
        }
    }
}

impl From<Error> for tombi_diagnostic::Diagnostic {
    fn from(error: Error) -> Self {
        tombi_diagnostic::Diagnostic::new_error(error.kind.to_string(), error.code(), error.range)
    }
}

impl tombi_diagnostic::SetDiagnostics for Error {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(self.into())
    }
}

#[derive(Debug)]
pub struct Patterns(pub Vec<String>);

impl std::fmt::Display for Patterns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() == 1 {
            write!(f, "{}", self.0[0])
        } else {
            write!(f, "{}", self.0.iter().map(|p| format!("({})", p)).join("|"))
        }
    }
}
