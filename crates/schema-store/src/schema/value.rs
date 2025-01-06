mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;

pub use array::ArraySchema;
pub use boolean::BooleanSchema;
pub use float::FloatSchema;
pub use integer::IntegerSchema;
pub use local_date::LocalDateSchema;
pub use local_date_time::LocalDateTimeSchema;
pub use local_time::LocalTimeSchema;
pub use offset_date_time::OffsetDateTimeSchema;
pub use string::StringSchema;
pub use table::TableSchema;

use super::{referable::Referable, DocumentSchema};

#[derive(Debug, Clone, PartialEq)]
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
    OneOf(Vec<Referable<ValueSchema>>),
    AnyOf(Vec<Referable<ValueSchema>>),
    AllOf(Vec<Referable<ValueSchema>>),
}

impl ValueSchema {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        if let Some(_type) = object.get("type") {
            if let serde_json::Value::String(type_str) = _type {
                return match type_str.as_str() {
                    "null" => Some(ValueSchema::Null),
                    "boolean" => Some(ValueSchema::Boolean(BooleanSchema::new(object))),
                    "integer" => Some(ValueSchema::Integer(IntegerSchema::new(object))),
                    "number" => Some(ValueSchema::Float(FloatSchema::new(object))),
                    "string" => Some(ValueSchema::String(StringSchema::new(object))),
                    "array" => Some(ValueSchema::Array(ArraySchema::new(object))),
                    "object" => Some(ValueSchema::Table(TableSchema::new(object))),
                    "local_date" => Some(ValueSchema::LocalDate(LocalDateSchema::new(object))),
                    "local_date_time" => {
                        Some(ValueSchema::LocalDateTime(LocalDateTimeSchema::new(object)))
                    }
                    "local_time" => Some(ValueSchema::LocalTime(LocalTimeSchema::new(object))),
                    "offset_date_time" => Some(ValueSchema::OffsetDateTime(
                        OffsetDateTimeSchema::new(object),
                    )),
                    _ => None,
                };
            }
        }
        None
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
            ValueSchema::OneOf(_) => None,
            ValueSchema::AnyOf(_) => None,
            ValueSchema::AllOf(_) => None,
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
            ValueSchema::OneOf(_) => None,
            ValueSchema::AnyOf(_) => None,
            ValueSchema::AllOf(_) => None,
        }
    }
}

impl Referable<ValueSchema> {
    pub fn new(object: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        if let Some(ref_value) = object.get("$ref") {
            if let serde_json::Value::String(ref_str) = ref_value {
                return Some(Referable::Ref(ref_str.clone()));
            }
        }

        ValueSchema::new(object).map(Referable::Resolved)
    }

    pub fn resolve<'a>(
        &'a mut self,
        document_schema: &'a DocumentSchema,
    ) -> Result<&'a ValueSchema, crate::Error> {
        match self {
            Referable::Ref(ref_str) => {
                let definitions = match document_schema.definitions.read() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Err(crate::Error::LockError {
                            ref_string: ref_str.clone(),
                        });
                    }
                };

                if let Some(definition_schema) = definitions.get(ref_str) {
                    *self = definition_schema.clone();
                    self.resolve(document_schema)
                } else {
                    Err(crate::Error::DefinitionNotFound {
                        definition_ref: ref_str.clone(),
                    })
                }
            }
            Referable::Resolved(resolved) => {
                match resolved {
                    ValueSchema::OneOf(schemas)
                    | ValueSchema::AnyOf(schemas)
                    | ValueSchema::AllOf(schemas) => {
                        for schema in schemas {
                            schema.resolve(document_schema)?;
                        }
                    }
                    _ => {}
                }
                Ok(resolved)
            }
        }
    }
}
