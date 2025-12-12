mod all_of;
mod any_of;
mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod not_schema;
mod offset_date_time;
mod one_of;
mod string;
mod table;
mod value;

pub mod format {
    pub mod email;
    pub mod hostname;
    pub mod uri;
    pub mod uuid;
}

use std::borrow::Cow;

pub use all_of::validate_all_of;
pub use any_of::validate_any_of;
use itertools::Itertools;
pub use one_of::validate_one_of;
use tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
use tombi_document_tree::{TryIntoDocumentTree, dig_keys};
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;
use tombi_severity_level::{SeverityLevel, SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_text::RelativePosition;

pub fn validate<'a: 'b, 'b>(
    tree: tombi_document_tree::DocumentTree,
    source_schema: Option<&'a tombi_schema_store::SourceSchema>,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
    async move {
        let current_schema = source_schema.as_ref().and_then(|source_schema| {
            source_schema.root_schema.as_ref().and_then(|root_schema| {
                root_schema
                    .value_schema
                    .as_ref()
                    .map(|value_schema| CurrentSchema {
                        value_schema: Cow::Borrowed(value_schema),
                        schema_uri: Cow::Borrowed(&root_schema.schema_uri),
                        definitions: Cow::Borrowed(&root_schema.definitions),
                    })
            })
        });

        if let Err(crate::Error { diagnostics, .. }) = tree
            .validate(&[], current_schema.as_ref(), schema_context)
            .await
        {
            Err(diagnostics.into_iter().unique().collect_vec())
        } else {
            Ok(())
        }
    }
    .boxed()
}

pub trait Validate {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> BoxFuture<'b, Result<(), crate::Error>>;
}

pub fn handle_deprecated<T>(
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    deprecated: Option<bool>,
    accessors: &[tombi_schema_store::Accessor],
    value: &T,
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) where
    T: tombi_document_tree::ValueImpl,
{
    if deprecated == Some(true) {
        let level = common_rules
            .and_then(|rules| {
                rules
                    .deprecated
                    .as_ref()
                    .map(SeverityLevelDefaultWarn::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::Deprecated(
                tombi_schema_store::SchemaAccessors::from(accessors),
            )),
            range: value.range(),
        }
        .push_diagnostic_with_level(level, diagnostics);
    } else if common_rules
        .and_then(|rules| rules.deprecated.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(diagnostics, comment_directives, common_rules, "deprecated");
    }
}

pub fn handle_deprecated_value<T>(
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    deprecated: Option<bool>,
    accessors: &[tombi_schema_store::Accessor],
    value: &T,
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) where
    T: tombi_document_tree::ValueImpl + ToString,
{
    if deprecated == Some(true) {
        let level = common_rules
            .and_then(|rules| {
                rules
                    .deprecated
                    .as_ref()
                    .map(SeverityLevelDefaultWarn::from)
            })
            .unwrap_or_default();

        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::DeprecatedValue(
                tombi_schema_store::SchemaAccessors::from(accessors),
                value.to_string(),
            )),
            range: value.range(),
        }
        .push_diagnostic_with_level(level, diagnostics);
    } else if common_rules
        .and_then(|rules| rules.deprecated.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        handle_unused_noqa(diagnostics, comment_directives, common_rules, "deprecated");
    }
}

fn handle_type_mismatch(
    expected: tombi_schema_store::ValueType,
    actual: tombi_document_tree::ValueType,
    range: tombi_text::Range,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) -> Result<(), crate::Error> {
    let mut diagnostics = vec![];

    let level = common_rules
        .and_then(|common_rules| {
            common_rules
                .type_mismatch
                .as_ref()
                .map(SeverityLevelDefaultError::from)
        })
        .unwrap_or_default();

    crate::Diagnostic {
        kind: Box::new(crate::DiagnosticKind::TypeMismatch { expected, actual }),
        range,
    }
    .push_diagnostic_with_level(level, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(crate::Error {
            score: 0,
            diagnostics,
        })
    }
}

fn handle_unused_noqa(
    mut diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
    comment_directives: Option<&[tombi_ast::TombiValueCommentDirective]>,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
    rule_name: &'static str,
) {
    let Some(comment_directive) = comment_directives else {
        return;
    };

    if common_rules
        .and_then(|rules| rules.unused_noqa.as_ref())
        .and_then(|rules| rules.disabled)
        .unwrap_or(false)
    {
        return;
    }

    for tombi_ast::TombiValueCommentDirective {
        content,
        content_range,
        ..
    } in comment_directive
    {
        let Ok(root) =
            tombi_parser::parse(content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION).try_into_root()
        else {
            continue;
        };

        let Ok(document_tree) = root.try_into_document_tree(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
        else {
            continue;
        };

        if let Some((key, value)) =
            dig_keys(&document_tree, &["lint", "rules", rule_name, "disabled"])
        {
            let range = key.range() + value.range();
            let range = tombi_text::Range::new(
                content_range.start + RelativePosition::from(range.start),
                content_range.start + RelativePosition::from(range.end),
            );
            crate::Diagnostic {
                kind: Box::new(crate::DiagnosticKind::UnusedNoqa { rule_name }),
                range,
            }
            .push_diagnostic_with_level(SeverityLevel::Warn, &mut diagnostics);
            return;
        }
    }
}
