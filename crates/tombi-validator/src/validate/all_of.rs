use std::fmt::Debug;

use tombi_comment_directive::value::CommonRules;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;

use super::Validate;

pub fn validate_all_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,

    _common_rules: Option<&'a CommonRules>,
) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>>
where
    T: Validate + Sync + Send + Debug,
{
    tracing::trace!("value = {:?}", value);
    tracing::trace!("all_of_schema = {:?}", all_of_schema);

    async move {
        let mut diagnostics = vec![];

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

            match value
                .validate(accessors, Some(&current_schema), schema_context)
                .await
            {
                Ok(()) => {}
                Err(mut schema_diagnostics) => diagnostics.append(&mut schema_diagnostics),
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
