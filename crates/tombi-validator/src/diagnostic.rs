use itertools::Itertools;
use tombi_accessor::MarkdownSchemaAccessors;
use tombi_schema_store::SchemaAccessors;
use tombi_severity_level::SeverityLevel;
use tombi_uri::SchemaUri;
use tombi_x_keyword::StringFormat;

#[derive(thiserror::Error, Debug)]
pub enum DiagnosticKind {
    #[error("an empty key is discouraged")]
    KeyEmpty,

    #[error("don't need to use `{rule_name}.disabled = true`. Please remove it.")]
    UnusedNoqa { rule_name: &'static str },

    /// The entire Table or Array is deprecated
    #[error("`{schema_accessors}` is deprecated")]
    Deprecated { schema_accessors: SchemaAccessors },

    /// The entire Table or Array is deprecated with a custom message (`deprecationMessage`)
    #[error("`{schema_accessors}` is deprecated. {message}")]
    DeprecatedWithMessage {
        schema_accessors: SchemaAccessors,
        message: String,
    },

    /// The value is deprecated
    #[error("`{schema_accessors} = {value}` is deprecated")]
    DeprecatedValue {
        schema_accessors: SchemaAccessors,
        value: String,
    },

    /// The value is deprecated with a custom message (`deprecationMessage`)
    #[error("`{schema_accessors} = {value}` is deprecated. {message}")]
    DeprecatedValueWithMessage {
        schema_accessors: SchemaAccessors,
        value: String,
        message: String,
    },

    #[error(
        "in strict mode, {accessors} does not allow \"{key}\" key. \
         Please add `\"additionalProperties\": true` to the location where it is defined in {schema_uri}, \
         or add `#:tombi schema.strict = false` as a document comment directive at the top of your document, \
         or set `strict = false` in the matching `schemas[*].strict` / `schema.strict` entry in your `tombi.toml`."
    )]
    TableStrictAdditionalKeys {
        accessors: MarkdownSchemaAccessors,
        key: String,
        schema_uri: SchemaUri,
    },

    #[error("\"{key}\" is not allowed")]
    KeyNotAllowed { key: String },

    #[error("unevaluated property \"{key}\" is not allowed")]
    UnevaluatedPropertyNotAllowed { key: String },

    #[error("key must match the pattern `{patterns}`")]
    KeyPattern { patterns: Patterns },

    #[error("expected a value of type {expected}, but found {actual}")]
    TypeMismatch {
        expected: tombi_schema_store::ValueType,
        actual: tombi_document_tree_syntax::ValueType,
    },

    #[error("the value must be const value \"{expected}\", but found \"{actual}\"")]
    Const { expected: String, actual: String },

    #[error("the value must be one of [{}], but found {actual}", .expected.join(", "))]
    Enum {
        expected: Vec<String>,
        actual: String,
    },

    #[error("the value must be ≤ {maximum}, but found {actual}")]
    IntegerMaximum { maximum: i64, actual: i64 },

    #[error("the value must be ≥ {minimum}, but found {actual}")]
    IntegerMinimum { minimum: i64, actual: i64 },

    #[error("the value must be < {exclusive_maximum}, but found {actual}")]
    IntegerExclusiveMaximum { exclusive_maximum: i64, actual: i64 },

    #[error("the value must be > {exclusive_minimum}, but found {actual}")]
    IntegerExclusiveMinimum { exclusive_minimum: i64, actual: i64 },

    #[error("the value {actual} is not a multiple of {multiple_of}")]
    IntegerMultipleOf { multiple_of: i64, actual: i64 },

    #[error("the value must be ≤ {maximum}, but found {actual}")]
    FloatMaximum { maximum: f64, actual: f64 },

    #[error("the value must be ≥ {minimum}, but found {actual}")]
    FloatMinimum { minimum: f64, actual: f64 },

    #[error("the value must be < {exclusive_maximum}, but found {actual}")]
    FloatExclusiveMaximum { exclusive_maximum: f64, actual: f64 },

    #[error("the value must be > {exclusive_minimum}, but found {actual}")]
    FloatExclusiveMinimum { exclusive_minimum: f64, actual: f64 },

    #[error("the value {actual} is not a multiple of {multiple_of}")]
    FloatMultipleOf { multiple_of: f64, actual: f64 },

    #[error("the length must be ≤ {maximum}, but found {actual}")]
    StringMaxLength { maximum: usize, actual: usize },

    #[error("the length must be ≥ {minimum}, but found {actual}")]
    StringMinLength { minimum: usize, actual: usize },

    #[error("{actual} is not a valid `{format}` format")]
    StringFormat {
        format: StringFormat,
        actual: String,
    },

    #[error("{actual} does not match the pattern `{pattern}`")]
    StringPattern { pattern: String, actual: String },

    #[error("array must contain at most {max_values} values, but found {actual}")]
    ArrayMaxValues { max_values: usize, actual: usize },

    #[error("array must contain at least {min_values} values, but found {actual}")]
    ArrayMinValues { min_values: usize, actual: usize },

    #[error("array must contain at least one item matching the `contains` schema")]
    ArrayContains,

    #[error(
        "array must contain at least {min_contains} items matching the `contains` schema, but found {actual}"
    )]
    ArrayMinContains { min_contains: usize, actual: usize },

    #[error(
        "array must contain at most {max_contains} items matching the `contains` schema, but found {actual}"
    )]
    ArrayMaxContains { max_contains: usize, actual: usize },

    #[error("array values must be unique")]
    ArrayUniqueValues,

    #[error("additional items are not allowed (tuple schema has {max_items} items)")]
    ArrayAdditionalItems { max_items: usize },

    #[error("unevaluated array item at index {index} is not allowed")]
    ArrayUnevaluatedItemNotAllowed { index: usize },

    #[error("table must contain at most {max_keys} keys, but found {actual}")]
    TableMaxKeys { max_keys: usize, actual: usize },

    #[error("table must contain at least {min_keys} keys, but found {actual}")]
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

    #[error("the schema matches no values")]
    AlwaysFails,

    #[error("\"not\" schema is matched")]
    NotSchemaMatch,

    #[error("when \"{dependent_key}\" is present, \"{required_key}\" is required")]
    TableDependencyRequired {
        dependent_key: String,
        required_key: String,
    },
}

#[derive(Debug)]
pub struct Diagnostic {
    pub kind: Box<DiagnosticKind>,
    pub span: tombi_text::Span,
}

impl DiagnosticKind {
    pub fn code(&self) -> &'static str {
        match *self {
            DiagnosticKind::UnusedNoqa { .. } => "comment/unused-noqa",
            DiagnosticKind::Deprecated { .. }
            | DiagnosticKind::DeprecatedWithMessage { .. }
            | DiagnosticKind::DeprecatedValue { .. }
            | DiagnosticKind::DeprecatedValueWithMessage { .. } => "jsonschema/deprecated",
            DiagnosticKind::TableStrictAdditionalKeys { .. } => {
                "jsonschema/table-strict-additional-keys"
            }
            DiagnosticKind::KeyNotAllowed { .. } => "jsonschema/key-not-allowed",
            DiagnosticKind::UnevaluatedPropertyNotAllowed { .. } => {
                "jsonschema/unevaluated-property-not-allowed"
            }
            DiagnosticKind::KeyPattern { .. } => "jsonschema/key-pattern",
            DiagnosticKind::TypeMismatch { .. } => "jsonschema/type-mismatch",
            DiagnosticKind::Const { .. } => "jsonschema/const",
            DiagnosticKind::Enum { .. } => "jsonschema/enum",
            DiagnosticKind::IntegerMaximum { .. } => "jsonschema/integer-maximum",
            DiagnosticKind::IntegerMinimum { .. } => "jsonschema/integer-minimum",
            DiagnosticKind::IntegerExclusiveMaximum { .. } => {
                "jsonschema/integer-exclusive-maximum"
            }
            DiagnosticKind::IntegerExclusiveMinimum { .. } => {
                "jsonschema/integer-exclusive-minimum"
            }
            DiagnosticKind::IntegerMultipleOf { .. } => "jsonschema/integer-multiple-of",
            DiagnosticKind::FloatMaximum { .. } => "jsonschema/float-maximum",
            DiagnosticKind::FloatMinimum { .. } => "jsonschema/float-minimum",
            DiagnosticKind::FloatExclusiveMaximum { .. } => "jsonschema/float-exclusive-maximum",
            DiagnosticKind::FloatExclusiveMinimum { .. } => "jsonschema/float-exclusive-minimum",
            DiagnosticKind::FloatMultipleOf { .. } => "jsonschema/float-multiple-of",
            DiagnosticKind::StringMaxLength { .. } => "jsonschema/string-max-length",
            DiagnosticKind::StringMinLength { .. } => "jsonschema/string-min-length",
            DiagnosticKind::StringFormat { .. } => "jsonschema/string-format",
            DiagnosticKind::StringPattern { .. } => "jsonschema/string-pattern",
            DiagnosticKind::ArrayMaxValues { .. } => "jsonschema/array-max-values",
            DiagnosticKind::ArrayMinValues { .. } => "jsonschema/array-min-values",
            DiagnosticKind::ArrayContains => "jsonschema/array-contains",
            DiagnosticKind::ArrayMinContains { .. } => "jsonschema/array-min-contains",
            DiagnosticKind::ArrayMaxContains { .. } => "jsonschema/array-max-contains",
            DiagnosticKind::ArrayUniqueValues => "jsonschema/array-unique-values",
            DiagnosticKind::ArrayAdditionalItems { .. } => "jsonschema/array-additional-items",
            DiagnosticKind::ArrayUnevaluatedItemNotAllowed { .. } => {
                "jsonschema/array-unevaluated-item-not-allowed"
            }
            DiagnosticKind::TableMaxKeys { .. } => "jsonschema/table-max-keys",
            DiagnosticKind::TableMinKeys { .. } => "jsonschema/table-min-keys",
            DiagnosticKind::TableKeyRequired { .. } => "jsonschema/table-key-required",
            DiagnosticKind::OneOfMultipleMatch { .. } => "jsonschema/one-of-multiple-match",
            DiagnosticKind::OneOfNoMatch { .. } => "jsonschema/one-of-no-match",
            DiagnosticKind::AlwaysFails => "jsonschema/always-fails",
            DiagnosticKind::NotSchemaMatch => "jsonschema/not-schema-match",
            DiagnosticKind::KeyEmpty => "discouraged/key-empty",
            DiagnosticKind::TableDependencyRequired { .. } => {
                "jsonschema/table-dependency-required"
            }
        }
    }
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind, span: impl Into<tombi_text::Span>) -> Self {
        Self {
            kind: Box::new(kind),
            span: span.into(),
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
                self.span,
            )),
            SeverityLevel::Warn => diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
                self.kind.to_string(),
                self.code(),
                self.span,
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
