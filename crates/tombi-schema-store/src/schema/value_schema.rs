use std::{borrow::Cow, str::FromStr, sync::Arc};

use futures::future::join_all;
use tombi_future::{BoxFuture, Boxable};
use tombi_json::StringNode;
use tombi_x_keyword::{StringFormat, TableKeysOrder, X_TOMBI_TABLE_KEYS_ORDER};

use super::{
    AllOfSchema, AnchorCollector, AnyOfSchema, ArraySchema, BooleanSchema, DynamicAnchorCollector,
    FindSchemaCandidates, FloatSchema, IntegerSchema, LocalDateSchema, LocalDateTimeSchema,
    LocalTimeSchema, OffsetDateTimeSchema, OneOfSchema, SchemaUri, StringSchema, TableSchema,
};
use crate::{
    Accessor, Referable, SchemaDefinitions, SchemaStore,
    schema::{if_then_else_schema::IfThenElseSchema, not_schema::NotSchema},
};

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
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut AnchorCollector>,
        dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) -> Option<Self> {
        let mut anchor_collector = anchor_collector;
        let mut dynamic_anchor_collector = dynamic_anchor_collector;
        match object.get("type") {
            Some(tombi_json::ValueNode::String(type_str)) => {
                return Self::new_single(
                    type_str.value.as_str(),
                    object,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                );
            }
            Some(tombi_json::ValueNode::Array(types)) => {
                let schemas = types
                    .items
                    .iter()
                    .filter_map(|type_value| {
                        if let tombi_json::ValueNode::String(type_str) = type_value {
                            Self::new_single(
                                type_str.value.as_str(),
                                object,
                                string_formats,
                                dialect,
                                anchor_collector.as_deref_mut(),
                                dynamic_anchor_collector.as_deref_mut(),
                            )
                        } else {
                            None
                        }
                    })
                    .map(|value_schema| Referable::Resolved {
                        schema_uri: None,
                        value: Arc::new(value_schema),
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
            return Some(ValueSchema::OneOf(OneOfSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )));
        }
        if object.get("anyOf").is_some() {
            return Some(ValueSchema::AnyOf(AnyOfSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )));
        }
        if object.get("allOf").is_some() {
            return Some(ValueSchema::AllOf(AllOfSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )));
        }
        if let Some(tombi_json::ValueNode::Array(enum_values)) = object.get("enum") {
            return Self::new_enum_value(
                object,
                enum_values,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            );
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
            return Self::new_single(
                inferred_type,
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            );
        }

        // Infer type from type-specific keywords when "type" is not explicitly specified
        if let Some(inferred_type) = Self::infer_type_from_keywords(object, dialect) {
            return Self::new_single(
                inferred_type,
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            );
        }

        None
    }

    /// Infer the JSON Schema type from type-specific keywords.
    ///
    /// When "type" is not explicitly specified, we can infer the type from keywords
    /// that are only valid for specific types:
    /// - String: minLength, maxLength, pattern, format, contentEncoding,
    ///           contentMediaType, contentSchema (content annotations)
    /// - Number: minimum, maximum, exclusiveMinimum, exclusiveMaximum, multipleOf
    /// - Array: items, prefixItems, minItems, maxItems, uniqueItems
    /// - Object: properties, patternProperties, additionalProperties, required,
    ///           minProperties, maxProperties, propertyNames
    fn infer_type_from_keywords(
        object: &tombi_json::ObjectNode,
        dialect: Option<crate::JsonSchemaDialect>,
    ) -> Option<&'static str> {
        let dialect = dialect.unwrap_or_default();
        let supports_keyword = |keyword: &str| crate::supports_keyword(dialect, keyword);

        // String-specific keywords
        if (supports_keyword("minLength") && object.get("minLength").is_some())
            || (supports_keyword("maxLength") && object.get("maxLength").is_some())
            || (supports_keyword("pattern") && object.get("pattern").is_some())
            || (supports_keyword("format") && object.get("format").is_some())
            || (supports_keyword("contentEncoding") && object.get("contentEncoding").is_some())
            || (supports_keyword("contentMediaType") && object.get("contentMediaType").is_some())
            || (supports_keyword("contentSchema") && object.get("contentSchema").is_some())
        {
            return Some("string");
        }

        // Numeric-specific keywords (use "number" as the broader type)
        if (supports_keyword("minimum") && object.get("minimum").is_some())
            || (supports_keyword("maximum") && object.get("maximum").is_some())
            || (supports_keyword("exclusiveMinimum") && object.get("exclusiveMinimum").is_some())
            || (supports_keyword("exclusiveMaximum") && object.get("exclusiveMaximum").is_some())
            || (supports_keyword("multipleOf") && object.get("multipleOf").is_some())
        {
            return Some("number");
        }

        // Array-specific keywords
        if (supports_keyword("items") && object.get("items").is_some())
            || (supports_keyword("prefixItems") && object.get("prefixItems").is_some())
            || (supports_keyword("minItems") && object.get("minItems").is_some())
            || (supports_keyword("maxItems") && object.get("maxItems").is_some())
            || (supports_keyword("uniqueItems") && object.get("uniqueItems").is_some())
            || (supports_keyword("contains") && object.get("contains").is_some())
        {
            return Some("array");
        }

        // Object-specific keywords
        if (supports_keyword("properties") && object.get("properties").is_some())
            || (supports_keyword("patternProperties") && object.get("patternProperties").is_some())
            || (supports_keyword("additionalProperties")
                && object.get("additionalProperties").is_some())
            || (supports_keyword("required") && object.get("required").is_some())
            || (supports_keyword("minProperties") && object.get("minProperties").is_some())
            || (supports_keyword("maxProperties") && object.get("maxProperties").is_some())
            || (supports_keyword("propertyNames") && object.get("propertyNames").is_some())
            || (supports_keyword("dependencies") && object.get("dependencies").is_some())
        {
            return Some("object");
        }

        None
    }

    fn new_single(
        type_str: &str,
        object: &tombi_json::ObjectNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut AnchorCollector>,
        dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) -> Option<Self> {
        let mut anchor_collector = anchor_collector;
        let mut dynamic_anchor_collector = dynamic_anchor_collector;
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
            "array" => Some(ValueSchema::Array(ArraySchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ))),
            "object" => Some(ValueSchema::Table(TableSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ))),
            _ => None,
        }
    }

    fn new_enum_value(
        object: &tombi_json::ObjectNode,
        enum_values: &tombi_json::ArrayNode,
        string_formats: Option<&[StringFormat]>,
        dialect: Option<crate::JsonSchemaDialect>,
        anchor_collector: Option<&mut AnchorCollector>,
        dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
    ) -> Option<Self> {
        let mut anchor_collector = anchor_collector;
        let mut dynamic_anchor_collector = dynamic_anchor_collector;
        let mut enum_types = tombi_hashmap::IndexSet::new();
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
                Self::new_single(
                    value_type,
                    object,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                )
            }
            _ => {
                let mut schemas = Vec::with_capacity(enum_types.len());
                for value_type in enum_types {
                    if let Some(schema) = Self::new_single(
                        value_type,
                        object,
                        string_formats,
                        dialect,
                        anchor_collector.as_deref_mut(),
                        dynamic_anchor_collector.as_deref_mut(),
                    ) {
                        schemas.push(Referable::Resolved {
                            schema_uri: None,
                            value: Arc::new(schema),
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
                    not: NotSchema::new(
                        object,
                        string_formats,
                        dialect,
                        anchor_collector.as_deref_mut(),
                        dynamic_anchor_collector.as_deref_mut(),
                    ),
                    if_then_else: IfThenElseSchema::new(
                        object,
                        string_formats,
                        dialect,
                        anchor_collector.as_deref_mut(),
                        dynamic_anchor_collector.as_deref_mut(),
                    )
                    .map(Box::new),
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
                        if schema
                            .resolved()
                            .is_some_and(|value_schema| matches!(value_schema, ValueSchema::Null))
                        {
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
            let schema_visits = crate::SchemaVisits::default();
            self.match_flattened_schemas_with_visits(
                condition,
                schema_uri,
                definitions,
                schema_store,
                &schema_visits,
            )
            .await
        }
        .boxed()
    }

    fn match_flattened_schemas_with_visits<
        'a: 'b,
        'b,
        T: Fn(&ValueSchema) -> bool + Sync + Send,
    >(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        schema_visits: &'a crate::SchemaVisits,
    ) -> BoxFuture<'b, Vec<ValueSchema>> {
        async move {
            let mut matched_schemas = Vec::new();
            match self {
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_store,
                        schema_visits,
                        &[],
                    )
                    .await
                    else {
                        return matched_schemas;
                    };

                    for current_schema in &collected {
                        matched_schemas.extend(
                            current_schema
                                .value_schema
                                .match_flattened_schemas_with_visits(
                                    condition,
                                    &current_schema.schema_uri,
                                    &current_schema.definitions,
                                    schema_store,
                                    schema_visits,
                                )
                                .await,
                        );
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
            let schema_visits = crate::SchemaVisits::default();
            self.is_match_with_visits(
                condition,
                schema_uri,
                definitions,
                schema_store,
                &schema_visits,
            )
            .await
        }
        .boxed()
    }

    fn is_match_with_visits<'a, 'b, T: Fn(&ValueSchema) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        schema_visits: &'a crate::SchemaVisits,
    ) -> BoxFuture<'b, bool>
    where
        'a: 'b,
    {
        async move {
            match self {
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. }) => {
                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_store,
                        schema_visits,
                        &[],
                    )
                    .await
                    else {
                        return false;
                    };

                    join_all(collected.iter().map(|current_schema| async {
                        current_schema
                            .value_schema
                            .is_match_with_visits(
                                condition,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                schema_store,
                                schema_visits,
                            )
                            .await
                    }))
                    .await
                    .into_iter()
                    .any(|is_matched| is_matched)
                }
                ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_store,
                        schema_visits,
                        &[],
                    )
                    .await
                    else {
                        return false;
                    };

                    join_all(collected.iter().map(|current_schema| async {
                        current_schema
                            .value_schema
                            .is_match_with_visits(
                                condition,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                schema_store,
                                schema_visits,
                            )
                            .await
                    }))
                    .await
                    .into_iter()
                    .all(|is_matched| is_matched)
                }
                _ => condition(self),
            }
        }
        .boxed()
    }

    fn find_schema_candidates_with_visits<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        schema_visits: &'a crate::SchemaVisits,
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

                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        schema_store,
                        schema_visits,
                        accessors,
                    )
                    .await
                    else {
                        return (candidates, errors);
                    };

                    for current_schema in &collected {
                        let (mut schema_candidates, schema_errors) = current_schema
                            .value_schema
                            .find_schema_candidates_with_visits(
                                accessors,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                schema_store,
                                schema_visits,
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

impl FindSchemaCandidates for ValueSchema {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)> {
        async move {
            let schema_visits = crate::SchemaVisits::default();
            self.find_schema_candidates_with_visits(
                accessors,
                schema_uri,
                definitions,
                schema_store,
                &schema_visits,
            )
            .await
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::ValueType;

    use super::*;
    use std::str::FromStr;

    fn parse_schema_with_dialect(
        json: &str,
        dialect: Option<crate::JsonSchemaDialect>,
    ) -> Option<ValueSchema> {
        let value_node = tombi_json::ValueNode::from_str(json).unwrap();
        let object = value_node.as_object().unwrap();
        ValueSchema::new(object, None, dialect, None, None)
    }

    #[test]
    fn test_const_string_creates_string_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": "dynamic" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));

        let Some(ValueSchema::String(s)) = schema else {
            panic!("schema is not a String schema");
        };
        assert_eq!(s.const_value, Some("dynamic".to_string()));
    }

    #[test]
    fn test_const_boolean_creates_boolean_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": true }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Boolean(_))));

        let Some(ValueSchema::Boolean(b)) = schema else {
            panic!("schema is not a Boolean schema");
        };
        assert_eq!(b.const_value, Some(true));
    }

    #[test]
    fn test_const_integer_creates_integer_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": 42 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Integer(_))));

        let Some(ValueSchema::Integer(i)) = schema else {
            panic!("schema is not an Integer schema");
        };
        assert_eq!(i.const_value, Some(42));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_const_float_creates_float_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": 3.14 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));

        if let Some(ValueSchema::Float(f)) = schema {
            assert_eq!(f.const_value, Some(3.14));
        }
    }

    #[test]
    fn test_const_null_creates_null_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": null }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Null)));
    }

    #[tokio::test]
    async fn test_anyof_with_const_and_ref_style_integer() {
        // Simulates ruff schema: anyOf with integer and const "dynamic"
        let schema = parse_schema_with_dialect(
            r#"{
                "anyOf": [
                    { "type": "integer" },
                    { "const": "dynamic" }
                ]
            }"#,
            Some(crate::JsonSchemaDialect::Draft07),
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
        let schema = parse_schema_with_dialect(
            r#"{ "const": [1, 2, 3] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_const_empty_array_creates_array_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": [] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_const_object_creates_table_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": {"key": "value"} }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_const_empty_object_creates_table_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": {} }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_const_nested_array_creates_array_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": [[1, 2], [3, 4]] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_content_keywords_are_preserved_as_string_annotations() {
        let schema = parse_schema_with_dialect(
            r#"{
                "type": "string",
                "contentEncoding": "base64",
                "contentMediaType": "application/json",
                "contentSchema": { "type": "object" }
            }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        let Some(ValueSchema::String(schema)) = schema else {
            panic!("schema is not a String schema");
        };

        assert_eq!(schema.content_encoding.as_deref(), Some("base64"));
        assert_eq!(
            schema.content_media_type.as_deref(),
            Some("application/json")
        );
        assert!(schema.content_schema.is_some());
    }

    #[test]
    fn test_content_keywords_infer_string_without_explicit_type() {
        let schema = parse_schema_with_dialect(
            r#"{ "contentMediaType": "application/json" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));
    }

    #[test]
    fn test_const_nested_object_creates_table_schema() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": {"nested": {"key": "value"}} }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    fn parse_schema_with_formats(json: &str, formats: &[StringFormat]) -> Option<ValueSchema> {
        let value_node = tombi_json::ValueNode::from_str(json).unwrap();
        let object = value_node.as_object().unwrap();
        ValueSchema::new(
            object,
            Some(formats),
            Some(crate::JsonSchemaDialect::Draft07),
            None,
            None,
        )
    }

    #[test]
    fn test_const_string_with_format_uses_new_single() {
        // When const has a format field, it should be processed through new_single
        // which handles date-time formats properly
        let schema = parse_schema_with_dialect(
            r#"{ "const": "2024-01-10", "format": "date" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        // Should create LocalDateSchema, not StringSchema
        assert!(matches!(schema, Some(ValueSchema::LocalDate(_))));
    }

    #[test]
    fn test_const_string_with_datetime_format() {
        let schema = parse_schema_with_dialect(
            r#"{ "const": "2024-01-10T12:00:00Z", "format": "date-time" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
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
        let schema = parse_schema_with_dialect(
            r#"{ "enum": [1, 2, 3] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Integer(_))));
    }

    #[test]
    fn test_enum_float_creates_float_schema() {
        // enum with floats should create FloatSchema
        let schema = parse_schema_with_dialect(
            r#"{ "enum": [1.5, 2.5, 3.5] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_enum_mixed_int_float_removes_integer() {
        // When enum contains both integers and floats, integer should be removed
        // and only number should remain, resulting in FloatSchema
        // This tests the logic at lines 198-200: if enum_types contains both
        // "number" and "integer", "integer" is removed
        let schema = parse_schema_with_dialect(
            r#"{ "enum": [1, 2.5] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        // After removing integer, only number remains, so FloatSchema should be created
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[tokio::test]
    async fn test_enum_mixed_int_float_with_other_types() {
        // When enum contains integers, floats, and other types (e.g., string),
        // integer should be removed (lines 198-200), leaving number and string,
        // resulting in OneOf with FloatSchema and StringSchema (but not IntegerSchema)
        let schema = parse_schema_with_dialect(
            r#"{ "enum": [1, 2.5, "text"] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );

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

    // --- Tests for type inference from type-specific keywords ---

    #[test]
    fn test_infer_string_from_min_length() {
        let schema = parse_schema_with_dialect(
            r#"{ "minLength": 1 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));
    }

    #[test]
    fn test_infer_string_from_max_length() {
        let schema = parse_schema_with_dialect(
            r#"{ "maxLength": 100 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));
    }

    #[test]
    fn test_infer_string_from_pattern() {
        let schema = parse_schema_with_dialect(
            r#"{ "pattern": "^[a-z]+$" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));
    }

    #[test]
    fn test_infer_string_from_format() {
        let schema = parse_schema_with_dialect(
            r#"{ "format": "date" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::LocalDate(_))));
    }

    #[test]
    fn test_infer_string_from_format_datetime() {
        let schema = parse_schema_with_dialect(
            r#"{ "format": "date-time" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::OffsetDateTime(_))));
    }

    #[test]
    fn test_infer_number_from_minimum() {
        let schema = parse_schema_with_dialect(
            r#"{ "minimum": 0 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_infer_number_from_maximum() {
        let schema = parse_schema_with_dialect(
            r#"{ "maximum": 100 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_infer_number_from_exclusive_minimum() {
        let schema = parse_schema_with_dialect(
            r#"{ "exclusiveMinimum": 0 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_infer_number_from_exclusive_maximum() {
        let schema = parse_schema_with_dialect(
            r#"{ "exclusiveMaximum": 100 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_infer_number_from_multiple_of() {
        let schema = parse_schema_with_dialect(
            r#"{ "multipleOf": 5 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
    }

    #[test]
    fn test_infer_array_from_items() {
        let schema = parse_schema_with_dialect(
            r#"{ "items": { "type": "string" } }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_infer_array_from_min_items() {
        let schema = parse_schema_with_dialect(
            r#"{ "minItems": 1 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_infer_array_from_max_items() {
        let schema = parse_schema_with_dialect(
            r#"{ "maxItems": 10 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_infer_array_from_unique_items() {
        let schema = parse_schema_with_dialect(
            r#"{ "uniqueItems": true }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Array(_))));
    }

    #[test]
    fn test_infer_array_from_prefix_items_only_in_2020_12() {
        let schema_2020 = parse_schema_with_dialect(
            r#"{ "prefixItems": [ { "type": "string" } ] }"#,
            Some(crate::JsonSchemaDialect::Draft2020_12),
        );
        assert!(matches!(schema_2020, Some(ValueSchema::Array(_))));

        let schema_07 = parse_schema_with_dialect(
            r#"{ "prefixItems": [ { "type": "string" } ] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(schema_07.is_none());
    }

    #[test]
    fn test_infer_object_from_properties() {
        let schema = parse_schema_with_dialect(
            r#"{ "properties": { "name": { "type": "string" } } }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_infer_object_from_required() {
        let schema = parse_schema_with_dialect(
            r#"{ "required": ["name"] }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_infer_object_from_additional_properties() {
        let schema = parse_schema_with_dialect(
            r#"{ "additionalProperties": false }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_infer_object_from_min_properties() {
        let schema = parse_schema_with_dialect(
            r#"{ "minProperties": 1 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_infer_object_from_max_properties() {
        let schema = parse_schema_with_dialect(
            r#"{ "maxProperties": 10 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_infer_object_from_property_names() {
        let schema = parse_schema_with_dialect(
            r#"{ "propertyNames": { "pattern": "^[a-z]+$" } }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Table(_))));
    }

    #[test]
    fn test_infer_string_with_additional_metadata() {
        let schema = parse_schema_with_dialect(
            r#"{ "minLength": 1, "maxLength": 50, "title": "Name", "description": "A name" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::String(_))));
        if let Some(ValueSchema::String(s)) = schema {
            assert_eq!(s.min_length, Some(1));
            assert_eq!(s.max_length, Some(50));
            assert_eq!(s.title.as_deref(), Some("Name"));
            assert_eq!(s.description.as_deref(), Some("A name"));
        }
    }

    #[test]
    fn test_infer_number_with_range() {
        let schema = parse_schema_with_dialect(
            r#"{ "minimum": 0, "maximum": 100 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Float(_))));
        if let Some(ValueSchema::Float(f)) = schema {
            assert_eq!(f.minimum, Some(0.0));
            assert_eq!(f.maximum, Some(100.0));
        }
    }

    #[test]
    fn test_no_inference_without_type_specific_keywords() {
        // Only common keywords like title/description should not trigger inference
        let schema = parse_schema_with_dialect(
            r#"{ "title": "Something", "description": "A thing" }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(schema.is_none());
    }

    #[test]
    fn test_explicit_type_takes_precedence_over_inference() {
        // When type is explicitly specified, it should be used regardless of keywords
        let schema = parse_schema_with_dialect(
            r#"{ "type": "integer", "minimum": 0 }"#,
            Some(crate::JsonSchemaDialect::Draft07),
        );
        assert!(matches!(schema, Some(ValueSchema::Integer(_))));
    }
}
