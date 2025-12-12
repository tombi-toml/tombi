use std::fmt::Debug;

use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;

use crate::validate::{handle_deprecated, not_schema::validate_not};

use super::Validate;

pub fn validate_all_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [tombi_ast::TombiValueCommentDirective]>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<(), crate::Error>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    tracing::trace!("value = {:?}", value);
    tracing::trace!("all_of_schema = {:?}", all_of_schema);

    async move {
        let mut total_diagnostics = vec![];
        let mut total_score = 0;

        let mut schemas = all_of_schema.schemas.write().await;
        for referable_schema in schemas.iter_mut() {
            let current_schema = if let Ok(Some(current_schema)) = referable_schema
                .resolve(
                    current_schema.schema_uri.clone(),
                    current_schema.definitions.clone(),
                    schema_context.store,
                )
                .await
                .inspect_err(|err| tracing::warn!("{err}"))
            {
                current_schema
            } else {
                continue;
            };

            if let Err(crate::Error { diagnostics, score }) = value
                .validate(accessors, Some(&current_schema), schema_context)
                .await
            {
                total_diagnostics.extend(diagnostics);
                total_score += score;
            }
        }

        if total_diagnostics.is_empty() {
            handle_deprecated(
                &mut total_diagnostics,
                all_of_schema.deprecated,
                accessors,
                value,
                comment_directives,
                common_rules,
            );
        }

        if let Some(not_schema) = all_of_schema.not.as_ref() {
            if let Err(error) = validate_not(
                value,
                accessors,
                not_schema,
                current_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await
            {
                total_diagnostics.extend(error.diagnostics);
            }
        }

        if total_diagnostics.is_empty() {
            Ok(())
        } else {
            Err(crate::Error {
                score: total_score,
                diagnostics: total_diagnostics,
            })
        }
    }
    .boxed()
}
