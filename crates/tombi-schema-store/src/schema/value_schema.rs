use std::{borrow::Cow, str::FromStr, sync::Arc};

use futures::future::join_all;
use indexmap::IndexSet;
use tombi_future::{BoxFuture, Boxable};
use tombi_json::StringNode;
use tombi_x_keyword::{StringFormat, TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

use super::{
    AllOfSchema, AnyOfSchema, ArraySchema, BooleanSchema, FindSchemaCandidates, FloatSchema,
    IntegerSchema, LocalDateSchema, LocalDateTimeSchema, LocalTimeSchema, OffsetDateTimeSchema,
    OneOfSchema, SchemaUri, StringSchema, TableSchema, referable_schema::CurrentSchema,
};
use crate::{Accessor, Referable, SchemaDefinitions, SchemaStore, schema::not_schema::NotSchema};

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
                return Self::new_single(type_str.value.as_str(), object, string_formats);
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

        // Handle "const" keyword without explicit "type"
        // Infer the type from the const value itself
        if let Some(const_value) = object.get("const") {
            let inferred_type = match const_value {
                tombi_json::ValueNode::Null(_) => "null",
                tombi_json::ValueNode::Bool(_) => "boolean",
                tombi_json::ValueNode::Number(n) => {
                    if n.value.is_i64() {
                        "integer"
                    } else {
                        "number"
                    }
                }
                tombi_json::ValueNode::String(_) => "string",
                tombi_json::ValueNode::Array(_) => "array",
                tombi_json::ValueNode::Object(_) => "object",
            };
            return Self::new_single(inferred_type, object, string_formats);
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
                            )));
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
                            return Some(ValueSchema::LocalDate(LocalDateSchema::new(object)));
                        }
                        "time-local" | "partial-time" => {
                            // NOTE: It's defined in OpenAPI.
                            //       time-local: see [OpenAPI Format Registry](https://spec.openapis.org/registry/format/time-local.html).
                            //       partial-time: used [schemars](https://github.com/GREsau/schemars).
                            return Some(ValueSchema::LocalTime(LocalTimeSchema::new(object)));
                        }
                        _ => string_formats.and_then(|string_formats| {
                            if let Ok(string_format) = StringFormat::from_str(format_str.as_str())
                                && string_formats.contains(&string_format)
                            {
                                return Some(string_format);
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
                tombi_json::ValueNode::Number(n) => {
                    if n.value.is_i64() {
                        enum_types.insert("integer");
                    } else {
                        enum_types.insert("number");
                    }
                }
                tombi_json::ValueNode::String(_) => {
                    enum_types.insert("string");
                }
                tombi_json::ValueNode::Array(_) | tombi_json::ValueNode::Object(_) => {
                    continue;
                }
            }
        }

        if enum_types.contains("number") && enum_types.contains("integer") {
            enum_types.shift_remove("integer");
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
                    keys_order: object
                        .get(X_TOMBI_TABLE_KEYS_ORDER)
                        .and_then(|v| v.as_str().and_then(|s| TableKeysOrder::try_from(s).ok())),
                    not: NotSchema::new(object, string_formats),
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
                    if has_deprecated { Some(true) } else { None }
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

#[cfg(test)]
mod tests {
    use crate::ValueType;

    use super::*;
    use std::str::FromStr;

    fn parse_schema(json: &str) -> Option<ValueSchema> {
        let value_node = tombi_json::ValueNode::from_str(json).unwrap();
        let object = value_node.as_object().unwrap();
        ValueSchema::new(object, None)
    }

    #[test]
    fn test_const_string_creates_string_schema() {
        let schema = parse_schema(r#"{ "const": "dynamic" }"#);
        assert!(matches!(schema, Some(ValueSchema::String(_))));

        let Some(ValueSchema::String(s)) = schema else {
            panic!("schema is not a String schema");
        };
        assert_eq!(s.const_value, Some("dynamic".to_string()));
    }

    #[test]
    fn test_const_boolean_creates_boolean_schema() {
        let schema = parse_schema(r#"{ "const": true }"#);
        assert!(matches!(schema, Some(ValueSchema::Boolean(_))));

        let Some(ValueSchema::Boolean(b)) = schema else {
            panic!("schema is not a Boolean schema");
        };
        assert_eq!(b.const_value, Some(true));
    }

    #[test]
    fn test_const_integer_creates_integer_schema() {
        let schema = parse_schema(r#"{ "const": 42 }"#);
        assert!(matches!(schema, Some(ValueSchema::Integer(_))));

        let Some(ValueSchema::Integer(i)) = schema else {
            panic!("schema is not an Integer schema");
        };
        assert_eq!(i.const_value, Some(42));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_const_float_creates_float_schema() {
        let schema = parse_schema(r#"{ "const": 3.14 }"#);
        assert!(matches!(schema, Some(ValueSchema::Float(_))));

        if let Some(ValueSchema::Float(f)) = schema {
            assert_eq!(f.const_value, Some(3.14));
        }
    }

    #[test]
    fn test_const_null_creates_null_schema() {
        let schema = parse_schema(r#"{ "const": null }"#);
        assert!(matches!(schema, Some(ValueSchema::Null)));
    }

    #[tokio::test]
    async fn test_anyof_with_const_and_ref_style_integer() {
        // Simulates ruff schema: anyOf with integer and const "dynamic"
        let schema = parse_schema(
            r#"{
                "anyOf": [
                    { "type": "integer" },
                    { "const": "dynamic" }
                ]
            }"#,
        );
        assert!(matches!(schema, Some(ValueSchema::AnyOf(_))));

        let Some(ValueSchema::AnyOf(any_of)) = schema else {
            panic!("schema is not an AnyOf schema");
        };

        let schemas = any_of.schemas.read().await;
        assert!(matches!(
            schemas.get(0).unwrap().value_type().await,
            ValueType::Integer
        ));
        assert!(matches!(
            schemas.get(1).unwrap().value_type().await,
            ValueType::String
        ));
    }

    #[test]
    fn test_const_array_creates_array_schema() {
        let schema = parse_schema(r#"{ "const": [1, 2, 3] }"#);
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_const_empty_array_creates_array_schema() {
        let schema = parse_schema(r#"{ "const": [] }"#);
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_const_object_creates_table_schema() {
        let schema = parse_schema(r#"{ "const": {"key": "value"} }"#);
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_const_empty_object_creates_table_schema() {
        let schema = parse_schema(r#"{ "const": {} }"#);
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_const_nested_array_creates_array_schema() {
        let schema = parse_schema(r#"{ "const": [[1, 2], [3, 4]] }"#);
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_const_nested_object_creates_table_schema() {
        let schema = parse_schema(r#"{ "const": {"nested": {"key": "value"}} }"#);
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    fn parse_schema_with_formats(json: &str, formats: &[StringFormat]) -> Option<ValueSchema> {
        let value_node = tombi_json::ValueNode::from_str(json).unwrap();
        let object = value_node.as_object().unwrap();
        ValueSchema::new(object, Some(formats))
    }

    #[test]
    fn test_const_string_with_format_uses_new_single() {
        // When const has a format field, it should be processed through new_single
        // which handles date-time formats properly
        let schema = parse_schema(r#"{ "const": "2024-01-10", "format": "date" }"#);
        // Should create LocalDateSchema, not StringSchema
        assert!(matches!(schema, Some(ValueSchema::LocalDate(_))));
    }

    #[test]
    fn test_const_string_with_datetime_format() {
        let schema = parse_schema(r#"{ "const": "2024-01-10T12:00:00Z", "format": "date-time" }"#);
        assert!(matches!(schema, Some(ValueSchema::OffsetDateTime(_))));
    }

    #[test]
    fn test_const_string_with_custom_format_and_string_formats() {
        // Custom format (e.g., email) should be validated against string_formats
        let schema = parse_schema_with_formats(
            r#"{ "const": "test@example.com", "format": "email" }"#,
            &[StringFormat::Email],
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));

        if let Some(ValueSchema::String(s)) = schema {
            assert_eq!(s.format, Some(StringFormat::Email));
        }
    }

    #[test]
    fn test_enum_integer_creates_integer_schema() {
        // enum with integers should create IntegerSchema, not FloatSchema
        let schema = parse_schema(r#"{ "enum": [1, 2, 3] }"#);
        assert!(matches!(schema, Some(ValueSchema::Integer(_))));
    }

    #[test]
    fn test_enum_float_creates_float_schema() {
        // enum with floats should create FloatSchema
        let schema = parse_schema(r#"{ "enum": [1.5, 2.5, 3.5] }"#);
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_enum_mixed_int_float_removes_integer() {
        // When enum contains both integers and floats, integer should be removed
        // and only number should remain, resulting in FloatSchema
        // This tests the logic at lines 198-200: if enum_types contains both
        // "number" and "integer", "integer" is removed
        let schema = parse_schema(r#"{ "enum": [1, 2.5] }"#);
        // After removing integer, only number remains, so FloatSchema should be created
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[tokio::test]
    async fn test_enum_mixed_int_float_with_other_types() {
        // When enum contains integers, floats, and other types (e.g., string),
        // integer should be removed (lines 198-200), leaving number and string,
        // resulting in OneOf with FloatSchema and StringSchema (but not IntegerSchema)
        let schema = parse_schema(r#"{ "enum": [1, 2.5, "text"] }"#);

        let Some(ValueSchema::OneOf(one_of)) = schema else {
            panic!("schema is not a OneOf schema");
        };
        let schemars = one_of.schemas.read().await;
        assert!(matches!(
            schemars.get(0).unwrap().value_type().await,
            ValueType::Float
        ));
        assert!(matches!(
            schemars.get(1).unwrap().value_type().await,
            ValueType::String
        ));
    }
}
