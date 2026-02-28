use std::fmt::Debug;

use tombi_ast::TombiValueCommentDirective;
use tombi_comment_directive::value::CommonLintRules;
use tombi_document_tree::ValueImpl;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::CurrentSchema;

use super::Validate;
use crate::validate::not_schema::validate_not;
use crate::validate::{validate_deprecated, validate_resolved_schema};

pub fn validate_any_of<'a: 'b, 'b, T>(
    value: &'a T,
    accessors: &'a [tombi_schema_store::Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    comment_directives: Option<&'a [TombiValueCommentDirective]>,
    common_rules: Option<&'a CommonLintRules>,
) -> BoxFuture<'b, Result<(), crate::Error>>
where
    T: Validate + ValueImpl + Sync + Send + Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("any_of_schema = {:?}", any_of_schema);

    async move {
        if let Some(not_schema) = any_of_schema.not.as_ref() {
            validate_not(
                value,
                accessors,
                not_schema,
                current_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await?;
        }

        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &any_of_schema.schemas,
            current_schema.schema_uri.clone(),
            current_schema.definitions.clone(),
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await
        else {
            return Ok(());
        };

        let mut total_error = crate::Error::new();

        for resolved_schema in &resolved_schemas {
            let Some(result) = validate_resolved_schema(
                value,
                accessors,
                resolved_schema,
                schema_context,
                comment_directives,
                common_rules,
            )
            .await
            else {
                continue;
            };

            match result {
                Ok(()) => {
                    return validate_deprecated(
                        any_of_schema.deprecated,
                        accessors,
                        value,
                        comment_directives,
                        common_rules,
                    );
                }
                Err(error) => {
                    total_error.combine(error);
                }
            }
        }

        if total_error.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(total_error)
        }
    }
    .boxed()
}
