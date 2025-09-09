use itertools::Itertools;
use tombi_schema_store::SchemaAccessors;
use tombi_severity_level::SeverityLevel;
use tombi_uri::SchemaUri;
use tombi_x_keyword::StringFormat;

#[derive(thiserror::Error, Debug)]
pub enum DiagnosticKind {
    /// The entire Table or Array is deprecated
    #[error("`{0}` is deprecated")]
    Deprecated(SchemaAccessors),

    /// The value is deprecated
    #[error("`{0} = {1}` is deprecated")]
    DeprecatedValue(SchemaAccessors, String),

    #[error(
        r#"In strict mode, `{accessors}` does not allow "{key}" key.
Please add `"additionalProperties": true` to the location where `{accessors}` is defined in {schema_uri},
or add `#:tombi schema.strict = false` as a document comment directive at the top of your document,
or set `schema.strict = false` in your `tombi.toml`."#
    )]
    StrictAdditionalKeys {
        accessors: SchemaAccessors,
        key: String,
        schema_uri: SchemaUri,
    },

    #[error("\"{key}\" is not allowed")]
    KeyNotAllowed { key: String },

    #[error("Key must match the pattern `{patterns}`")]
    KeyPattern { patterns: Patterns },

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
    StringMaxLength { maximum: usize, actual: usize },

    #[error("The length must be ≥ {minimum}, but found {actual}")]
    StringMinLength { minimum: usize, actual: usize },

    #[error("{actual} is not a valid `{format}` format")]
    StringFormat {
        format: StringFormat,
        actual: String,
    },

    #[error("{actual} does not match the pattern `{pattern}`")]
    StringPattern { pattern: String, actual: String },

    #[error("Array must contain at most {max_values} values, but found {actual}")]
    ArrayMaxValues { max_values: usize, actual: usize },

    #[error("Array must contain at least {min_values} values, but found {actual}")]
    ArrayMinValues { min_values: usize, actual: usize },

    #[error("Array values must be unique")]
    ArrayUniqueValues,

    #[error("Table must contain at most {max_keys} keys, but found {actual}")]
    TableMaxKeys { max_keys: usize, actual: usize },

    #[error("Table must contain at least {min_keys} keys, but found {actual}")]
    TableMinKeys { min_keys: usize, actual: usize },

    #[error("\"{key}\" is required")]
    TableKeyRequired { key: String },
}

#[derive(Debug)]
pub struct Diagnostic {
    pub kind: Box<DiagnosticKind>,
    pub range: tombi_text::Range,
}

impl Diagnostic {
    #[inline]
    pub fn code(&self) -> &'static str {
        match *self.kind {
            DiagnosticKind::Deprecated { .. } | DiagnosticKind::DeprecatedValue { .. } => {
                "deprecated"
            }
            DiagnosticKind::StrictAdditionalKeys { .. } => "strict-additional-keys",
            DiagnosticKind::KeyNotAllowed { .. } => "key-not-allowed",
            DiagnosticKind::KeyPattern { .. } => "key-pattern",
            DiagnosticKind::TypeMismatch { .. } => "type-mismatch",
            DiagnosticKind::Const { .. } => "const",
            DiagnosticKind::Enumerate { .. } => "enumerate",
            DiagnosticKind::IntegerMaximum { .. } => "integer-maximum",
            DiagnosticKind::IntegerMinimum { .. } => "integer-minimum",
            DiagnosticKind::IntegerExclusiveMaximum { .. } => "integer-exclusive-maximum",
            DiagnosticKind::IntegerExclusiveMinimum { .. } => "integer-exclusive-minimum",
            DiagnosticKind::IntegerMultipleOf { .. } => "integer-multiple-of",
            DiagnosticKind::FloatMaximum { .. } => "float-maximum",
            DiagnosticKind::FloatMinimum { .. } => "float-minimum",
            DiagnosticKind::FloatExclusiveMaximum { .. } => "float-exclusive-maximum",
            DiagnosticKind::FloatExclusiveMinimum { .. } => "float-exclusive-minimum",
            DiagnosticKind::FloatMultipleOf { .. } => "float-multiple-of",
            DiagnosticKind::StringMaxLength { .. } => "string-max-length",
            DiagnosticKind::StringMinLength { .. } => "string-min-length",
            DiagnosticKind::StringFormat { .. } => "string-format",
            DiagnosticKind::StringPattern { .. } => "string-pattern",
            DiagnosticKind::ArrayMaxValues { .. } => "array-max-values",
            DiagnosticKind::ArrayMinValues { .. } => "array-min-values",
            DiagnosticKind::ArrayUniqueValues => "array-unique-values",
            DiagnosticKind::TableMaxKeys { .. } => "table-max-keys",
            DiagnosticKind::TableMinKeys { .. } => "table-min-keys",
            DiagnosticKind::TableKeyRequired { .. } => "table-key-required",
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
