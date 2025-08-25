use regex::Regex;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;
use tombi_x_keyword::StringFormat;

use crate::{validate::format, Validate};

fn get_string_value(token: Option<tombi_syntax::SyntaxToken>) -> String {
    token
        .map(|t| {
            let text = t.text();
            // Remove quotes and process escape sequences
            if text.starts_with("\"\"\"") || text.starts_with("'''") {
                // Multi-line string
                text[3..text.len() - 3].to_string()
            } else if text.starts_with('"') || text.starts_with('\'') {
                // Single-line string
                text[1..text.len() - 1].to_string()
            } else {
                text.to_string()
            }
        })
        .unwrap_or_default()
}

impl Validate for tombi_ast::BasicString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = vec![];

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    ValueType::String => {}
                    ValueType::Null => return Ok(()),
                    value_type => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch2 {
                                expected: value_type,
                                actual: ValueType::String,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                        return Err(diagnostics);
                    }
                }

                if let tombi_schema_store::ValueSchema::String(string_schema) =
                    current_schema.value_schema.as_ref()
                {
                    let value = get_string_value(self.token());

                    // Validate const value
                    if let Some(const_value) = &string_schema.const_value {
                        if value != *const_value {
                            crate::Error {
                                kind: crate::ErrorKind::Const {
                                    expected: format!("\"{const_value}\""),
                                    actual: format!("\"{}\"", value),
                                },
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }
                    }

                    // Validate enum
                    if let Some(enumerate) = &string_schema.enumerate {
                        if !enumerate.contains(&value) {
                            crate::Error {
                                kind: crate::ErrorKind::Enumerate {
                                    expected: enumerate
                                        .iter()
                                        .map(|s| format!("\"{s}\""))
                                        .collect(),
                                    actual: format!("\"{}\"", value),
                                },
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
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
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
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
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }
                    }

                    // Validate format
                    if let Some(format) = string_schema.format {
                        let valid = match format {
                            StringFormat::Email => format::email::validate_format(&value),
                            StringFormat::Hostname => format::hostname::validate_format(&value),
                            StringFormat::Uri => format::uri::validate_format(&value),
                            StringFormat::Uuid => format::uuid::validate_format(&value),
                        };

                        if !valid {
                            crate::Error {
                                kind: crate::ErrorKind::StringFormat {
                                    format,
                                    actual: format!("\"{}\"", value),
                                },
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }
                    }

                    // Validate pattern
                    if let Some(pattern) = &string_schema.pattern {
                        if let Ok(regex) = Regex::new(pattern) {
                            if !regex.is_match(&value) {
                                crate::Error {
                                    kind: crate::ErrorKind::StringPattern {
                                        pattern: pattern.clone(),
                                        actual: format!("\"{}\"", value),
                                    },
                                    range: self.range(),
                                }
                                .set_diagnostics(&mut diagnostics);
                            }
                        }
                    }

                    // Check deprecated
                    if diagnostics.is_empty() && string_schema.deprecated == Some(true) {
                        crate::Warning {
                            kind: Box::new(crate::WarningKind::DeprecatedValue(
                                tombi_schema_store::SchemaAccessors::new(accessors.to_vec()),
                                format!("\"{}\"", value),
                            )),
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
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
}

impl Validate for tombi_ast::LiteralString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        // Use the same validation logic as BasicString
        async move {
            let mut diagnostics = vec![];

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    ValueType::String => {}
                    ValueType::Null => return Ok(()),
                    value_type => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch2 {
                                expected: value_type,
                                actual: ValueType::String,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                        return Err(diagnostics);
                    }
                }

                if let tombi_schema_store::ValueSchema::String(string_schema) =
                    current_schema.value_schema.as_ref()
                {
                    let value = get_string_value(self.token());
                    validate_string_schema(
                        &value,
                        string_schema,
                        self.range(),
                        accessors,
                        &mut diagnostics,
                    );
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
}

impl Validate for tombi_ast::MultiLineBasicString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = vec![];

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    tombi_schema_store::ValueType::String => {}
                    tombi_schema_store::ValueType::Null => return Ok(()),
                    value_type => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch2 {
                                expected: value_type,
                                actual: ValueType::String,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                        return Err(diagnostics);
                    }
                }

                if let tombi_schema_store::ValueSchema::String(string_schema) =
                    current_schema.value_schema.as_ref()
                {
                    let value = get_string_value(self.token());
                    validate_string_schema(
                        &value,
                        string_schema,
                        self.range(),
                        accessors,
                        &mut diagnostics,
                    );
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
}

impl Validate for tombi_ast::MultiLineLiteralString {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
        _comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = vec![];

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    tombi_schema_store::ValueType::String => {}
                    tombi_schema_store::ValueType::Null => return Ok(()),
                    value_type => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch2 {
                                expected: value_type,
                                actual: ValueType::String,
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                        return Err(diagnostics);
                    }
                }

                if let tombi_schema_store::ValueSchema::String(string_schema) =
                    current_schema.value_schema.as_ref()
                {
                    let value = get_string_value(self.token());
                    validate_string_schema(
                        &value,
                        string_schema,
                        self.range(),
                        accessors,
                        &mut diagnostics,
                    );
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
}

// Helper function to validate string against schema
fn validate_string_schema(
    value: &str,
    string_schema: &tombi_schema_store::StringSchema,
    range: tombi_text::Range,
    accessors: &[tombi_schema_store::SchemaAccessor],
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
                tombi_schema_store::SchemaAccessors::new(accessors.to_vec()),
                format!("\"{}\"", value),
            )),
            range,
        }
        .set_diagnostics(diagnostics);
    }
}
