use std::{borrow::Cow, str::FromStr, sync::Arc};

use futures::future::join_all;
use indexmap::IndexSet;
use tombi_future::{BoxFuture, Boxable};
use tombi_json::StringNode;
use tombi_x_keyword::StringFormat;

use super::{
    referable_schema::CurrentSchema, AllOfSchema, AnyOfSchema, ArraySchema, BooleanSchema,
    FindSchemaCandidates, FloatSchema, IntegerSchema, LocalDateSchema, LocalDateTimeSchema,
    LocalTimeSchema, OffsetDateTimeSchema, OneOfSchema, SchemaUri, StringSchema, TableSchema,
};
use crate::{Accessor, Referable, SchemaDefinitions, SchemaStore};

#[derive(Debug, Clone)]
pub enum ValueSchema {
    Null,
    Boolean(BooleanSchema),
    Integer(IntegerSchema),
    Float(FloatSchema),
    String(StringSchema),
    LocalDate(LocalDateSchema),
    LocalDateTime(LocalDateTimeSchema),
    LocalTime(LocalTimeSchema),
    OffsetDateTime(OffsetDateTimeSchema),
    Array(ArraySchema),
    Table(TableSchema),
    OneOf(OneOfSchema),
    AnyOf(AnyOfSchema),
    AllOf(AllOfSchema),
}

impl ValueSchema {
    pub fn new(
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<Self> {
        match object.get("type") {
            Some(tombi_json::ValueNode::String(type_str)) => {
                return Self::new_single(type_str.value.as_str(), object, string_formats)
            }
            Some(tombi_json::ValueNode::Array(types)) => {
                let schemas = types
                    .items
                    .iter()
                    .filter_map(|type_value| {
                        if let tombi_json::ValueNode::String(type_str) = type_value {
                            Self::new_single(type_str.value.as_str(), object, string_formats)
                        } else {
                            None
                        }
                    })
                    .map(|value_schema| Referable::Resolved {
                        schema_uri: None,
                        value: value_schema,
                    })
                    .collect();

                return Some(Self::OneOf(OneOfSchema {
                    schemas: Arc::new(tokio::sync::RwLock::new(schemas)),
                    ..Default::default()
                }));
            }
            _ => {}
        }

        if object.get("oneOf").is_some() {
            return Some(ValueSchema::OneOf(OneOfSchema::new(object, string_formats)));
        }
        if object.get("anyOf").is_some() {
            return Some(ValueSchema::AnyOf(AnyOfSchema::new(object, string_formats)));
        }
        if object.get("allOf").is_some() {
            return Some(ValueSchema::AllOf(AllOfSchema::new(object, string_formats)));
        }
        if let Some(tombi_json::ValueNode::Array(enum_values)) = object.get("enum") {
            return Self::new_enum_value(object, enum_values, string_formats);
        }

        None
    }

    fn new_single(
        type_str: &str,
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<Self> {
        match type_str {
            "null" => Some(ValueSchema::Null),
            "boolean" => Some(ValueSchema::Boolean(BooleanSchema::new(object))),
            "integer" => Some(ValueSchema::Integer(IntegerSchema::new(object))),
            "number" => Some(ValueSchema::Float(FloatSchema::new(object))),
            "string" => {
                let string_format = if let Some(tombi_json::ValueNode::String(StringNode {
                    value: format_str,
                    ..
                })) = object.get("format")
                {
                    // See: https://json-schema.org/understanding-json-schema/reference/type#built-in-formats
                    match format_str.as_str() {
                        "date-time" => {
                            return Some(ValueSchema::OffsetDateTime(OffsetDateTimeSchema::new(
                                object,
                            )))
                        }
                        "date-time-local" | "partial-date-time" => {
                            // NOTE: It's defined in OpenAPI.
                            //       date-time-local: see [OpenAPI Format Registry](https://spec.openapis.org/registry/format/date-time-local.html).
                            //       partial-date-time: used [schemars](https://github.com/GREsau/schemars).
                            return Some(ValueSchema::LocalDateTime(LocalDateTimeSchema::new(
                                object,
                            )));
                        }
                        "date" => {
                            return Some(ValueSchema::LocalDate(LocalDateSchema::new(object)))
                        }
                        "time-local" | "partial-time" => {
                            // NOTE: It's defined in OpenAPI.
                            //       time-local: see [OpenAPI Format Registry](https://spec.openapis.org/registry/format/time-local.html).
                            //       partial-time: used [schemars](https://github.com/GREsau/schemars).
                            return Some(ValueSchema::LocalTime(LocalTimeSchema::new(object)));
                        }
                        _ => string_formats.and_then(|string_formats| {
                            if let Ok(string_format) = StringFormat::from_str(format_str.as_str()) {
                                if string_formats.contains(&string_format) {
                                    return Some(string_format);
                                }
                            }
                            None
                        }),
                    }
                } else {
                    None
                };

                Some(ValueSchema::String(StringSchema::new(
                    object,
                    string_format,
                )))
            }
            "array" => Some(ValueSchema::Array(ArraySchema::new(object, string_formats))),
            "object" => Some(ValueSchema::Table(TableSchema::new(object, string_formats))),
            _ => None,
        }
    }

    fn new_enum_value(
        object: &tombi_json::ObjectNode,
        enum_values: &tombi_json::ArrayNode,
        string_formats: Option<&[StringFormat]>,
    ) -> Option<Self> {
        let mut enum_types = IndexSet::new();
        for enum_value in &enum_values.items {
            match enum_value {
                tombi_json::ValueNode::Null(_) => {
                    enum_types.insert("null");
                }
                tombi_json::ValueNode::Bool(_) => {
                    enum_types.insert("boolean");
                }
                tombi_json::ValueNode::Number(_) => {
                    enum_types.insert("number");
                }
                tombi_json::ValueNode::String(_) => {
                    enum_types.insert("string");
                }
                tombi_json::ValueNode::Array(_) | tombi_json::ValueNode::Object(_) => {
                    continue;
                }
            }
        }

        match enum_types.len() {
            0 => None,
            1 => {
                let value_type = enum_types.into_iter().next().unwrap();
                Self::new_single(value_type, object, string_formats)
            }
            _ => {
                let mut schemas = Vec::with_capacity(enum_types.len());
                for value_type in enum_types {
                    if let Some(schema) = Self::new_single(value_type, object, string_formats) {
                        schemas.push(Referable::Resolved {
                            schema_uri: None,
                            value: schema,
                        });
                    }
                }
                let title = object
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let description = object
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Some(Self::OneOf(OneOfSchema {
                    title,
                    description,
                    range: object.range,
                    schemas: Arc::new(tokio::sync::RwLock::new(schemas)),
                    default: object.get("default").cloned().map(|v| v.into()),
                    examples: object
                        .get("examples")
                        .and_then(|v| v.as_array())
                        .map(|array| array.items.iter().map(|v| v.into()).collect()),
                    deprecated: object.get("deprecated").and_then(|v| v.as_bool()),
                }))
            }
        }
    }

    pub async fn value_type(&self) -> crate::ValueType {
        match self {
            Self::Null => crate::ValueType::Null,
            Self::Boolean(boolean) => boolean.value_type(),
            Self::Integer(integer) => integer.value_type(),
            Self::Float(float) => float.value_type(),
            Self::String(string) => string.value_type(),
            Self::LocalDate(local_date) => local_date.value_type(),
            Self::LocalDateTime(local_date_time) => local_date_time.value_type(),
            Self::LocalTime(local_time) => local_time.value_type(),
            Self::OffsetDateTime(offset_date_time) => offset_date_time.value_type(),
            Self::Array(array) => array.value_type(),
            Self::Table(table) => table.value_type(),
            Self::OneOf(one_of) => one_of.value_type().await,
            Self::AnyOf(any_of) => any_of.value_type().await,
            Self::AllOf(all_of) => all_of.value_type().await,
        }
    }

    pub async fn deprecated(&self) -> Option<bool> {
        match self {
            Self::Null => None,
            Self::Boolean(boolean) => boolean.deprecated,
            Self::Integer(integer) => integer.deprecated,
            Self::Float(float) => float.deprecated,
            Self::String(string) => string.deprecated,
            Self::LocalDate(local_date) => local_date.deprecated,
            Self::LocalDateTime(local_date_time) => local_date_time.deprecated,
            Self::LocalTime(local_time) => local_time.deprecated,
            Self::OffsetDateTime(offset_date_time) => offset_date_time.deprecated,
            Self::Array(array) => array.deprecated,
            Self::Table(table) => table.deprecated,
            Self::OneOf(OneOfSchema {
                deprecated,
                schemas,
                ..
            })
            | Self::AnyOf(AnyOfSchema {
                deprecated,
                schemas,
                ..
            })
            | Self::AllOf(AllOfSchema {
                deprecated,
                schemas,
                ..
            }) => {
                if let Some(true) = deprecated {
                    Some(true)
                } else {
                    let mut has_deprecated = false;
                    for schema in schemas.read().await.iter() {
                        if schema.value_type().await == crate::ValueType::Null {
                            continue;
                        }
                        if schema.deprecated().await != Some(true) {
                            return None;
                        } else {
                            has_deprecated = true;
                        }
                    }
                    if has_deprecated {
                        Some(true)
                    } else {
                        None
                    }
                }
            }
        }
    }

    pub(crate) fn set_deprecated(&mut self, deprecated: bool) {
        match self {
            Self::Null => {}
            Self::Boolean(boolean) => boolean.deprecated = Some(deprecated),
            Self::Integer(integer) => integer.deprecated = Some(deprecated),
            Self::Float(float) => float.deprecated = Some(deprecated),
            Self::String(string) => string.deprecated = Some(deprecated),
            Self::LocalDate(local_date) => local_date.deprecated = Some(deprecated),
            Self::LocalDateTime(local_date_time) => local_date_time.deprecated = Some(deprecated),
            Self::LocalTime(local_time) => local_time.deprecated = Some(deprecated),
            Self::OffsetDateTime(offset_date_time) => {
                offset_date_time.deprecated = Some(deprecated)
            }
            Self::Array(array) => array.deprecated = Some(deprecated),
            Self::Table(table) => table.deprecated = Some(deprecated),
            Self::OneOf(one_of) => one_of.deprecated = Some(deprecated),
            Self::AnyOf(any_of) => any_of.deprecated = Some(deprecated),
            Self::AllOf(all_of) => all_of.deprecated = Some(deprecated),
        }
    }

    pub fn title(&self) -> Option<&str> {
        match self {
            ValueSchema::Null => None,
            ValueSchema::Boolean(schema) => schema.title.as_deref(),
            ValueSchema::Integer(schema) => schema.title.as_deref(),
            ValueSchema::Float(schema) => schema.title.as_deref(),
            ValueSchema::String(schema) => schema.title.as_deref(),
            ValueSchema::LocalDate(schema) => schema.title.as_deref(),
            ValueSchema::LocalDateTime(schema) => schema.title.as_deref(),
            ValueSchema::LocalTime(schema) => schema.title.as_deref(),
            ValueSchema::OffsetDateTime(schema) => schema.title.as_deref(),
            ValueSchema::Array(schema) => schema.title.as_deref(),
            ValueSchema::Table(schema) => schema.title.as_deref(),
            ValueSchema::OneOf(schema) => schema.title.as_deref(),
            ValueSchema::AnyOf(schema) => schema.title.as_deref(),
            ValueSchema::AllOf(schema) => schema.title.as_deref(),
        }
    }

    pub fn set_title(&mut self, title: Option<String>) {
        match self {
            ValueSchema::Null => {}
            ValueSchema::Boolean(schema) => schema.title = title,
            ValueSchema::Integer(schema) => schema.title = title,
            ValueSchema::Float(schema) => schema.title = title,
            ValueSchema::String(schema) => schema.title = title,
            ValueSchema::LocalDate(schema) => schema.title = title,
            ValueSchema::LocalDateTime(schema) => schema.title = title,
            ValueSchema::LocalTime(schema) => schema.title = title,
            ValueSchema::OffsetDateTime(schema) => schema.title = title,
            ValueSchema::Array(schema) => schema.title = title,
            ValueSchema::Table(schema) => schema.title = title,
            ValueSchema::OneOf(schema) => schema.title = title,
            ValueSchema::AnyOf(schema) => schema.title = title,
            ValueSchema::AllOf(schema) => schema.title = title,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            ValueSchema::Null => None,
            ValueSchema::Boolean(schema) => schema.description.as_deref(),
            ValueSchema::Integer(schema) => schema.description.as_deref(),
            ValueSchema::Float(schema) => schema.description.as_deref(),
            ValueSchema::String(schema) => schema.description.as_deref(),
            ValueSchema::LocalDate(schema) => schema.description.as_deref(),
            ValueSchema::LocalDateTime(schema) => schema.description.as_deref(),
            ValueSchema::LocalTime(schema) => schema.description.as_deref(),
            ValueSchema::OffsetDateTime(schema) => schema.description.as_deref(),
            ValueSchema::Array(schema) => schema.description.as_deref(),
            ValueSchema::Table(schema) => schema.description.as_deref(),
            ValueSchema::OneOf(schema) => schema.description.as_deref(),
            ValueSchema::AnyOf(schema) => schema.description.as_deref(),
            ValueSchema::AllOf(schema) => schema.description.as_deref(),
        }
    }

    pub fn set_description(&mut self, description: Option<String>) {
        match self {
            ValueSchema::Null => {}
            ValueSchema::Boolean(schema) => schema.description = description,
            ValueSchema::Integer(schema) => schema.description = description,
            ValueSchema::Float(schema) => schema.description = description,
            ValueSchema::String(schema) => schema.description = description,
            ValueSchema::LocalDate(schema) => schema.description = description,
            ValueSchema::LocalDateTime(schema) => schema.description = description,
            ValueSchema::LocalTime(schema) => schema.description = description,
            ValueSchema::OffsetDateTime(schema) => schema.description = description,
            ValueSchema::Array(schema) => schema.description = description,
            ValueSchema::Table(schema) => schema.description = description,
            ValueSchema::OneOf(schema) => schema.description = description,
            ValueSchema::AnyOf(schema) => schema.description = description,
            ValueSchema::AllOf(schema) => schema.description = description,
        }
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            ValueSchema::Null => tombi_text::Range::default(),
            ValueSchema::Boolean(schema) => schema.range,
            ValueSchema::Integer(schema) => schema.range,
            ValueSchema::Float(schema) => schema.range,
            ValueSchema::String(schema) => schema.range,
            ValueSchema::LocalDate(schema) => schema.range,
            ValueSchema::LocalDateTime(schema) => schema.range,
            ValueSchema::LocalTime(schema) => schema.range,
            ValueSchema::OffsetDateTime(schema) => schema.range,
            ValueSchema::Array(schema) => schema.range,
            ValueSchema::Table(schema) => schema.range,
            ValueSchema::OneOf(schema) => schema.range,
            ValueSchema::AnyOf(schema) => schema.range,
            ValueSchema::AllOf(schema) => schema.range,
        }
    }

    pub fn match_flattened_schemas<'a: 'b, 'b, T: Fn(&ValueSchema) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, Vec<ValueSchema>> {
        async move {
            let mut matched_schemas = Vec::new();
            match self {
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                    for referable_schema in schemas.write().await.iter_mut() {
                        if let Ok(Some(current_schema)) = referable_schema
                            .resolve(
                                Cow::Borrowed(schema_uri),
                                Cow::Borrowed(definitions),
                                schema_store,
                            )
                            .await
                        {
                            matched_schemas.extend(
                                current_schema
                                    .value_schema
                                    .match_flattened_schemas(
                                        condition,
                                        &current_schema.schema_uri,
                                        &current_schema.definitions,
                                        schema_store,
                                    )
                                    .await,
                            )
                        }
                    }
                }
                _ => {
                    if condition(self) {
                        matched_schemas.push(self.clone());
                    }
                }
            };

            matched_schemas
        }
        .boxed()
    }

    pub fn is_match<'a, 'b, T: Fn(&ValueSchema) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, bool>
    where
        'a: 'b,
    {
        async move {
            match self {
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. }) => join_all(
                    schemas
                        .write()
                        .await
                        .iter_mut()
                        .map(|referable_schema| async {
                            if let Ok(Some(CurrentSchema {
                                value_schema,
                                schema_uri,
                                definitions,
                            })) = referable_schema
                                .resolve(
                                    Cow::Borrowed(schema_uri),
                                    Cow::Borrowed(definitions),
                                    schema_store,
                                )
                                .await
                            {
                                value_schema
                                    .is_match(condition, &schema_uri, &definitions, schema_store)
                                    .await
                            } else {
                                false
                            }
                        }),
                )
                .await
                .into_iter()
                .any(|is_matched| is_matched),
                ValueSchema::AllOf(AllOfSchema { schemas, .. }) => join_all(
                    schemas
                        .write()
                        .await
                        .iter_mut()
                        .map(|referable_schema| async {
                            if let Ok(Some(CurrentSchema {
                                value_schema,
                                schema_uri,
                                definitions,
                            })) = referable_schema
                                .resolve(
                                    Cow::Borrowed(schema_uri),
                                    Cow::Borrowed(definitions),
                                    schema_store,
                                )
                                .await
                            {
                                value_schema
                                    .is_match(condition, &schema_uri, &definitions, schema_store)
                                    .await
                            } else {
                                false
                            }
                        }),
                )
                .await
                .into_iter()
                .all(|is_matched| is_matched),
                _ => condition(self),
            }
        }
        .boxed()
    }
}

impl FindSchemaCandidates for ValueSchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            match self {
                Self::OneOf(OneOfSchema {
                    title,
                    description,
                    schemas,
                    ..
                })
                | Self::AnyOf(AnyOfSchema {
                    title,
                    description,
                    schemas,
                    ..
                })
                | Self::AllOf(AllOfSchema {
                    title,
                    description,
                    schemas,
                    ..
                }) => {
                    let mut candidates = Vec::new();
                    let mut errors = Vec::new();

                    for referable_schema in schemas.write().await.iter_mut() {
                        let Ok(Some(current_schema)) = referable_schema
                            .resolve(
                                Cow::Borrowed(schema_uri),
                                Cow::Borrowed(definitions),
                                schema_store,
                            )
                            .await
                        else {
                            continue;
                        };

                        let (mut schema_candidates, schema_errors) = current_schema
                            .value_schema
                            .find_schema_candidates(
                                accessors,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                schema_store,
                            )
                            .await;

                        for schema_candidate in &mut schema_candidates {
                            if title.is_some() || description.is_some() {
                                schema_candidate.set_title(title.clone());
                                schema_candidate.set_description(description.clone());
                            }
                        }

                        candidates.extend(schema_candidates);
                        errors.extend(schema_errors);
                    }

                    (candidates, errors)
                }
                ValueSchema::Null => (Vec::with_capacity(0), Vec::with_capacity(0)),
                _ => (vec![self.clone()], Vec::with_capacity(0)),
            }
        }
        .boxed()
    }
}
