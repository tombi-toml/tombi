use regex::Regex;
use tombi_comment_directive::CommentContext;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueSchema;
use tombi_x_keyword::StringFormat;

use crate::{validate::format, validate::type_mismatch};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::String {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::String(string_schema) => {
                        validate_string(self, accessors, string_schema).await
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        validate_any_of(
                            self,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        validate_all_of(
                            self,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            comment_context,
                        )
                        .await
                    }
                    ValueSchema::Null => return Ok(()),
                    value_schema => type_mismatch(
                        value_schema.value_type().await,
                        self.value_type(),
                        self.range(),
                    ),
                }
            } else {
                Ok(())
            }
        }
        .boxed()
    }
}

async fn validate_string(
    string_value: &tombi_document_tree::String,
    accessors: &[tombi_schema_store::Accessor],
    string_schema: &tombi_schema_store::StringSchema,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let mut diagnostics = vec![];
    let value = string_value.value().to_string();
    let range = string_value.range();

    if let Some(const_value) = &string_schema.const_value {
        if value != *const_value {
            crate::Error {
                kind: crate::ErrorKind::Const {
                    expected: format!("\"{const_value}\""),
                    actual: string_value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(enumerate) = &string_schema.enumerate {
        if !enumerate.contains(&value) {
            crate::Error {
                kind: crate::ErrorKind::Enumerate {
                    expected: enumerate.iter().map(|s| format!("\"{s}\"")).collect(),
                    actual: string_value.to_string(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(max_length) = &string_schema.max_length {
        if value.len() > *max_length {
            crate::Error {
                kind: crate::ErrorKind::StringMaximumLength {
                    maximum: *max_length,
                    actual: value.len(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(min_length) = &string_schema.min_length {
        if value.len() < *min_length {
            crate::Error {
                kind: crate::ErrorKind::StringMinimumLength {
                    minimum: *min_length,
                    actual: value.len(),
                },
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if let Some(format) = string_schema.format {
        match format {
            StringFormat::Email => {
                if !format::email::validate_format(&value) {
                    crate::Error {
                        kind: crate::ErrorKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        },
                        range,
                    }
                    .set_diagnostics(&mut diagnostics);
                }
            }
            StringFormat::Hostname => {
                if !format::hostname::validate_format(&value) {
                    crate::Error {
                        kind: crate::ErrorKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        },
                        range,
                    }
                    .set_diagnostics(&mut diagnostics);
                }
            }
            StringFormat::Uri => {
                if !format::uri::validate_format(&value) {
                    crate::Error {
                        kind: crate::ErrorKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        },
                        range,
                    }
                    .set_diagnostics(&mut diagnostics);
                }
            }
            StringFormat::Uuid => {
                if !format::uuid::validate_format(&value) {
                    crate::Error {
                        kind: crate::ErrorKind::StringFormat {
                            format,
                            actual: string_value.to_string(),
                        },
                        range,
                    }
                    .set_diagnostics(&mut diagnostics);
                }
            }
        }
    }

    if let Some(pattern) = &string_schema.pattern {
        if let Ok(regex) = Regex::new(pattern) {
            if !regex.is_match(&value) {
                crate::Error {
                    kind: crate::ErrorKind::StringPattern {
                        pattern: pattern.clone(),
                        actual: string_value.to_string(),
                    },
                    range,
                }
                .set_diagnostics(&mut diagnostics);
            }
        } else {
            tracing::warn!("Invalid regex pattern: {:?}", pattern);
        }
    }

    if diagnostics.is_empty() {
        if string_schema.deprecated == Some(true) {
            crate::Warning {
                kind: Box::new(crate::WarningKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(accessors),
                    string_value.to_string(),
                )),
                range,
            }
            .set_diagnostics(&mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
