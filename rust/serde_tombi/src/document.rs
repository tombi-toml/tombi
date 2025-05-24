use itertools::Itertools;

pub use tombi_document::{
    Array, ArrayKind, Boolean, Document, Float, Integer, IntegerKind, Key, LocalDate,
    LocalDateTime, LocalTime, OffsetDateTime, String, StringKind, Table, TableKind, Value,
};

/// A trait for converting TOML values to their string representation.
pub(crate) trait ToTomlString {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        parent_keys: &[&tombi_document::Key],
    );
}

impl ToTomlString for (&tombi_document::Key, &tombi_document::Value) {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        parent_keys: &[&tombi_document::Key],
    ) {
        let (key, value) = *self;
        match value {
            tombi_document::Value::Table(table)
                if table.kind() == tombi_document::TableKind::KeyValue =>
            {
                table.to_toml_string(
                    result,
                    &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                );
            }
            _ => {
                result.push_str(&format!(
                    "{} = ",
                    parent_keys.iter().chain(&[key]).copied().join(".")
                ));
                value.to_toml_string(result, &[]);
            }
        }
        result.push('\n');
    }
}

impl ToTomlString for tombi_document::Value {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        parent_keys: &[&tombi_document::Key],
    ) {
        match self {
            tombi_document::Value::String(s) => result.push_str(&format!("\"{}\"", s.value())),
            tombi_document::Value::Integer(i) => result.push_str(&i.value().to_string()),
            tombi_document::Value::Float(f) => result.push_str(&f.value().to_string()),
            tombi_document::Value::Boolean(b) => result.push_str(&b.value().to_string()),
            tombi_document::Value::Array(a) => a.to_toml_string(result, parent_keys),
            tombi_document::Value::Table(t) => t.to_toml_string(result, parent_keys),
            tombi_document::Value::OffsetDateTime(dt) => result.push_str(&dt.to_string()),
            tombi_document::Value::LocalDateTime(dt) => result.push_str(&dt.to_string()),
            tombi_document::Value::LocalDate(d) => result.push_str(&d.to_string()),
            tombi_document::Value::LocalTime(t) => result.push_str(&t.to_string()),
        }
    }
}

impl ToTomlString for tombi_document::Table {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        parent_keys: &[&tombi_document::Key],
    ) {
        match self.kind() {
            tombi_document::TableKind::Table => {
                if self.key_values().len() == 1 {
                    if let Some((key, value)) = self.key_values().iter().next() {
                        match value {
                            tombi_document::Value::Table(table)
                                if table.kind() == tombi_document::TableKind::Table =>
                            {
                                return table.to_toml_string(
                                    result,
                                    &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                                );
                            }
                            tombi_document::Value::Array(array)
                                if array.kind() == tombi_document::ArrayKind::ArrayOfTable =>
                            {
                                return array.to_toml_string(
                                    result,
                                    &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                                );
                            }
                            _ => {}
                        }
                    }
                }

                if !parent_keys.is_empty() {
                    result.push_str(&format!(
                        "[{}]\n",
                        parent_keys
                            .iter()
                            .map(ToString::to_string)
                            .collect_vec()
                            .join(".")
                    ));
                }

                let mut table_key_values = Vec::new();
                for (key, value) in self.key_values() {
                    match value {
                        tombi_document::Value::Table(table)
                            if table.kind() == tombi_document::TableKind::Table =>
                        {
                            table_key_values.push((key, value));
                            continue;
                        }
                        tombi_document::Value::Array(array)
                            if array.kind() == tombi_document::ArrayKind::ArrayOfTable =>
                        {
                            table_key_values.push((key, value));
                            continue;
                        }
                        _ => (key, value).to_toml_string(result, &[]),
                    }
                }

                for (key, value) in table_key_values {
                    value.to_toml_string(
                        result,
                        &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                    );
                }
            }
            tombi_document::TableKind::InlineTable => {
                result.push('{');
                for (i, (key, value)) in self.key_values().iter().enumerate() {
                    if i != 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!(
                        "{} = ",
                        parent_keys.iter().chain(&[key]).copied().join(".")
                    ));
                    value.to_toml_string(
                        result,
                        &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                    );
                }
                result.push('}');
            }
            tombi_document::TableKind::KeyValue => {
                for key_value in self.key_values() {
                    key_value.to_toml_string(result, parent_keys);
                }
            }
        }
    }
}

impl ToTomlString for tombi_document::Array {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        parent_keys: &[&tombi_document::Key],
    ) {
        match self.kind() {
            tombi_document::ArrayKind::Array => {
                result.push('[');
                if !self.values().is_empty() {
                    for (i, value) in self.values().iter().enumerate() {
                        if i != 0 {
                            result.push_str(", ");
                        }
                        value.to_toml_string(result, parent_keys);
                    }
                }
                result.push(']');
            }
            tombi_document::ArrayKind::ArrayOfTable => {
                for value in self.values().iter() {
                    result.push_str(&format!(
                        "[[{}]]\n",
                        parent_keys
                            .iter()
                            .map(ToString::to_string)
                            .collect_vec()
                            .join(".")
                    ));
                    if let tombi_document::Value::Table(table) = value {
                        for (key, value) in table.key_values() {
                            match value {
                                tombi_document::Value::Table(table)
                                    if table.kind() == tombi_document::TableKind::KeyValue =>
                                {
                                    table.to_toml_string(
                                        result,
                                        &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                                    );
                                }
                                _ => {
                                    result.push_str(&format!("{} = ", key));
                                    value.to_toml_string(
                                        result,
                                        &parent_keys.iter().chain(&[key]).copied().collect_vec(),
                                    );
                                }
                            }
                            result.push('\n');
                        }
                    }
                }
            }
        }
    }
}

impl ToTomlString for tombi_document::Boolean {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        result.push_str(if self.value() { "true" } else { "false" });
    }
}

impl ToTomlString for tombi_document::Float {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        if self.value().is_infinite() {
            if self.value().is_sign_positive() {
                result.push_str("inf");
            } else {
                result.push_str("-inf");
            }
        } else if self.value().is_nan() {
            result.push_str("nan");
        } else {
            result.push_str(&self.value().to_string());
        }
    }
}

impl ToTomlString for tombi_document::String {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        match self.kind() {
            tombi_document::StringKind::BasicString => {
                result.push_str(&tombi_toml_text::to_basic_string(self.value()));
            }
            tombi_document::StringKind::LiteralString => {
                result.push_str(&tombi_toml_text::to_literal_string(self.value()));
            }
            tombi_document::StringKind::MultiLineBasicString => {
                result.push_str(&tombi_toml_text::to_multi_line_basic_string(self.value()));
            }
            tombi_document::StringKind::MultiLineLiteralString => {
                result.push_str(&tombi_toml_text::to_multi_line_literal_string(self.value()));
            }
        }
    }
}

impl ToTomlString for tombi_document::OffsetDateTime {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        result.push_str(&self.to_string());
    }
}

impl ToTomlString for tombi_document::LocalDateTime {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        result.push_str(&self.to_string());
    }
}

impl ToTomlString for tombi_document::LocalDate {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        result.push_str(&self.to_string());
    }
}

impl ToTomlString for tombi_document::LocalTime {
    fn to_toml_string(
        &self,
        result: &mut std::string::String,
        _parent_keys: &[&tombi_document::Key],
    ) {
        result.push_str(&self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use tombi_test_lib::toml_text_assert_eq;
    use tombi_document::KeyKind;

    use crate::document::*;

    #[tokio::test]
    async fn test_document_serialization() {
        tombi_test_lib::init_tracing();

        // Create a test document with various value types
        let mut document = Document::new();

        // Add string value
        document.insert(
            Key::new(KeyKind::BareKey, "string".to_string()),
            Value::String(String::new(StringKind::BasicString, "hello".to_string())),
        );

        // Add integer value
        document.insert(
            Key::new(KeyKind::BareKey, "integer".to_string()),
            Value::Integer(Integer::new(42)),
        );

        // Add float value
        document.insert(
            Key::new(KeyKind::BareKey, "float".to_string()),
            Value::Float(Float::new(std::f64::consts::PI)),
        );

        // Add boolean value
        document.insert(
            Key::new(KeyKind::BareKey, "boolean".to_string()),
            Value::Boolean(Boolean::new(true)),
        );

        // Add array value
        let mut array = Array::new(ArrayKind::Array);
        array.push(Value::Integer(Integer::new(1)));
        array.push(Value::Integer(Integer::new(2)));
        array.push(Value::Integer(Integer::new(3)));
        document.insert(
            Key::new(KeyKind::BareKey, "array".to_string()),
            Value::Array(array),
        );

        // Test to_string method
        let toml_string = crate::to_string_async(&document).await.unwrap();
        let expected = r#"
string = "hello"
integer = 42
float = 3.141592653589793
boolean = true
array = [1, 2, 3]
"#;
        toml_text_assert_eq!(toml_string, expected);
    }

    #[tokio::test]
    async fn test_array_of_tables_serialization() {
        tombi_test_lib::init_tracing();

        // Create a test document with array of tables
        let mut document = Document::new();

        // Create array of tables
        let mut array_of_tables = Array::new(ArrayKind::ArrayOfTable);

        // First table in array
        let mut table1 = Table::new(TableKind::Table);
        table1.insert(
            Key::new(KeyKind::BareKey, "name".to_string()),
            Value::String(String::new(StringKind::BasicString, "apple".to_string())),
        );
        table1.insert(
            Key::new(KeyKind::BareKey, "color".to_string()),
            Value::String(String::new(StringKind::BasicString, "red".to_string())),
        );
        array_of_tables.push(Value::Table(table1));

        // Second table in array
        let mut table2 = Table::new(TableKind::Table);
        table2.insert(
            Key::new(KeyKind::BareKey, "name".to_string()),
            Value::String(String::new(StringKind::BasicString, "banana".to_string())),
        );
        table2.insert(
            Key::new(KeyKind::BareKey, "color".to_string()),
            Value::String(String::new(StringKind::BasicString, "yellow".to_string())),
        );
        array_of_tables.push(Value::Table(table2));

        // Add array of tables to root table
        document.insert(
            Key::new(KeyKind::BareKey, "fruits".to_string()),
            Value::Array(array_of_tables),
        );

        // Test to_string method
        let toml_string = crate::to_string_async(&document).await.unwrap();
        let expected = r#"
[[fruits]]
name = "apple"
color = "red"

[[fruits]]
name = "banana"
color = "yellow"
"#;
        toml_text_assert_eq!(toml_string, expected);
    }

    #[tokio::test]
    async fn test_nested_tables_serialization() {
        tombi_test_lib::init_tracing();

        // Create a test document with nested tables
        let mut document = Document::new();

        // Create nested table
        let mut nested_table = Table::new(TableKind::Table);
        nested_table.insert(
            Key::new(KeyKind::BareKey, "name".to_string()),
            Value::String(String::new(StringKind::BasicString, "John".to_string())),
        );
        nested_table.insert(
            Key::new(KeyKind::BareKey, "age".to_string()),
            Value::Integer(Integer::new(30)),
        );

        // Add nested table to root table
        document.insert(
            Key::new(KeyKind::BareKey, "person".to_string()),
            Value::Table(nested_table),
        );

        // Test to_string method
        let toml_string = crate::to_string_async(&document).await.unwrap();
        let expected = r#"
[person]
name = "John"
age = 30
"#;
        toml_text_assert_eq!(toml_string, expected);
    }

    #[tokio::test]
    async fn test_complex_nested_structures_serialization() {
        tombi_test_lib::init_tracing();

        let mut document = Document::new();

        // Create nested table structure [aaa.bbb]
        let mut aaa_table = Table::new(TableKind::Table);
        let mut bbb_table = Table::new(TableKind::Table);

        // Add values to [aaa.bbb]
        bbb_table.insert(
            Key::new(KeyKind::BareKey, "ddd".to_string()),
            Value::String(String::new(StringKind::BasicString, "value1".to_string())),
        );

        // Create and add inline table
        let mut inline_table = Table::new(TableKind::InlineTable);
        inline_table.insert(
            Key::new(KeyKind::BareKey, "x".to_string()),
            Value::Integer(Integer::new(1)),
        );
        inline_table.insert(
            Key::new(KeyKind::BareKey, "y".to_string()),
            Value::Integer(Integer::new(2)),
        );
        bbb_table.insert(
            Key::new(KeyKind::BareKey, "inline".to_string()),
            Value::Table(inline_table),
        );

        // Create nested table [aaa.bbb.ccc]
        let mut ccc_table = Table::new(TableKind::Table);
        ccc_table.insert(
            Key::new(KeyKind::BareKey, "value".to_string()),
            Value::String(String::new(
                StringKind::BasicString,
                "deep nested".to_string(),
            )),
        );

        // Create array of tables
        let mut array_of_tables = Array::new(ArrayKind::ArrayOfTable);
        let mut array_table1 = Table::new(TableKind::Table);
        array_table1.insert(
            Key::new(KeyKind::BareKey, "id".to_string()),
            Value::Integer(Integer::new(1)),
        );
        array_of_tables.push(Value::Table(array_table1));

        let mut array_table2 = Table::new(TableKind::Table);
        array_table2.insert(
            Key::new(KeyKind::BareKey, "id".to_string()),
            Value::Integer(Integer::new(2)),
        );
        array_of_tables.push(Value::Table(array_table2));

        // Add array of tables to ccc_table
        ccc_table.insert(
            Key::new(KeyKind::BareKey, "items".to_string()),
            Value::Array(array_of_tables),
        );

        // Add ccc_table to bbb_table
        bbb_table.insert(
            Key::new(KeyKind::BareKey, "ccc".to_string()),
            Value::Table(ccc_table),
        );

        // Add bbb_table to aaa_table
        aaa_table.insert(
            Key::new(KeyKind::BareKey, "bbb".to_string()),
            Value::Table(bbb_table),
        );

        // Add aaa_table to root table
        document.insert(
            Key::new(KeyKind::BareKey, "aaa".to_string()),
            Value::Table(aaa_table),
        );

        tracing::trace!("document: {document:?}");

        // Test to_string method
        let toml_string = crate::to_string_async(&document).await.unwrap();
        let expected = r#"
[aaa.bbb]
ddd = "value1"

[aaa.bbb.inline]
x = 1
y = 2

[aaa.bbb.ccc]
value = "deep nested"

[[aaa.bbb.ccc.items]]
id = 1

[[aaa.bbb.ccc.items]]
id = 2
"#;
        toml_text_assert_eq!(toml_string, expected);
    }

    #[tokio::test]
    async fn test_date_time_serialization() {
        tombi_test_lib::init_tracing();

        let mut document = Document::new();

        let now =
            OffsetDateTime::from_ymd_hms(2024, 1, 1, 0, 0, 0, tombi_document::TimeZoneOffset::Z);
        document.insert(
            Key::new(KeyKind::BareKey, "now".to_string()),
            Value::OffsetDateTime(now),
        );

        let toml_string = crate::to_string_async(&document).await.unwrap();
        let expected = r#"
now = 2024-01-01T00:00:00Z
"#;
        toml_text_assert_eq!(toml_string, expected);
    }
}
