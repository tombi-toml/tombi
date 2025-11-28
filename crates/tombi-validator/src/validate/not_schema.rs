use tombi_document_tree::ValueImpl;
use tombi_schema_store::{CurrentSchema, SchemaContext};
use tombi_severity_level::SeverityLevelDefaultError;

use crate::Validate;

pub async fn validate_not<'a, T>(
    value: &T,
    accessors: &[tombi_schema_store::Accessor],
    not_schema: &tombi_schema_store::NotSchema,
    current_schema: &CurrentSchema<'a>,
    schema_context: &SchemaContext<'a>,
    common_rules: Option<&tombi_comment_directive::value::CommonLintRules>,
) -> Result<(), crate::Error>
where
    T: Validate + ValueImpl + Sync + Send,
{
    if let Ok(Some(current_schema)) = not_schema
        .schema
        .write()
        .await
        .resolve(
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
        )
        .await
        .inspect_err(|err| tracing::warn!("{err}"))
    {
        if value
            .validate(accessors, Some(&current_schema), schema_context)
            .await
            .is_ok()
        {
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
        }
    }

    Ok(())
}
