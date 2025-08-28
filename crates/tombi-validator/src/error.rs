use itertools::Itertools;
use tombi_severity_level::SeverityLevel;
use tombi_x_keyword::StringFormat;

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

    #[error("Expected a value of type {expected}, but found {actual}")]
    TypeMismatch2 {
        expected: tombi_schema_store::ValueType,
        actual: tombi_schema_store::ValueType,
    },

    #[error("The value must be const value \"{expected}\", but found \"{actual}\"")]
    Const { expected: String, actual: String },

    #[error("The value must be one of [{}], but found {actual}", .expected.join(", "))]
    Enumerate {
        expected: Vec<String>,
        actual: String,
    },

    #[error("The value must be > {maximum}, but found {actual}")]
    IntegerMaximum { maximum: i64, actual: i64 },

    #[error("The value must be < {minimum}, but found {actual}")]
    IntegerMinimum { minimum: i64, actual: i64 },

    #[error("The value must be ≥ {maximum}, but found {actual}")]
    IntegerExclusiveMaximum { maximum: i64, actual: i64 },

    #[error("The value must be ≤ {minimum}, but found {actual}")]
    IntegerExclusiveMinimum { minimum: i64, actual: i64 },

    #[error("The value {actual} is not a multiple of {multiple_of}")]
    IntegerMultipleOf { multiple_of: i64, actual: i64 },

    #[error("The value must be > {maximum}, but found {actual}")]
    FloatMaximum { maximum: f64, actual: f64 },

    #[error("The value must be < {minimum}, but found {actual}")]
    FloatMinimum { minimum: f64, actual: f64 },

    #[error("The value must be ≥ {maximum}, but found {actual}")]
    FloatExclusiveMaximum { maximum: f64, actual: f64 },

    #[error("The value must be ≤ {minimum}, but found {actual}")]
    FloatExclusiveMinimum { minimum: f64, actual: f64 },

    #[error("The value {actual} is not a multiple of {multiple_of}")]
    FloatMultipleOf { multiple_of: f64, actual: f64 },

    #[error("The length must be ≤ {maximum}, but found {actual}")]
    StringMaximumLength { maximum: usize, actual: usize },

    #[error("The length must be ≥ {minimum}, but found {actual}")]
    StringMinimumLength { minimum: usize, actual: usize },

    #[error("{actual} is not a valid `{format}` format")]
    StringFormat {
        format: StringFormat,
        actual: String,
    },

    #[error("{actual} does not match the pattern `{pattern}`")]
    StringPattern { pattern: String, actual: String },

    #[error("Array must contain at most {max_values} values, but found {actual}")]
    ArrayMaxItems { max_values: usize, actual: usize },

    #[error("Array must contain at least {min_values} values, but found {actual}")]
    ArrayMinItems { min_values: usize, actual: usize },

    #[error("Array values must be unique")]
    ArrayUniqueItems,

    #[error("Table must contain at most {max_properties} properties, but found {actual}")]
    TableMaxProperties {
        max_properties: usize,
        actual: usize,
    },

    #[error("Table must contain at least {min_properties} properties, but found {actual}")]
    TableMinProperties {
        min_properties: usize,
        actual: usize,
    },

    #[error("Key must match the pattern `{patterns}`")]
    KeyPattern { patterns: Patterns },
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
            ErrorKind::TypeMismatch { .. } | ErrorKind::TypeMismatch2 { .. } => "type-mismatch",
            ErrorKind::Const { .. } => "const",
            ErrorKind::Enumerate { .. } => "enumerate",
            ErrorKind::IntegerMaximum { .. } => "integer-maximum",
            ErrorKind::IntegerMinimum { .. } => "integer-minimum",
            ErrorKind::IntegerExclusiveMaximum { .. } => "integer-exclusive-maximum",
            ErrorKind::IntegerExclusiveMinimum { .. } => "integer-exclusive-minimum",
            ErrorKind::IntegerMultipleOf { .. } => "integer-multiple-of",
            ErrorKind::FloatMaximum { .. } => "float-maximum",
            ErrorKind::FloatMinimum { .. } => "float-minimum",
            ErrorKind::FloatExclusiveMaximum { .. } => "float-exclusive-maximum",
            ErrorKind::FloatExclusiveMinimum { .. } => "float-exclusive-minimum",
            ErrorKind::FloatMultipleOf { .. } => "float-multiple-of",
            ErrorKind::StringMaximumLength { .. } => "string-maximum-length",
            ErrorKind::StringMinimumLength { .. } => "string-minimum-length",
            ErrorKind::StringFormat { .. } => "string-format",
            ErrorKind::StringPattern { .. } => "string-pattern",
            ErrorKind::ArrayMaxItems { .. } => "array-max-items",
            ErrorKind::ArrayMinItems { .. } => "array-min-items",
            ErrorKind::ArrayUniqueItems => "array-unique-items",
            ErrorKind::TableMaxProperties { .. } => "table-max-properties",
            ErrorKind::TableMinProperties { .. } => "table-min-properties",
            ErrorKind::KeyPattern { .. } => "key-pattern",
        }
    }

    pub fn push_diagnostic_with_level(
        self,
        level: impl Into<SeverityLevel>,
        diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    ) {
        match level.into() {
            SeverityLevel::Error => diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                self.kind.to_string(),
                self.code(),
                self.range,
            )),
            SeverityLevel::Warn => diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
                self.kind.to_string(),
                self.code(),
                self.range,
            )),
            SeverityLevel::Off => {}
        }
    }
}

#[derive(Debug)]
pub struct Patterns(pub Vec<String>);

impl std::fmt::Display for Patterns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() == 1 {
            write!(f, "{}", self.0[0])
        } else {
            write!(f, "{}", self.0.iter().map(|p| format!("({p})")).join("|"))
        }
    }
}
