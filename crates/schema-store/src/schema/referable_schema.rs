use std::borrow::Cow;

use futures::future::BoxFuture;

use super::{AllOfSchema, AnyOfSchema, OneOfSchema, SchemaDefinitions, SchemaUrl, ValueSchema};

#[derive(Debug, Clone, PartialEq)]
pub enum Referable<T> {
    Resolved(T),
    Ref {
        reference: String,
        title: Option<String>,
        description: Option<String>,
    },
}

pub struct CurrentSchema<'a> {
    pub schema_url: Cow<'a, SchemaUrl>,
    pub value_schema: &'a ValueSchema,
    pub definitions: Cow<'a, SchemaDefinitions>,
}

impl<T> Referable<T> {
    pub fn resolved(&self) -> Option<&T> {
        match self {
            Self::Resolved(t) => Some(t),
            Self::Ref { .. } => None,
        }
    }
}

impl Referable<ValueSchema> {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        if let Some(serde_json::Value::String(ref_string)) = object.get("$ref") {
            return Some(Referable::Ref {
                reference: ref_string.clone(),
                title: object
                    .get("title")
                    .and_then(|title| title.as_str().map(|s| s.to_string())),
                description: object
                    .get("description")
                    .and_then(|description| description.as_str().map(|s| s.to_string())),
            });
        }

        ValueSchema::new(object).map(Referable::Resolved)
    }

    pub async fn value_type(&self) -> crate::ValueType {
        match self {
            Referable::Resolved(schema) => schema.value_type().await,
            Referable::Ref { .. } => unreachable!("unreachable ref value_tyle."),
        }
    }

    pub fn resolve<'a: 'b, 'b>(
        &'a mut self,
        schema_url: Cow<'a, SchemaUrl>,
        definitions: Cow<'a, SchemaDefinitions>,
        schema_store: &'a crate::SchemaStore,
    ) -> BoxFuture<'b, Result<CurrentSchema<'a>, crate::Error>> {
        Box::pin(async move {
            match self {
                Referable::Ref {
                    reference,
                    title,
                    description,
                } => {
                    if let Some(definition_schema) = definitions.read().await.get(reference) {
                        let mut referable_schema = definition_schema.to_owned();
                        if let Referable::Resolved(ref mut schema) = &mut referable_schema {
                            if title.is_some() || description.is_some() {
                                schema.set_title(title.to_owned());
                                schema.set_description(description.to_owned());
                            }
                        }

                        *self = referable_schema;
                    } else if is_online_url(reference) {
                        let schema_url = SchemaUrl::parse(reference)?;
                        let document_schema =
                            schema_store.try_load_document_schema(&schema_url).await?;

                        if let Some(value_schema) = document_schema.value_schema {
                            *self = Referable::Resolved(value_schema);
                            return self
                                .resolve(
                                    Cow::Owned(schema_url),
                                    Cow::Owned(document_schema.definitions),
                                    schema_store,
                                )
                                .await;
                        } else {
                            return Err(crate::Error::InvalidJsonSchemaReference {
                                reference: reference.to_owned(),
                            });
                        }
                    } else {
                        return Err(crate::Error::UnsupportedReference {
                            reference: reference.to_owned(),
                        });
                    }

                    self.resolve(schema_url, definitions, schema_store).await
                }
                Referable::Resolved(value_schema) => {
                    match value_schema {
                        ValueSchema::OneOf(OneOfSchema { schemas, .. })
                        | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                        | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                            for schema in schemas.write().await.iter_mut() {
                                schema
                                    .resolve(schema_url.clone(), definitions.clone(), schema_store)
                                    .await?;
                            }
                        }
                        _ => {}
                    }

                    Ok(CurrentSchema {
                        schema_url,
                        value_schema,
                        definitions,
                    })
                }
            }
        })
    }
}

pub fn is_online_url(reference: &str) -> bool {
    reference.starts_with("https://") || reference.starts_with("http://")
}
