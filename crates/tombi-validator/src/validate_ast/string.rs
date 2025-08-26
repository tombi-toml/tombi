use itertools::Itertools;
use regex::Regex;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;
use tombi_toml_text::{
    try_from_basic_string, try_from_literal_string, try_from_multi_line_basic_string,
    try_from_multi_line_literal_string,
};
use tombi_x_keyword::StringFormat;

use crate::{
    validate::format,
    validate_ast::{
        all_of::validate_all_of, any_of::validate_any_of, one_of::validate_one_of, Validate,
        ValueImpl,
    },
};

impl Validate for tombi_ast::BasicString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(string_value) = self.token().and_then(|token| {
                try_from_basic_string(token.text(), schema_context.toml_version).ok()
            }) else {
                return Ok(());
            };

            validate_string(
                &string_value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::BasicString {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl Validate for tombi_ast::LiteralString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        // Use the same validation logic as BasicString
        async move {
            let Some(string_value) = self
                .token()
                .and_then(|token| try_from_literal_string(token.text()).ok())
            else {
                return Ok(());
            };

            validate_string(
                &string_value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::LiteralString {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl Validate for tombi_ast::MultiLineBasicString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(string_value) = self.token().and_then(|token| {
                try_from_multi_line_basic_string(token.text(), schema_context.toml_version).ok()
            }) else {
                return Ok(());
            };

            validate_string(
                &string_value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::MultiLineBasicString {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

impl Validate for tombi_ast::MultiLineLiteralString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let Some(string_value) = self
                .token()
                .and_then(|token| try_from_multi_line_literal_string(token.text()).ok())
            else {
                return Ok(());
            };

            validate_string(
                &string_value,
                self,
                accessors,
                current_schema,
                schema_context,
                comment_context,
            )
            .await
        }
        .boxed()
    }
}

impl ValueImpl for tombi_ast::MultiLineLiteralString {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn range(&self) -> tombi_text::Range {
        self.range()
    }
}

fn validate_string<'a: 'b, 'b, T>(
    string_value: &'a str,
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext,
    comment_context: &'a CommentContext<'a>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + ValueImpl + Sync + Send + std::fmt::Debug,
{
    async move {
        let mut diagnostics = vec![];

        if let Some(current_schema) = current_schema {
            match current_schema.value_schema.as_ref() {
                tombi_schema_store::ValueSchema::String(string_schema) => validate_string_schema(
                    string_value,
                    string_schema,
                    value.range(),
                    accessors,
                    &mut diagnostics,
                ),
                tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                    return validate_one_of(
                        value,
                        accessors,
                        one_of_schema,
                        current_schema,
                        schema_context,
                        comment_context,
                    )
                    .await
                }
                tombi_schema_store::ValueSchema::AnyOf(any_of_schema) => {
                    return validate_any_of(
                        value,
                        accessors,
                        any_of_schema,
                        current_schema,
                        schema_context,
                        comment_context,
                    )
                    .await
                }
                tombi_schema_store::ValueSchema::AllOf(all_of_schema) => {
                    return validate_all_of(
                        value,
                        accessors,
                        all_of_schema,
                        current_schema,
                        schema_context,
                        comment_context,
                    )
                    .await
                }
                tombi_schema_store::ValueSchema::Null => return Ok(()),
                schema => {
                    crate::Error {
                        kind: crate::ErrorKind::TypeMismatch2 {
                            expected: schema.value_type().await,
                            actual: ValueType::String,
                        },
                        range: value.range(),
                    }
                    .set_diagnostics(&mut diagnostics);
                    return Err(diagnostics);
                }
            }
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
    .boxed()
}

// Helper function to validate string against schema
fn validate_string_schema(
    value: &str,
    string_schema: &tombi_schema_store::StringSchema,
    range: tombi_text::Range,
    accessors: &[tombi_schema_store::Accessor],
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    // Validate const value
    if let Some(const_value) = &string_schema.const_value {
        if value != const_value {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: format!("\"{const_value}\""),
                    actual: format!("\"{}\"", value),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    // Validate enum
    if let Some(enumerate) = &string_schema.enumerate {
        if enumerate.iter().any(|s| s == value) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(|s| format!("\"{s}\"")).collect(),
                    actual: format!("\"{}\"", value),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    // Validate max length
    if let Some(max_length) = &string_schema.max_length {
        if value.len() > *max_length {
            crate::Error {
                kind: crate::ErrorKind::StringMaximumLength {
                    maximum: *max_length,
                    actual: value.len(),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    // Validate min length
    if let Some(min_length) = &string_schema.min_length {
        if value.len() < *min_length {
            crate::Error {
                kind: crate::ErrorKind::StringMinimumLength {
                    minimum: *min_length,
                    actual: value.len(),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    // Validate format
    if let Some(format) = string_schema.format {
        let valid = match format {
            StringFormat::Email => format::email::validate_format(value),
            StringFormat::Hostname => format::hostname::validate_format(value),
            StringFormat::Uri => format::uri::validate_format(value),
            StringFormat::Uuid => format::uuid::validate_format(value),
        };

        if !valid {
            crate::Error {
                kind: crate::ErrorKind::StringFormat {
                    format,
                    actual: format!("\"{}\"", value),
                },
                range,
            }
            .set_diagnostics(diagnostics);
        }
    }

    // Validate pattern
    if let Some(pattern) = &string_schema.pattern {
        if let Ok(regex) = Regex::new(pattern) {
            if !regex.is_match(value) {
                crate::Error {
                    kind: crate::ErrorKind::StringPattern {
                        pattern: pattern.clone(),
                        actual: format!("\"{}\"", value),
                    },
                    range,
                }
                .set_diagnostics(diagnostics);
            }
        }
    }

    // Check deprecated
    if diagnostics.is_empty() && string_schema.deprecated == Some(true) {
        crate::Warning {
            kind: Box::new(crate::WarningKind::DeprecatedValue(
                tombi_schema_store::SchemaAccessors::new(
                    accessors.into_iter().map(Into::into).collect_vec(),
                ),
                format!("\"{}\"", value),
            )),
            range,
        }
        .set_diagnostics(diagnostics);
    }
}
