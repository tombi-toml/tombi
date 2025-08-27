use tombi_toml_version::TomlVersion;

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    OffsetDatetime(String),
    LocalDatetime(String),
    LocalDate(String),
    LocalTime(String),
    Array(Vec<Value>),
    Table(indexmap::IndexMap<String, Value>),
}

pub trait IntoValue {
    fn into_value(self, toml_version: TomlVersion) -> Value;
}

impl IntoValue for tombi_document_tree::Value {
    fn into_value(self, toml_version: TomlVersion) -> Value {
        match self {
            tombi_document_tree::Value::Boolean(value) => Value::Bool(value.value()),
            tombi_document_tree::Value::Integer(value) => Value::Integer(value.value()),
            tombi_document_tree::Value::Float(value) => Value::Float(value.value()),
            tombi_document_tree::Value::String(value) => Value::String(value.into_value()),
            tombi_document_tree::Value::OffsetDateTime(value) => {
                Value::OffsetDatetime(value.value().to_string())
            }
            tombi_document_tree::Value::LocalDateTime(value) => {
                Value::LocalDatetime(value.value().to_string())
            }
            tombi_document_tree::Value::LocalDate(value) => {
                Value::LocalDate(value.value().to_string())
            }
            tombi_document_tree::Value::LocalTime(value) => {
                Value::LocalTime(value.value().to_string())
            }
            tombi_document_tree::Value::Array(array) => Value::Array(
                array
                    .into_iter()
                    .map(|value| value.into_value(toml_version))
                    .collect(),
            ),
            tombi_document_tree::Value::Table(value) => Value::Table(
                value
                    .into_iter()
                    .map(|(k, v)| (k.to_raw_text(toml_version), v.into_value(toml_version)))
                    .collect(),
            ),
            tombi_document_tree::Value::Incomplete { .. } => {
                unreachable!("Incomplete value should not be converted to Value.")
            }
        }
    }
}

impl IntoValue for tombi_document_tree::DocumentTree {
    fn into_value(self, toml_version: TomlVersion) -> Value {
        Value::Table(
            tombi_document_tree::Table::from(self)
                .into_iter()
                .map(|(k, v)| (k.to_raw_text(toml_version), v.into_value(toml_version)))
                .collect(),
        )
    }
}
