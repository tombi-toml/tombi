use tombi_toml_version::TomlVersion;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Type {
    Bool,
    Integer,
    Float,
    String,
    Datetime,
    DatetimeLocal,
    DateLocal,
    TimeLocal,
}

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum Value {
    Literal { r#type: Type, value: String },
    Array(Vec<Value>),
    Table(indexmap::IndexMap<String, Value>),
}

pub trait IntoValue {
    fn into_value(self, toml_version: TomlVersion) -> Value;
}

impl IntoValue for tombi_document_tree::Value {
    fn into_value(self, toml_version: TomlVersion) -> Value {
        match self {
            tombi_document_tree::Value::Boolean(value) => Value::Literal {
                r#type: Type::Bool,
                value: value.value().to_string(),
            },
            tombi_document_tree::Value::Integer(value) => Value::Literal {
                r#type: Type::Integer,
                value: value.value().to_string(),
            },
            tombi_document_tree::Value::Float(value) => Value::Literal {
                r#type: Type::Float,
                value: value.value().to_string(),
            },
            tombi_document_tree::Value::String(value) => Value::Literal {
                r#type: Type::String,
                value: value.into_value(),
            },
            tombi_document_tree::Value::OffsetDateTime(value) => Value::Literal {
                r#type: Type::Datetime,
                value: value.value().to_string(),
            },
            tombi_document_tree::Value::LocalDateTime(value) => Value::Literal {
                r#type: Type::DatetimeLocal,
                value: value.value().to_string(),
            },
            tombi_document_tree::Value::LocalDate(value) => Value::Literal {
                r#type: Type::DateLocal,
                value: value.value().to_string(),
            },
            tombi_document_tree::Value::LocalTime(value) => Value::Literal {
                r#type: Type::TimeLocal,
                value: value.value().to_string(),
            },
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
