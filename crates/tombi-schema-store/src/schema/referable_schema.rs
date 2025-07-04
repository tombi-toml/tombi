use std::borrow::Cow;

use crate::x_taplo::XTaplo;

use super::{AllOfSchema, AnyOfSchema, OneOfSchema, SchemaDefinitions, SchemaUrl, ValueSchema};

#[derive(Debug, Clone, PartialEq)]
pub enum Referable<T> {
    Resolved {
        schema_url: Option<SchemaUrl>,
        value: T,
    },
    Ref {
        reference: String,
        title: Option<String>,
        description: Option<String>,
        deprecated: Option<bool>,
    },
}

#[derive(Clone)]
pub struct CurrentSchema<'a> {
    pub value_schema: Cow<'a, ValueSchema>,
    pub schema_url: Cow<'a, SchemaUrl>,
    pub definitions: Cow<'a, SchemaDefinitions>,
}

impl std::fmt::Debug for CurrentSchema<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentSchema")
            .field("value_schema", &self.value_schema)
            .field("schema_url", &self.schema_url.to_string())
            .finish()
    }
}

impl<T> Referable<T> {
    pub fn resolved(&self) -> Option<&T> {
        match self {
            Self::Resolved { value, .. } => Some(value),
            Self::Ref { .. } => None,
        }
    }
}

impl Referable<ValueSchema> {
    pub fn new(object: &tombi_json::ObjectNode) -> Option<Self> {
        if let Some(x_taplo) = object.get("x-taplo") {
            if let Ok(x_taplo) = tombi_json::from_value_node::<XTaplo>(x_taplo.to_owned()) {
                if x_taplo.hidden == Some(true) {
                    return None;
                }
            }
        }
        if let Some(tombi_json::ValueNode::String(ref_string)) = object.get("$ref") {
            return Some(Referable::Ref {
                reference: ref_string.value.clone(),
                title: object
                    .get("title")
                    .and_then(|title| title.as_str().map(|s| s.to_string())),
                description: object
                    .get("description")
                    .and_then(|description| description.as_str().map(|s| s.to_string())),
                deprecated: object
                    .get("deprecated")
                    .and_then(|deprecated| deprecated.as_bool()),
            });
        }

        ValueSchema::new(object).map(|value_schema| Referable::Resolved {
            schema_url: None,
            value: value_schema,
        })
    }

    pub fn deprecated<'a: 'b, 'b>(&'a self) -> tombi_future::BoxFuture<'b, Option<bool>> {
        Box::pin(async move {
            match self {
                Referable::Resolved { value, .. } => value.deprecated().await,
                Referable::Ref { .. } => None,
            }
        })
    }

    pub async fn value_type(&self) -> crate::ValueType {
        match self {
            Referable::Resolved { value, .. } => value.value_type().await,
            Referable::Ref { .. } => unreachable!("unreachable ref value_tyle."),
        }
    }

    pub fn resolve<'a: 'b, 'b>(
        &'a mut self,
        schema_url: Cow<'a, SchemaUrl>,
        definitions: Cow<'a, SchemaDefinitions>,
        schema_store: &'a crate::SchemaStore,
    ) -> tombi_future::BoxFuture<'b, Result<Option<CurrentSchema<'a>>, crate::Error>> {
        Box::pin(async move {
            match self {
                Referable::Ref {
                    reference,
                    title,
                    description,
                    deprecated,
                } => {
                    if let Some(definition_schema) = definitions.read().await.get(reference) {
                        let mut referable_schema = definition_schema.to_owned();
                        if let Referable::Resolved {
                            value: ref mut value_schema,
                            ..
                        } = &mut referable_schema
                        {
                            if title.is_some() || description.is_some() {
                                value_schema.set_title(title.to_owned());
                                value_schema.set_description(description.to_owned());
                            }
                            if let Some(deprecated) = deprecated {
                                value_schema.set_deprecated(*deprecated);
                            }
                        }

                        *self = referable_schema;
                    } else if is_online_url(reference) {
                        let schema_url = SchemaUrl::parse(reference)?;

                        if let Some(mut document_schema) =
                            schema_store.try_get_document_schema(&schema_url).await?
                        {
                            if let Some(value_schema) = &mut document_schema.value_schema {
                                if title.is_some() || description.is_some() {
                                    value_schema.set_title(title.to_owned());
                                    value_schema.set_description(description.to_owned());
                                }
                                if let Some(deprecated) = deprecated {
                                    value_schema.set_deprecated(*deprecated);
                                }

                                *self = Referable::Resolved {
                                    schema_url: Some(document_schema.schema_url.clone()),
                                    value: value_schema.clone(),
                                };

                                return self
                                    .resolve(
                                        Cow::Owned(document_schema.schema_url),
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
                            return Ok(None);
                        }
                    } else {
                        return Err(crate::Error::UnsupportedReference {
                            reference: reference.to_owned(),
                        });
                    }

                    self.resolve(schema_url, definitions, schema_store).await
                }
                Referable::Resolved {
                    schema_url: reference_url,
                    value: value_schema,
                    ..
                } => {
                    let (schema_url, definitions) = {
                        match reference_url {
                            Some(reference_url) => {
                                if let Some(document_schema) =
                                    schema_store.try_get_document_schema(reference_url).await?
                                {
                                    (
                                        Cow::Owned(document_schema.schema_url),
                                        Cow::Owned(document_schema.definitions),
                                    )
                                } else {
                                    (schema_url, definitions)
                                }
                            }
                            None => (schema_url, definitions),
                        }
                    };

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

                    Ok(Some(CurrentSchema {
                        value_schema: Cow::Borrowed(value_schema),
                        schema_url,
                        definitions,
                    }))
                }
            }
        })
    }
}

pub fn is_online_url(reference: &str) -> bool {
    reference.starts_with("https://") || reference.starts_with("http://")
}
