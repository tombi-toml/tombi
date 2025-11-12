use std::fmt::Debug;

use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;

use crate::validate::push_deprecated;

use super::Validate;

pub fn validate_all_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<(), crate::Error>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    tracing::trace!("value = {:?}", value);
    tracing::trace!("all_of_schema = {:?}", all_of_schema);

    async move {
        let mut total_diagnostics = vec![];

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

            if let Err(crate::Error { diagnostics, .. }) = value
                .validate(accessors, Some(&current_schema), schema_context)
                .await
            {
                total_diagnostics.extend(diagnostics);
            }
        }

        if total_diagnostics.is_empty() && all_of_schema.deprecated == Some(true) {
            push_deprecated(&mut total_diagnostics, accessors, value, common_rules);
        }

        if total_diagnostics.is_empty() {
            Ok(())
        } else {
            Err(total_diagnostics.into())
        }
    }
    .boxed()
}
