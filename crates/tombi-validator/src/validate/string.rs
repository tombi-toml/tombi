use regex::Regex;
use tombi_comment_directive::StringValueTombiCommentDirectiveRules;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::ValueType;
use tombi_x_keyword::StringFormat;

use crate::{
    comment_directive::get_tombi_value_comment_directive_and_diagnostics, validate::format,
};

use super::{validate_all_of, validate_any_of, validate_one_of, Validate};

impl Validate for tombi_document_tree::String {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::SchemaAccessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = vec![];
            let mut comment_directive_diagnostics = vec![];

            let _comment_directive = {
                let mut comment_directives = vec![];

                for comment in self.leading_comments() {
                    if let Some(comment_directive) = comment.tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }
                if let Some(comment) = self.trailing_comment() {
                    if let Some(comment_directive) = comment.tombi_value_directive() {
                        comment_directives.push(comment_directive);
                    }
                }

                let (comment_directive, diagnostics) =
                    get_tombi_value_comment_directive_and_diagnostics::<
                        StringValueTombiCommentDirectiveRules,
                    >(&comment_directives)
                    .await;

                if !diagnostics.is_empty() {
                    comment_directive_diagnostics.extend(diagnostics);
                }

                comment_directive
            };

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.value_type().await {
                    ValueType::String
                    | ValueType::OneOf(_)
                    | ValueType::AnyOf(_)
                    | ValueType::AllOf(_) => {}
                    ValueType::Null => return Ok(()),
                    value_schema => {
                        crate::Error {
                            kind: crate::ErrorKind::TypeMismatch {
                                expected: value_schema,
                                actual: self.value_type(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);

                        return Err(diagnostics);
                    }
                }

                let string_schema = match current_schema.value_schema.as_ref() {
                    tombi_schema_store::ValueSchema::String(string_schema) => string_schema,
                    tombi_schema_store::ValueSchema::OneOf(one_of_schema) => {
                        return validate_one_of(
                            self,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::AnyOf(any_of_schema) => {
                        return validate_any_of(
                            self,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await
                    }
                    tombi_schema_store::ValueSchema::AllOf(all_of_schema) => {
                        return validate_all_of(
                            self,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                        )
                        .await
                    }
                    _ => unreachable!("Expected a String schema"),
                };

                let value = self.value().to_string();

                if let Some(const_value) = &string_schema.const_value {
                    if value != *const_value {
                        crate::Error {
                            kind: crate::ErrorKind::Const {
                                expected: format!("\"{const_value}\""),
                                actual: self.to_string(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(enumerate) = &string_schema.enumerate {
                    if !enumerate.contains(&value) {
                        crate::Error {
                            kind: crate::ErrorKind::Enumerate {
                                expected: enumerate.iter().map(|s| format!("\"{s}\"")).collect(),
                                actual: self.to_string(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(max_length) = &string_schema.max_length {
                    if value.len() > *max_length {
                        crate::Error {
                            kind: crate::ErrorKind::MaximumLength {
                                maximum: *max_length,
                                actual: value.len(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(min_length) = &string_schema.min_length {
                    if value.len() < *min_length {
                        crate::Error {
                            kind: crate::ErrorKind::MinimumLength {
                                minimum: *min_length,
                                actual: value.len(),
                            },
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }

                if let Some(format) = string_schema.format {
                    match format {
                        StringFormat::Email => {
                            if !format::email::validate_format(&value) {
                                crate::Error {
                                    kind: crate::ErrorKind::Format {
                                        format,
                                        actual: self.to_string(),
                                    },
                                    range: self.range(),
                                }
                                .set_diagnostics(&mut diagnostics);
                            }
                        }
                        StringFormat::Hostname => {
                            if !format::hostname::validate_format(&value) {
                                crate::Error {
                                    kind: crate::ErrorKind::Format {
                                        format,
                                        actual: self.to_string(),
                                    },
                                    range: self.range(),
                                }
                                .set_diagnostics(&mut diagnostics);
                            }
                        }
                        StringFormat::Uri => {
                            if !format::uri::validate_format(&value) {
                                crate::Error {
                                    kind: crate::ErrorKind::Format {
                                        format,
                                        actual: self.to_string(),
                                    },
                                    range: self.range(),
                                }
                                .set_diagnostics(&mut diagnostics);
                            }
                        }
                        StringFormat::Uuid => {
                            if !format::uuid::validate_format(&value) {
                                crate::Error {
                                    kind: crate::ErrorKind::Format {
                                        format,
                                        actual: self.to_string(),
                                    },
                                    range: self.range(),
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
                                kind: crate::ErrorKind::Pattern {
                                    pattern: pattern.clone(),
                                    actual: self.to_string(),
                                },
                                range: self.range(),
                            }
                            .set_diagnostics(&mut diagnostics);
                        }
                    } else {
                        tracing::error!("Invalid regex pattern: {:?}", pattern);
                    }
                }

                if diagnostics.is_empty() {
                    if string_schema.deprecated == Some(true) {
                        crate::Warning {
                            kind: Box::new(crate::WarningKind::DeprecatedValue(
                                tombi_schema_store::SchemaAccessors::new(accessors.to_vec()),
                                self.to_string(),
                            )),
                            range: self.range(),
                        }
                        .set_diagnostics(&mut diagnostics);
                    }
                }
            }

            if !comment_directive_diagnostics.is_empty() {
                diagnostics.extend(comment_directive_diagnostics);
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
