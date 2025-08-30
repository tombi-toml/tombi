use std::{borrow::Cow, str::FromStr};

use tombi_future::Boxable;
use tombi_schema_store::{
    AllOfSchema, AnyOfSchema, OneOfSchema, SchemaContext, SchemaDefinitions, SchemaUri, ValueSchema,
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
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>>;
}

impl GetEnumerate for ValueSchema {
    fn get_enumerate<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
        async move {
            match self {
                ValueSchema::Boolean(schema) => {
                    let mut enumerate_values = Vec::new();

                    // Add const_value if present
                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::Boolean(*const_value));
                    }

                    // Add enumerate values if present
                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values
                            .extend(enumerate.iter().map(|v| DisplayValue::Boolean(*v)));
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::Integer(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::Integer(*const_value));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values
                            .extend(enumerate.iter().map(|v| DisplayValue::Integer(*v)));
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::Float(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::Float(*const_value));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values.extend(enumerate.iter().map(|v| DisplayValue::Float(*v)));
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::String(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::String(const_value.clone()));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values
                            .extend(enumerate.iter().map(|v| DisplayValue::String(v.clone())));
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::OffsetDateTime(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::OffsetDateTime(const_value.clone()));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values.extend(
                            enumerate
                                .iter()
                                .map(|v| DisplayValue::OffsetDateTime(v.clone())),
                        );
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::LocalDateTime(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::LocalDateTime(const_value.clone()));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values.extend(
                            enumerate
                                .iter()
                                .map(|v| DisplayValue::LocalDateTime(v.clone())),
                        );
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::LocalDate(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::LocalDate(const_value.clone()));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values
                            .extend(enumerate.iter().map(|v| DisplayValue::LocalDate(v.clone())));
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::LocalTime(schema) => {
                    let mut enumerate_values = Vec::new();

                    if let Some(const_value) = &schema.const_value {
                        enumerate_values.push(DisplayValue::LocalTime(const_value.clone()));
                    }

                    if let Some(enumerate) = &schema.enumerate {
                        enumerate_values
                            .extend(enumerate.iter().map(|v| DisplayValue::LocalTime(v.clone())));
                    }

                    if !enumerate_values.is_empty() {
                        Some(enumerate_values)
                    } else {
                        None
                    }
                }
                ValueSchema::Array(_) | ValueSchema::Table(_) | ValueSchema::Null => None,
                ValueSchema::OneOf(OneOfSchema { schemas, .. })
                | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
                | ValueSchema::AllOf(AllOfSchema { schemas, .. }) => {
                    get_enumerate_from_schemas(schemas, schema_uri, definitions, schema_context)
                        .await
                }
            }
        }
        .boxed()
    }
}

/// Helper function to get enumerate values from a collection of schemas
fn get_enumerate_from_schemas<'a: 'b, 'b>(
    schemas: &'a tombi_schema_store::ReferableValueSchemas,
    schema_uri: &'a SchemaUri,
    definitions: &'a SchemaDefinitions,
    schema_context: &'a SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<Vec<DisplayValue>>> {
    async move {
        let mut enumerate_values = Vec::new();
        for schema in schemas.write().await.iter_mut() {
            if let Ok(Some(resolved)) = schema
                .resolve(
                    Cow::Borrowed(schema_uri),
                    Cow::Borrowed(definitions),
                    schema_context.store,
                )
                .await
            {
                if let Some(values) = resolved
                    .value_schema
                    .get_enumerate(schema_uri, definitions, schema_context)
                    .await
                {
                    enumerate_values.extend(values);
                }
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
