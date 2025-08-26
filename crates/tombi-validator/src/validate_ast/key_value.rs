use std::borrow::Cow;

use tombi_comment_directive::CommentContext;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{Accessor, CurrentSchema, SchemaAccessor, ValueSchema};

use crate::validate_ast::Validate;

impl Validate for tombi_ast::KeyValue {
    fn validate<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
        comment_context: &'a CommentContext<'a>,
    ) -> BoxFuture<'b, Result<(), Vec<tombi_diagnostic::Diagnostic>>> {
        async move {
            let mut diagnostics = Vec::new();

            // Get keys and validate them
            if let Some(keys) = self.keys() {
                // Navigate through keys and get the schema
                let (updated_accessors, new_schema, mut key_diagnostics) =
                    navigate_keys_and_get_schema(&keys, accessors, current_schema, schema_context)
                        .await;
                diagnostics.append(&mut key_diagnostics);

                // Validate the value with updated schema and accessors
                if let Some(value) = self.value() {
                    if let Err(mut errs) = value
                        .validate(
                            &updated_accessors,
                            new_schema.as_ref(),
                            schema_context,
                            comment_context,
                        )
                        .await
                    {
                        diagnostics.append(&mut errs);
                    }
                }
            } else {
                if let Some(value) = self.value() {
                    if let Err(mut errs) = value
                        .validate(accessors, current_schema, schema_context, comment_context)
                        .await
                    {
                        diagnostics.append(&mut errs);
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

/// Navigate through keys and get the schema for the value
async fn navigate_keys_and_get_schema<'a>(
    keys: &'a tombi_ast::Keys,
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> (
    Vec<Accessor>,
    Option<CurrentSchema<'a>>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    let mut diagnostics = Vec::new();
    let mut updated_schema_accessors = accessors.to_vec();

    // Build accessor lists with keys
    for key in keys.keys() {
        if let Ok(key_text) = key.try_to_raw_text(schema_context.toml_version) {
            updated_schema_accessors.push(Accessor::Key(key_text));
        }
    }

    // Try to get sub-schema if configured
    if let Some(sub_schema_result) = schema_context
        .get_subschema(&updated_schema_accessors, current_schema)
        .await
    {
        match sub_schema_result {
            Ok(doc_schema) => {
                if let Some(value_schema) = doc_schema.value_schema {
                    return (
                        updated_schema_accessors,
                        Some(CurrentSchema {
                            value_schema: Cow::Owned(value_schema),
                            schema_uri: Cow::Owned(doc_schema.schema_uri),
                            definitions: Cow::Owned(doc_schema.definitions),
                        }),
                        diagnostics,
                    );
                }
            }
            Err(err) => {
                diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                    err.to_string(),
                    "schema-error",
                    keys.range(),
                ));
                return (updated_schema_accessors, None, diagnostics);
            }
        }
    }

    // Navigate through current schema
    let new_schema = if let Some(current) = current_schema {
        navigate_through_schema(keys, current, schema_context).await
    } else {
        None
    };

    (updated_schema_accessors, new_schema, diagnostics)
}

/// Navigate through a schema to find the schema for keys
async fn navigate_through_schema<'a>(
    keys: &'a tombi_ast::Keys,
    current: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
) -> Option<CurrentSchema<'a>> {
    // Get the first key for navigation
    let first_key = keys.keys().next()?;
    let key_text = first_key
        .try_to_raw_text(schema_context.toml_version)
        .ok()?;
    let key_accessor = SchemaAccessor::Key(key_text);

    match &*current.value_schema {
        ValueSchema::Table(table_schema) => {
            let properties = table_schema.properties.blocking_read();
            properties.get(&key_accessor).and_then(|property_schema| {
                resolve_property_schema_owned(property_schema.clone(), current)
            })
        }
        ValueSchema::OneOf(one_of) => {
            // For OneOf, we need to check all schemas
            let schemas = one_of.schemas.blocking_read();
            for schema_ref in schemas.iter() {
                if let tombi_schema_store::Referable::Resolved { value, .. } = schema_ref {
                    if let ValueSchema::Table(table_schema) = value {
                        let properties = table_schema.properties.blocking_read();
                        if let Some(property_schema) = properties.get(&key_accessor) {
                            return resolve_property_schema_owned(property_schema.clone(), current);
                        }
                    }
                }
            }
            None
        }
        ValueSchema::AnyOf(any_of) => {
            // For AnyOf, check all schemas
            let schemas = any_of.schemas.blocking_read();
            for schema_ref in schemas.iter() {
                if let tombi_schema_store::Referable::Resolved { value, .. } = schema_ref {
                    if let ValueSchema::Table(table_schema) = value {
                        let properties = table_schema.properties.blocking_read();
                        if let Some(property_schema) = properties.get(&key_accessor) {
                            return resolve_property_schema_owned(property_schema.clone(), current);
                        }
                    }
                }
            }
            None
        }
        ValueSchema::AllOf(all_of) => {
            // For AllOf, check all schemas and merge
            let schemas = all_of.schemas.blocking_read();
            for schema_ref in schemas.iter() {
                if let tombi_schema_store::Referable::Resolved { value, .. } = schema_ref {
                    if let ValueSchema::Table(table_schema) = value {
                        let properties = table_schema.properties.blocking_read();
                        if let Some(property_schema) = properties.get(&key_accessor) {
                            return resolve_property_schema_owned(property_schema.clone(), current);
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Resolve a property schema to CurrentSchema (takes ownership)
fn resolve_property_schema_owned<'a>(
    property_schema: tombi_schema_store::PropertySchema,
    current: &'a CurrentSchema<'a>,
) -> Option<CurrentSchema<'a>> {
    match &property_schema.property_schema {
        tombi_schema_store::Referable::Resolved { value, schema_uri } => Some(CurrentSchema {
            value_schema: Cow::Owned(value.clone()),
            schema_uri: match schema_uri {
                Some(uri) => Cow::Owned(uri.clone()),
                None => match &current.schema_uri {
                    Cow::Owned(uri) => Cow::Owned(uri.clone()),
                    Cow::Borrowed(uri) => Cow::Borrowed(*uri),
                },
            },
            definitions: match &current.definitions {
                Cow::Owned(defs) => Cow::Owned(defs.clone()),
                Cow::Borrowed(defs) => Cow::Borrowed(*defs),
            },
        }),
        tombi_schema_store::Referable::Ref { .. } => {
            // TODO: Resolve reference
            None
        }
    }
}
