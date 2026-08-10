use std::{borrow::Cow, str::FromStr};

use futures::future::join_all;
use tombi_future::{BoxFuture, Boxable};
use tombi_json::StringNode;
use tombi_x_keyword::{StringFormat, TomlDateTimeType};

use super::{
    AllOfSchema, AnchorCollector, AnyOfSchema, ArraySchema, BooleanSchema, Deprecation,
    DynamicAnchorCollector, FindSchemaCandidates, FloatSchema, IntegerSchema, LocalDateSchema,
    LocalDateTimeSchema, LocalTimeSchema, OffsetDateTimeSchema, OneOfSchema, SchemaUri,
    StringSchema, TableSchema,
};
use crate::{Accessor, SchemaDefinitions, SchemaStore, schema::any_schema::AnythingSchema};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SchemaView {
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
    Null,
    Anything(AnythingSchema),
    Nothing(tombi_text::Range),
}

impl SchemaView {
    pub fn adjacent_applicators(
        &self,
    ) -> (
        Option<&OneOfSchema>,
        Option<&AnyOfSchema>,
        Option<&AllOfSchema>,
        Option<&super::NotSchema>,
    ) {
        macro_rules! adjacent {
            ($schema:expr) => {
                (
                    $schema.one_of.as_deref(),
                    $schema.any_of.as_deref(),
                    $schema.all_of.as_deref(),
                    $schema.not.as_deref(),
                )
            };
        }

        match self {
            Self::Boolean(schema) => adjacent!(schema),
            Self::Integer(schema) => adjacent!(schema),
            Self::Float(schema) => adjacent!(schema),
            Self::String(schema) => adjacent!(schema),
            Self::LocalDate(schema) => adjacent!(schema),
            Self::LocalDateTime(schema) => adjacent!(schema),
            Self::LocalTime(schema) => adjacent!(schema),
            Self::OffsetDateTime(schema) => adjacent!(schema),
            Self::Array(schema) => adjacent!(schema),
            Self::Table(schema) => adjacent!(schema),
            Self::OneOf(_)
            | Self::AnyOf(_)
            | Self::AllOf(_)
            | Self::Null
            | Self::Anything(_)
            | Self::Nothing(_) => (None, None, None, None),
        }
    }

    pub(crate) fn new_single(
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
            "null" => Some(SchemaView::Null),
            "boolean" => Some(SchemaView::Boolean(BooleanSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ))),
            "integer" => Some(SchemaView::Integer(IntegerSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ))),
            "number" => Some(SchemaView::Float(FloatSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ))),
            "string" => {
                let string_format = if let Some(tombi_json::ValueNode::String(StringNode {
                    value: format_str,
                    ..
                })) = object.get("format")
                {
                    let format = StringFormat::from_str(format_str.as_str()).ok();
                    match format.and_then(StringFormat::toml_date_time_type) {
                        Some(TomlDateTimeType::OffsetDateTime) => {
                            return Some(SchemaView::OffsetDateTime(OffsetDateTimeSchema::new(
                                object,
                                string_formats,
                                dialect,
                                anchor_collector.as_deref_mut(),
                                dynamic_anchor_collector.as_deref_mut(),
                            )));
                        }
                        Some(TomlDateTimeType::LocalDateTime) => {
                            return Some(SchemaView::LocalDateTime(LocalDateTimeSchema::new(
                                object,
                                string_formats,
                                dialect,
                                anchor_collector.as_deref_mut(),
                                dynamic_anchor_collector.as_deref_mut(),
                            )));
                        }
                        Some(TomlDateTimeType::LocalDate) => {
                            return Some(SchemaView::LocalDate(LocalDateSchema::new(
                                object,
                                string_formats,
                                dialect,
                                anchor_collector.as_deref_mut(),
                                dynamic_anchor_collector.as_deref_mut(),
                            )));
                        }
                        Some(TomlDateTimeType::LocalTime) => {
                            return Some(SchemaView::LocalTime(LocalTimeSchema::new(
                                object,
                                string_formats,
                                dialect,
                                anchor_collector.as_deref_mut(),
                                dynamic_anchor_collector.as_deref_mut(),
                            )));
                        }
                        None => format.filter(|format| {
                            string_formats.is_some_and(|formats| formats.contains(format))
                        }),
                    }
                } else {
                    None
                };

                Some(SchemaView::String(StringSchema::new(
                    object,
                    string_format,
                    string_formats,
                    dialect,
                    anchor_collector.as_deref_mut(),
                    dynamic_anchor_collector.as_deref_mut(),
                )))
            }
            "array" => Some(SchemaView::Array(ArraySchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            ))),
            "object" => Some(SchemaView::Table(TableSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector,
                dynamic_anchor_collector,
            ))),
            _ => None,
        }
    }

    pub async fn value_type(&self) -> crate::ValueType {
        match self {
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
            Self::Null | Self::Nothing(_) => crate::ValueType::Null,
            Self::Anything(_) => crate::ValueType::AnyOf(vec![
                crate::ValueType::Boolean,
                crate::ValueType::Integer,
                crate::ValueType::Float,
                crate::ValueType::String,
                crate::ValueType::LocalDate,
                crate::ValueType::LocalDateTime,
                crate::ValueType::LocalTime,
                crate::ValueType::OffsetDateTime,
                crate::ValueType::Array,
                crate::ValueType::Table,
            ]),
        }
    }

    /// The deprecation state of this schema, used for diagnostics and completion.
    ///
    /// For composites (`oneOf`/`anyOf`/`allOf`) the node is deprecated only when every
    /// non-null branch is deprecated; the first branch's `deprecationMessage` (if any) is
    /// surfaced. The schema's own deprecation takes precedence over its branches.
    pub async fn deprecation(&self) -> Option<Deprecation> {
        match self {
            Self::Boolean(boolean) => boolean.deprecation.clone(),
            Self::Integer(integer) => integer.deprecation.clone(),
            Self::Float(float) => float.deprecation.clone(),
            Self::String(string) => string.deprecation.clone(),
            Self::LocalDate(local_date) => local_date.deprecation.clone(),
            Self::LocalDateTime(local_date_time) => local_date_time.deprecation.clone(),
            Self::LocalTime(local_time) => local_time.deprecation.clone(),
            Self::OffsetDateTime(offset_date_time) => offset_date_time.deprecation.clone(),
            Self::Array(array) => array.deprecation.clone(),
            Self::Table(table) => table.deprecation.clone(),
            Self::OneOf(OneOfSchema {
                deprecation,
                schemas,
                ..
            })
            | Self::AnyOf(AnyOfSchema {
                deprecation,
                schemas,
                ..
            })
            | Self::AllOf(AllOfSchema {
                deprecation,
                schemas,
                ..
            }) => {
                if let Some(deprecation) = deprecation {
                    return Some(deprecation.clone());
                }

                let mut has_branch = false;
                let mut message = None;
                for schema in schemas.read().await.iter() {
                    if schema
                        .resolved()
                        .is_some_and(|schema_view| matches!(schema_view, SchemaView::Null))
                    {
                        continue;
                    }
                    has_branch = true;
                    match schema.deprecation().await {
                        Some(deprecation) => {
                            if message.is_none() {
                                message = deprecation.message().map(ToString::to_string);
                            }
                        }
                        None => return None,
                    }
                }

                if has_branch {
                    Some(
                        message
                            .map(Deprecation::Message)
                            .unwrap_or(Deprecation::True),
                    )
                } else {
                    None
                }
            }
            Self::Null | Self::Anything(_) | Self::Nothing(_) => None,
        }
    }

    pub fn title(&self) -> Option<&str> {
        match self {
            SchemaView::Boolean(schema) => schema.title.as_deref(),
            SchemaView::Integer(schema) => schema.title.as_deref(),
            SchemaView::Float(schema) => schema.title.as_deref(),
            SchemaView::String(schema) => schema.title.as_deref(),
            SchemaView::LocalDate(schema) => schema.title.as_deref(),
            SchemaView::LocalDateTime(schema) => schema.title.as_deref(),
            SchemaView::LocalTime(schema) => schema.title.as_deref(),
            SchemaView::OffsetDateTime(schema) => schema.title.as_deref(),
            SchemaView::Array(schema) => schema.title.as_deref(),
            SchemaView::Table(schema) => schema.title.as_deref(),
            SchemaView::OneOf(schema) => schema.title.as_deref(),
            SchemaView::AnyOf(schema) => schema.title.as_deref(),
            SchemaView::AllOf(schema) => schema.title.as_deref(),
            SchemaView::Null | SchemaView::Nothing(_) => None,
            SchemaView::Anything(schema) => schema.title.as_deref(),
        }
    }

    pub fn set_title(&mut self, title: Option<String>) {
        match self {
            SchemaView::Boolean(schema) => schema.title = title,
            SchemaView::Integer(schema) => schema.title = title,
            SchemaView::Float(schema) => schema.title = title,
            SchemaView::String(schema) => schema.title = title,
            SchemaView::LocalDate(schema) => schema.title = title,
            SchemaView::LocalDateTime(schema) => schema.title = title,
            SchemaView::LocalTime(schema) => schema.title = title,
            SchemaView::OffsetDateTime(schema) => schema.title = title,
            SchemaView::Array(schema) => schema.title = title,
            SchemaView::Table(schema) => schema.title = title,
            SchemaView::OneOf(schema) => schema.title = title,
            SchemaView::AnyOf(schema) => schema.title = title,
            SchemaView::AllOf(schema) => schema.title = title,
            SchemaView::Null | SchemaView::Nothing(_) => {}
            SchemaView::Anything(schema) => schema.title = title,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            SchemaView::Boolean(schema) => schema.description.as_deref(),
            SchemaView::Integer(schema) => schema.description.as_deref(),
            SchemaView::Float(schema) => schema.description.as_deref(),
            SchemaView::String(schema) => schema.description.as_deref(),
            SchemaView::LocalDate(schema) => schema.description.as_deref(),
            SchemaView::LocalDateTime(schema) => schema.description.as_deref(),
            SchemaView::LocalTime(schema) => schema.description.as_deref(),
            SchemaView::OffsetDateTime(schema) => schema.description.as_deref(),
            SchemaView::Array(schema) => schema.description.as_deref(),
            SchemaView::Table(schema) => schema.description.as_deref(),
            SchemaView::OneOf(schema) => schema.description.as_deref(),
            SchemaView::AnyOf(schema) => schema.description.as_deref(),
            SchemaView::AllOf(schema) => schema.description.as_deref(),
            SchemaView::Null | SchemaView::Nothing(_) => None,
            SchemaView::Anything(schema) => schema.description.as_deref(),
        }
    }

    pub fn set_description(&mut self, description: Option<String>) {
        match self {
            SchemaView::Boolean(schema) => schema.description = description,
            SchemaView::Integer(schema) => schema.description = description,
            SchemaView::Float(schema) => schema.description = description,
            SchemaView::String(schema) => schema.description = description,
            SchemaView::LocalDate(schema) => schema.description = description,
            SchemaView::LocalDateTime(schema) => schema.description = description,
            SchemaView::LocalTime(schema) => schema.description = description,
            SchemaView::OffsetDateTime(schema) => schema.description = description,
            SchemaView::Array(schema) => schema.description = description,
            SchemaView::Table(schema) => schema.description = description,
            SchemaView::OneOf(schema) => schema.description = description,
            SchemaView::AnyOf(schema) => schema.description = description,
            SchemaView::AllOf(schema) => schema.description = description,
            SchemaView::Null | SchemaView::Nothing(_) => {}
            SchemaView::Anything(schema) => schema.description = description,
        }
    }

    pub fn set_default(&mut self, default: Option<tombi_json::Value>) {
        match default {
            Some(default) => match self {
                SchemaView::Boolean(schema) => {
                    if let Some(value) = default.as_bool() {
                        schema.default = Some(value);
                    }
                }
                SchemaView::Integer(schema) => {
                    if let Some(value) = default.as_i64() {
                        schema.default = Some(value);
                    }
                }
                SchemaView::Float(schema) => {
                    if let Some(value) = default.as_f64() {
                        schema.default = Some(value);
                    }
                }
                SchemaView::String(schema) => {
                    if let Some(value) = default.as_str() {
                        schema.default = Some(value.to_string());
                    }
                }
                SchemaView::LocalDate(schema) => {
                    if let Some(value) = default.as_str() {
                        schema.default = Some(value.to_string());
                    }
                }
                SchemaView::LocalDateTime(schema) => {
                    if let Some(value) = default.as_str() {
                        schema.default = Some(value.to_string());
                    }
                }
                SchemaView::LocalTime(schema) => {
                    if let Some(value) = default.as_str() {
                        schema.default = Some(value.to_string());
                    }
                }
                SchemaView::OffsetDateTime(schema) => {
                    if let Some(value) = default.as_str() {
                        schema.default = Some(value.to_string());
                    }
                }
                SchemaView::Array(schema) => schema.default = Some(default),
                SchemaView::Table(schema) => {
                    if let Some(value) = default.as_object().cloned() {
                        schema.default = Some(value);
                    }
                }
                SchemaView::OneOf(schema) => schema.default = Some(default),
                SchemaView::AnyOf(schema) => schema.default = Some(default),
                SchemaView::AllOf(schema) => schema.default = Some(default),
                SchemaView::Null | SchemaView::Anything(_) | SchemaView::Nothing(_) => {}
            },
            None => match self {
                SchemaView::Boolean(schema) => schema.default = None,
                SchemaView::Integer(schema) => schema.default = None,
                SchemaView::Float(schema) => schema.default = None,
                SchemaView::String(schema) => schema.default = None,
                SchemaView::LocalDate(schema) => schema.default = None,
                SchemaView::LocalDateTime(schema) => schema.default = None,
                SchemaView::LocalTime(schema) => schema.default = None,
                SchemaView::OffsetDateTime(schema) => schema.default = None,
                SchemaView::Array(schema) => schema.default = None,
                SchemaView::Table(schema) => schema.default = None,
                SchemaView::OneOf(schema) => schema.default = None,
                SchemaView::AnyOf(schema) => schema.default = None,
                SchemaView::AllOf(schema) => schema.default = None,
                SchemaView::Null | SchemaView::Anything(_) | SchemaView::Nothing(_) => {}
            },
        }
    }

    pub fn set_examples(&mut self, examples: Option<Vec<tombi_json::Value>>) {
        match examples {
            None => match self {
                SchemaView::Boolean(schema) => schema.examples = None,
                SchemaView::Integer(schema) => schema.examples = None,
                SchemaView::Float(schema) => schema.examples = None,
                SchemaView::String(schema) => schema.examples = None,
                SchemaView::LocalDate(schema) => schema.examples = None,
                SchemaView::LocalDateTime(schema) => schema.examples = None,
                SchemaView::LocalTime(schema) => schema.examples = None,
                SchemaView::OffsetDateTime(schema) => schema.examples = None,
                SchemaView::Array(schema) => schema.examples = None,
                SchemaView::Table(schema) => schema.examples = None,
                SchemaView::OneOf(schema) => schema.examples = None,
                SchemaView::AnyOf(schema) => schema.examples = None,
                SchemaView::AllOf(schema) => schema.examples = None,
                SchemaView::Null | SchemaView::Anything(_) | SchemaView::Nothing(_) => {}
            },
            Some(examples) => match self {
                SchemaView::Boolean(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_bool)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::Integer(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_i64)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::Float(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_f64)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::String(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::LocalDate(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::LocalDateTime(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::LocalTime(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::OffsetDateTime(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(tombi_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::Array(schema) => schema.examples = Some(examples),
                SchemaView::Table(schema) => {
                    let converted: Vec<_> = examples
                        .iter()
                        .filter_map(|example| example.as_object().cloned())
                        .collect();
                    if !converted.is_empty() {
                        schema.examples = Some(converted);
                    }
                }
                SchemaView::OneOf(schema) => schema.examples = Some(examples.clone()),
                SchemaView::AnyOf(schema) => schema.examples = Some(examples.clone()),
                SchemaView::AllOf(schema) => schema.examples = Some(examples),
                SchemaView::Null | SchemaView::Anything(_) | SchemaView::Nothing(_) => {}
            },
        }
    }

    pub(crate) fn set_deprecation(&mut self, deprecation: Deprecation) {
        let deprecation = Some(deprecation);
        match self {
            Self::Boolean(boolean) => boolean.deprecation = deprecation,
            Self::Integer(integer) => integer.deprecation = deprecation,
            Self::Float(float) => float.deprecation = deprecation,
            Self::String(string) => string.deprecation = deprecation,
            Self::LocalDate(local_date) => local_date.deprecation = deprecation,
            Self::LocalDateTime(local_date_time) => local_date_time.deprecation = deprecation,
            Self::LocalTime(local_time) => local_time.deprecation = deprecation,
            Self::OffsetDateTime(offset_date_time) => offset_date_time.deprecation = deprecation,
            Self::Array(array) => array.deprecation = deprecation,
            Self::Table(table) => table.deprecation = deprecation,
            Self::OneOf(one_of) => one_of.deprecation = deprecation,
            Self::AnyOf(any_of) => any_of.deprecation = deprecation,
            Self::AllOf(all_of) => all_of.deprecation = deprecation,
            Self::Null | Self::Anything(_) | Self::Nothing(_) => {}
        }
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            SchemaView::Null => tombi_text::Range::default(),
            SchemaView::Boolean(schema) => schema.range,
            SchemaView::Integer(schema) => schema.range,
            SchemaView::Float(schema) => schema.range,
            SchemaView::String(schema) => schema.range,
            SchemaView::LocalDate(schema) => schema.range,
            SchemaView::LocalDateTime(schema) => schema.range,
            SchemaView::LocalTime(schema) => schema.range,
            SchemaView::OffsetDateTime(schema) => schema.range,
            SchemaView::Array(schema) => schema.range,
            SchemaView::Table(schema) => schema.range,
            SchemaView::OneOf(schema) => schema.range,
            SchemaView::AnyOf(schema) => schema.range,
            SchemaView::AllOf(schema) => schema.range,
            SchemaView::Anything(schema) => schema.range,
            SchemaView::Nothing(range) => *range,
        }
    }

    pub fn match_flattened_schemas<'a: 'b, 'b, T: Fn(&SchemaView) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, Vec<SchemaView>> {
        async move {
            let schema_visits = crate::SchemaVisits::default();
            self.match_flattened_schemas_with_visits(
                condition,
                schema_uri,
                definitions,
                strict,
                schema_store,
                &schema_visits,
            )
            .await
        }
        .boxed()
    }

    fn match_flattened_schemas_with_visits<'a: 'b, 'b, T: Fn(&SchemaView) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        schema_visits: &'a crate::SchemaVisits,
    ) -> BoxFuture<'b, Vec<SchemaView>> {
        async move {
            let mut matched_schemas = Vec::new();
            match self {
                SchemaView::OneOf(OneOfSchema { schemas, .. })
                | SchemaView::AnyOf(AnyOfSchema { schemas, .. })
                | SchemaView::AllOf(AllOfSchema { schemas, .. }) => {
                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        strict,
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
                                .schema_view
                                .match_flattened_schemas_with_visits(
                                    condition,
                                    &current_schema.schema_uri,
                                    &current_schema.definitions,
                                    current_schema.strict,
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

    pub fn is_match<'a, 'b, T: Fn(&SchemaView) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
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
                strict,
                schema_store,
                &schema_visits,
            )
            .await
        }
        .boxed()
    }

    fn is_match_with_visits<'a, 'b, T: Fn(&SchemaView) -> bool + Sync + Send>(
        &'a self,
        condition: &'a T,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        schema_visits: &'a crate::SchemaVisits,
    ) -> BoxFuture<'b, bool>
    where
        'a: 'b,
    {
        async move {
            match self {
                SchemaView::OneOf(OneOfSchema { schemas, .. })
                | SchemaView::AnyOf(AnyOfSchema { schemas, .. }) => {
                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        strict,
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
                            .schema_view
                            .is_match_with_visits(
                                condition,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                current_schema.strict,
                                schema_store,
                                schema_visits,
                            )
                            .await
                    }))
                    .await
                    .into_iter()
                    .any(|is_matched| is_matched)
                }
                SchemaView::AllOf(AllOfSchema { schemas, .. }) => {
                    let Some(collected) = crate::resolve_and_collect_schemas(
                        schemas,
                        Cow::Borrowed(schema_uri),
                        Cow::Borrowed(definitions),
                        strict,
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
                            .schema_view
                            .is_match_with_visits(
                                condition,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                current_schema.strict,
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
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        schema_visits: &'a crate::SchemaVisits,
    ) -> BoxFuture<'b, (Vec<SchemaView>, Vec<crate::Error>)> {
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
                        strict,
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
                            .schema_view
                            .find_schema_candidates_with_visits(
                                accessors,
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                current_schema.strict,
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
                SchemaView::Null => (Vec::new(), Vec::new()),
                _ => (vec![self.clone()], Vec::new()),
            }
        }
        .boxed()
    }
}

impl FindSchemaCandidates for SchemaView {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<SchemaView>, Vec<crate::Error>)> {
        async move {
            let schema_visits = crate::SchemaVisits::default();
            self.find_schema_candidates_with_visits(
                accessors,
                schema_uri,
                definitions,
                strict,
                schema_store,
                &schema_visits,
            )
            .await
        }
        .boxed()
    }
}
