use std::{borrow::Cow, str::FromStr};

use tombi_future::Boxable;
use tombi_schema_store::{
    AllOfSchema, AnyOfSchema, OneOfSchema, SchemaContext, SchemaDefinitions, SchemaUrl, ValueSchema,
};

#[derive(Debug, Clone)]
pub enum DisplayValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    OffsetDateTime(String),
    LocalDateTime(String),
    LocalDate(String),
    LocalTime(String),
    Array(Vec<DisplayValue>),
    Table(Vec<(String, DisplayValue)>),
}

impl DisplayValue {
    pub fn try_new_offset_date_time(
        local_date_time: &str,
    ) -> Result<Self, tombi_date_time::parse::Error> {
        tombi_date_time::LocalDateTime::from_str(local_date_time)?;
        Ok(DisplayValue::OffsetDateTime(local_date_time.to_string()))
    }

    pub fn try_new_local_date_time(
        local_date_time: &str,
    ) -> Result<Self, tombi_date_time::parse::Error> {
        tombi_date_time::LocalDateTime::from_str(local_date_time)?;
        Ok(DisplayValue::LocalDateTime(local_date_time.to_string()))
    }

    pub fn try_new_local_date(local_date: &str) -> Result<Self, tombi_date_time::parse::Error> {
        tombi_date_time::LocalDate::from_str(local_date)?;
        Ok(DisplayValue::LocalDate(local_date.to_string()))
    }

    pub fn try_new_local_time(local_time: &str) -> Result<Self, tombi_date_time::parse::Error> {
        tombi_date_time::LocalTime::from_str(local_time)?;
        Ok(DisplayValue::LocalTime(local_time.to_string()))
    }
}

impl TryFrom<&tombi_json::Value> for DisplayValue {
    type Error = ();

    fn try_from(value: &tombi_json::Value) -> Result<Self, Self::Error> {
        match value {
            tombi_json::Value::Bool(boolean) => Ok(DisplayValue::Boolean(*boolean)),
            tombi_json::Value::Number(number) => match number {
                tombi_json::Number::Integer(integer) => Ok(DisplayValue::Integer(*integer)),
                tombi_json::Number::Float(float) => Ok(DisplayValue::Float(*float)),
            },
            tombi_json::Value::String(string) => Ok(DisplayValue::String(string.clone())),
            tombi_json::Value::Array(array) => Ok(DisplayValue::Array(
                array.iter().map(|item| item.try_into().unwrap()).collect(),
            )),
            tombi_json::Value::Object(object) => Ok(DisplayValue::Table(
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), value.try_into().unwrap()))
                    .collect(),
            )),
            tombi_json::Value::Null => Err(()),
        }
    }
}

impl From<tombi_json::Object> for DisplayValue {
    fn from(object: tombi_json::Object) -> Self {
        DisplayValue::Table(
            object
                .into_inner()
                .iter()
                .filter_map(|(key, value)| value.try_into().map(|v| (key.clone(), v)).ok())
                .collect(),
        )
    }
}

impl From<&tombi_json::Object> for DisplayValue {
    fn from(object: &tombi_json::Object) -> Self {
        DisplayValue::Table(
            object
                .iter()
                .filter_map(|(key, value)| value.try_into().map(|v| (key.clone(), v)).ok())
                .collect(),
        )
    }
}

impl std::fmt::Display for DisplayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayValue::Boolean(boolean) => write!(f, "{boolean}"),
            DisplayValue::Integer(integer) => write!(f, "{integer}"),
            DisplayValue::Float(float) => write!(f, "{float}"),
            DisplayValue::String(string) => write!(f, "\"{}\"", string.replace("\"", "\\\"")),
            DisplayValue::OffsetDateTime(offset_date_time) => write!(f, "{offset_date_time}"),
            DisplayValue::LocalDateTime(local_date_time) => write!(f, "{local_date_time}"),
            DisplayValue::LocalDate(local_date) => write!(f, "{local_date}"),
            DisplayValue::LocalTime(local_time) => write!(f, "{local_time}"),
            DisplayValue::Array(array) => {
                write!(f, "[")?;
                for (i, value) in array.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, "]")
            }
            DisplayValue::Table(table) => {
                write!(f, "{{ ")?;
                for (i, (key, value)) in table.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {value}")?;
                }
                write!(f, " }}")
            }
        }
    }
}

pub trait GetEnumerate {
    fn get_enumerate<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>>;
}

impl GetEnumerate for ValueSchema {
    fn get_enumerate<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
        async move {
            match self {
                ValueSchema::Boolean(schema) => schema
                    .enumerate
                    .as_ref()
                    .map(|v| v.clone().into_iter().map(DisplayValue::Boolean).collect()),
                ValueSchema::Integer(schema) => schema
                    .enumerate
                    .as_ref()
                    .map(|v| v.clone().into_iter().map(DisplayValue::Integer).collect()),
                ValueSchema::Float(schema) => schema
                    .enumerate
                    .as_ref()
                    .map(|v| v.clone().into_iter().map(DisplayValue::Float).collect()),
                ValueSchema::String(schema) => schema
                    .enumerate
                    .as_ref()
                    .map(|v| v.clone().into_iter().map(DisplayValue::String).collect()),
                ValueSchema::LocalDate(schema) => schema
                    .enumerate
                    .as_ref()
                    .map(|v| v.clone().into_iter().map(DisplayValue::LocalDate).collect()),
                ValueSchema::LocalDateTime(schema) => schema.enumerate.as_ref().map(|v| {
                    v.clone()
                        .into_iter()
                        .map(DisplayValue::LocalDateTime)
                        .collect()
                }),
                ValueSchema::LocalTime(schema) => schema
                    .enumerate
                    .as_ref()
                    .map(|v| v.clone().into_iter().map(DisplayValue::LocalTime).collect()),
                ValueSchema::OffsetDateTime(schema) => schema.enumerate.as_ref().map(|v| {
                    v.clone()
                        .into_iter()
                        .map(DisplayValue::OffsetDateTime)
                        .collect()
                }),
                ValueSchema::Array(_) | ValueSchema::Table(_) | ValueSchema::Null => None,
                ValueSchema::OneOf(schema) => {
                    schema
                        .get_enumerate(schema_url, definitions, schema_context)
                        .await
                }
                ValueSchema::AnyOf(schema) => {
                    schema
                        .get_enumerate(schema_url, definitions, schema_context)
                        .await
                }
                ValueSchema::AllOf(schema) => {
                    schema
                        .get_enumerate(schema_url, definitions, schema_context)
                        .await
                }
            }
        }
        .boxed()
    }
}

impl GetEnumerate for OneOfSchema {
    fn get_enumerate<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
        get_enumerate_from_schemas(&self.schemas, schema_url, definitions, schema_context)
    }
}

impl GetEnumerate for AnyOfSchema {
    fn get_enumerate<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
        get_enumerate_from_schemas(&self.schemas, schema_url, definitions, schema_context)
    }
}

impl GetEnumerate for AllOfSchema {
    fn get_enumerate<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
        get_enumerate_from_schemas(&self.schemas, schema_url, definitions, schema_context)
    }
}

/// Helper function to get enumerate values from a collection of schemas
fn get_enumerate_from_schemas<'a: 'b, 'b>(
    schemas: &'a tombi_schema_store::ReferableValueSchemas,
    schema_url: &'a SchemaUrl,
    definitions: &'a SchemaDefinitions,
    schema_context: &'a SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
    async move {
        let mut enumerate_values = Vec::new();
        for schema in schemas.write().await.iter_mut() {
            match schema
                .resolve(
                    Cow::Borrowed(schema_url),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            {
                Ok(Some(resolved)) => {
                    if let Some(values) = resolved
                        .value_schema
                        .get_enumerate(schema_url, definitions, schema_context)
                        .await
                    {
                        enumerate_values.extend(values);
                    }
                }
                _ => {}
            }
        }

        if enumerate_values.is_empty() {
            None
        } else {
            Some(enumerate_values)
        }
    }
    .boxed()
}
