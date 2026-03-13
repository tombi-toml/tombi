use itertools::Itertools;
use tombi_schema_store::SchemaAccessors;
use tombi_severity_level::SeverityLevel;
use tombi_uri::SchemaUri;
use tombi_x_keyword::StringFormat;

#[derive(thiserror::Error, Debug)]
pub enum DiagnosticKind {
    #[error("An empty key is discouraged")]
    KeyEmpty,

    #[error("Don't need to use `{rule_name}.disabled = true`. Please remove it.")]
    UnusedNoqa { rule_name: &'static str },

    /// The entire Table or Array is deprecated
    #[error("`{0}` is deprecated")]
    Deprecated(SchemaAccessors),

    /// The value is deprecated
    #[error("`{0} = {1}` is deprecated")]
    DeprecatedValue(SchemaAccessors, String),

    #[error(
        "In strict mode, `{accessors}` does not allow \"{key}\" key. \
         Please add `\"additionalProperties\": true` to the location where `{accessors}` is defined in \
         {schema_uri}, or add `#:tombi schema.strict = false` as a document comment directive at the \
         top of your document, or set `schema.strict = false` in your `tombi.toml`."
    )]
    TableStrictAdditionalKeys {
        accessors: SchemaAccessors,
        key: String,
        schema_uri: SchemaUri,
    },

    #[error("\"{key}\" is not allowed")]
    KeyNotAllowed { key: String },

    #[error("Unevaluated property \"{key}\" is not allowed")]
    UnevaluatedPropertyNotAllowed { key: String },

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
    Enum {
        expected: Vec<String>,
        actual: String,
    },

    #[error("The value must be < {maximum}, but found {actual}")]
    IntegerMaximum { maximum: i64, actual: i64 },

    #[error("The value must be > {minimum}, but found {actual}")]
    IntegerMinimum { minimum: i64, actual: i64 },

    #[error("The value must be ≤ {maximum}, but found {actual}")]
    IntegerExclusiveMaximum { maximum: i64, actual: i64 },

    #[error("The value must be ≥ {minimum}, but found {actual}")]
    IntegerExclusiveMinimum { minimum: i64, actual: i64 },

    #[error("The value {actual} is not a multiple of {multiple_of}")]
    IntegerMultipleOf { multiple_of: i64, actual: i64 },

    #[error("The value must be < {maximum}, but found {actual}")]
    FloatMaximum { maximum: f64, actual: f64 },

    #[error("The value must be > {minimum}, but found {actual}")]
    FloatMinimum { minimum: f64, actual: f64 },

    #[error("The value must be ≤ {maximum}, but found {actual}")]
    FloatExclusiveMaximum { maximum: f64, actual: f64 },

    #[error("The value must be ≥ {minimum}, but found {actual}")]
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

    #[error("Array must contain at least one item matching the `contains` schema")]
    ArrayContains,

    #[error(
        "Array must contain at least {min_contains} items matching the `contains` schema, but found {actual}"
    )]
    ArrayMinContains { min_contains: usize, actual: usize },

    #[error(
        "Array must contain at most {max_contains} items matching the `contains` schema, but found {actual}"
    )]
    ArrayMaxContains { max_contains: usize, actual: usize },

    #[error("Array values must be unique")]
    ArrayUniqueValues,

    #[error("Additional items are not allowed (tuple schema has {max_items} items)")]
    ArrayAdditionalItems { max_items: usize },

    #[error("Unevaluated array item at index {index} is not allowed")]
    ArrayUnevaluatedItemNotAllowed { index: usize },

    #[error("Table must contain at most {max_keys} keys, but found {actual}")]
    TableMaxKeys { max_keys: usize, actual: usize },

    #[error("Table must contain at least {min_keys} keys, but found {actual}")]
    TableMinKeys { min_keys: usize, actual: usize },

    #[error("\"{key}\" is required")]
    TableKeyRequired { key: String },

    #[error("1 of {total_count} schemas must be matched, but found {valid_count} matched schemas")]
    OneOfMultipleMatch {
        valid_count: usize,
        total_count: usize,
    },

    #[error("1 of {total_count} schemas must be matched, but no schema candidates were available")]
    OneOfNoMatch { total_count: usize },

    #[error("The schema matches no values")]
    Nothing,

    #[error("\"not\" schema is matched")]
    NotSchemaMatch,

    #[error("When \"{dependent_key}\" is present, \"{required_key}\" is required")]
    TableDependencyRequired {
        dependent_key: String,
        required_key: String,
    },
}

#[derive(Debug)]
pub struct Diagnostic {
    pub kind: Box<DiagnosticKind>,
    pub range: tombi_text::Range,
}

impl DiagnosticKind {
    pub fn code(&self) -> &'static str {
        match *self {
            DiagnosticKind::UnusedNoqa { .. } => "unused-noqa",
            DiagnosticKind::Deprecated { .. } | DiagnosticKind::DeprecatedValue { .. } => {
                "deprecated"
            }
            DiagnosticKind::TableStrictAdditionalKeys { .. } => "table-strict-additional-keys",
            DiagnosticKind::KeyNotAllowed { .. } => "key-not-allowed",
            DiagnosticKind::UnevaluatedPropertyNotAllowed { .. } => {
                "unevaluated-property-not-allowed"
            }
            DiagnosticKind::KeyPattern { .. } => "key-pattern",
            DiagnosticKind::TypeMismatch { .. } => "type-mismatch",
            DiagnosticKind::Const { .. } => "const",
            DiagnosticKind::Enum { .. } => "enum",
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
            DiagnosticKind::ArrayContains => "array-contains",
            DiagnosticKind::ArrayMinContains { .. } => "array-min-contains",
            DiagnosticKind::ArrayMaxContains { .. } => "array-max-contains",
            DiagnosticKind::ArrayUniqueValues => "array-unique-values",
            DiagnosticKind::ArrayAdditionalItems { .. } => "array-additional-items",
            DiagnosticKind::ArrayUnevaluatedItemNotAllowed { .. } => {
                "array-unevaluated-item-not-allowed"
            }
            DiagnosticKind::TableMaxKeys { .. } => "table-max-keys",
            DiagnosticKind::TableMinKeys { .. } => "table-min-keys",
            DiagnosticKind::TableKeyRequired { .. } => "table-key-required",
            DiagnosticKind::OneOfMultipleMatch { .. } => "one-of-multiple-match",
            DiagnosticKind::OneOfNoMatch { .. } => "one-of-no-match",
            DiagnosticKind::Nothing => "nothing",
            DiagnosticKind::NotSchemaMatch => "not-schema-match",
            DiagnosticKind::KeyEmpty => "key-empty",
            DiagnosticKind::TableDependencyRequired { .. } => "table-dependency-required",
        }
    }
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind, range: impl Into<tombi_text::Range>) -> Self {
        Self {
            kind: Box::new(kind),
            range: range.into(),
        }
    }

    #[inline]
    pub fn code(&self) -> &'static str {
        self.kind.code()
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
