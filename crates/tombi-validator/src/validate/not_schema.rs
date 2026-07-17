use tombi_ast::TombiValueCommentDirective;
use tombi_document_tree::ValueImpl;
use tombi_schema_store::{CurrentSchema, SchemaContext};
use tombi_severity_level::SeverityLevelDefaultError;

use crate::{
    Validate,
    validate::{handle_unused_noqa, is_assertion_success},
};

pub async fn validate_not<'a, T>(
    value: &T,
    accessors: &[tombi_schema_store::Accessor],
    not_schema: &tombi_schema_store::NotSchema,
    current_schema: &CurrentSchema<'a>,
    schema_context: &SchemaContext<'a>,
    comment_directives: Option<impl Iterator<Item = &'a TombiValueCommentDirective> + 'a>,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) -> Result<crate::EvaluatedLocations, crate::Error>
where
    T: Validate + ValueImpl + Sync + Send,
{
    let matches_not_schema = match tombi_schema_store::resolve_schema_item(
        &not_schema.schema,
        current_schema.schema_uri.clone(),
        current_schema.definitions.clone(),
        schema_context.store,
    )
    .await
    {
        Ok(Some(current_schema)) => is_assertion_success(
            &value
                .validate(accessors, Some(&current_schema), schema_context)
                .await,
        ),
        Ok(None) => false,
        Err(err) => {
            return Err(vec![err.to_diagnostic(value.range())].into());
        }
    };

    if matches_not_schema {
        let mut diagnostics = Vec::with_capacity(1);
        crate::Diagnostic {
            kind: Box::new(crate::DiagnosticKind::NotSchemaMatch),
            range: value.range(),
        }
        .push_diagnostic_with_level(
            common_rules
                .and_then(|rules| rules.not_schema_match.as_ref())
                .map(SeverityLevelDefaultError::from)
                .unwrap_or_default(),
            &mut diagnostics,
        );

        return Err(diagnostics.into());
    } else if common_rules
        .and_then(|rules| rules.not_schema_match.as_ref())
        .and_then(|rules| rules.disabled)
        == Some(true)
    {
        let mut diagnostics = Vec::with_capacity(1);
        handle_unused_noqa(
            &mut diagnostics,
            comment_directives,
            common_rules,
            "not-schema-match",
        );

        if !diagnostics.is_empty() {
            return Err(diagnostics.into());
        }
    }

    Ok(crate::EvaluatedLocations::new())
}
