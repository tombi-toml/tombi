pub(crate) fn table_to_json_object(table: &tombi_document_tree::Table) -> tombi_json_value::Object {
    let mut object = tombi_json_value::Object::new();
    for (key, value) in table.key_values() {
        object.insert(key.value.clone(), value_to_json_value(value));
    }
    object
}

pub(crate) fn value_to_json_value(value: &tombi_document_tree::Value) -> tombi_json_value::Value {
    match value {
        tombi_document_tree::Value::Boolean(b) => tombi_json_value::Value::Bool(b.value()),
        tombi_document_tree::Value::Integer(i) => tombi_json_value::Value::Number(i.value().into()),
        tombi_document_tree::Value::Float(f) => tombi_json_value::Value::Number(f.value().into()),
        tombi_document_tree::Value::String(s) => {
            tombi_json_value::Value::String(s.value().to_string())
        }
        tombi_document_tree::Value::OffsetDateTime(dt) => {
            tombi_json_value::Value::String(dt.value().to_string())
        }
        tombi_document_tree::Value::LocalDateTime(dt) => {
            tombi_json_value::Value::String(dt.value().to_string())
        }
        tombi_document_tree::Value::LocalDate(d) => {
            tombi_json_value::Value::String(d.value().to_string())
        }
        tombi_document_tree::Value::LocalTime(t) => {
            tombi_json_value::Value::String(t.value().to_string())
        }
        tombi_document_tree::Value::Array(a) => {
            tombi_json_value::Value::Array(a.values().iter().map(value_to_json_value).collect())
        }
        tombi_document_tree::Value::Table(t) => {
            tombi_json_value::Value::Object(table_to_json_object(t))
        }
        tombi_document_tree::Value::Incomplete { .. } => tombi_json_value::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tombi_document_tree::TryIntoDocumentTree;

    fn parse_toml_to_table(source: &str) -> tombi_document_tree::Table {
        let root = tombi_parser::parse(source)
            .try_into_root()
            .expect("TOML parse error");
        let tree: tombi_document_tree::DocumentTree = root
            .try_into_document_tree(Default::default())
            .expect("document tree error");
        tree.into()
    }

    #[test]
    fn test_boolean() {
        let table = parse_toml_to_table("flag = true\n");
        let obj = table_to_json_object(&table);
        assert_eq!(obj.get("flag"), Some(&tombi_json_value::Value::Bool(true)));
    }

    #[test]
    fn test_integer() {
        let table = parse_toml_to_table("count = 42\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("count"),
            Some(&tombi_json_value::Value::Number(42i64.into()))
        );
    }

    #[test]
    fn test_negative_integer() {
        let table = parse_toml_to_table("temp = -10\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("temp"),
            Some(&tombi_json_value::Value::Number((-10i64).into()))
        );
    }

    #[test]
    fn test_float() {
        let table = parse_toml_to_table("pi = 3.14\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("pi"),
            Some(&tombi_json_value::Value::Number((314_f64 / 100.0).into()))
        );
    }

    #[test]
    fn test_string() {
        let table = parse_toml_to_table("name = \"hello\"\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("name"),
            Some(&tombi_json_value::Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_offset_date_time() {
        let table = parse_toml_to_table("ts = 1979-05-27T07:32:00Z\n");
        let obj = table_to_json_object(&table);
        let value = obj.get("ts").expect("key 'ts' not found");
        match value {
            tombi_json_value::Value::String(s) => {
                assert!(s.contains("1979-05-27"), "unexpected: {s}");
                assert!(s.contains("07:32:00"), "unexpected: {s}");
            }
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn test_local_date_time() {
        let table = parse_toml_to_table("ts = 1979-05-27T07:32:00\n");
        let obj = table_to_json_object(&table);
        let value = obj.get("ts").expect("key 'ts' not found");
        match value {
            tombi_json_value::Value::String(s) => {
                assert!(s.contains("1979-05-27"), "unexpected: {s}");
                assert!(s.contains("07:32:00"), "unexpected: {s}");
            }
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn test_local_date() {
        let table = parse_toml_to_table("d = 2023-01-15\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("d"),
            Some(&tombi_json_value::Value::String("2023-01-15".to_string()))
        );
    }

    #[test]
    fn test_local_time() {
        let table = parse_toml_to_table("t = 14:30:00\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("t"),
            Some(&tombi_json_value::Value::String("14:30:00".to_string()))
        );
    }

    #[test]
    fn test_array() {
        let table = parse_toml_to_table("items = [1, 2, 3]\n");
        let obj = table_to_json_object(&table);
        assert_eq!(
            obj.get("items"),
            Some(&tombi_json_value::Value::Array(vec![
                tombi_json_value::Value::Number(1i64.into()),
                tombi_json_value::Value::Number(2i64.into()),
                tombi_json_value::Value::Number(3i64.into()),
            ]))
        );
    }

    #[test]
    fn test_nested_table() {
        let table = parse_toml_to_table("[inner]\nkey = \"val\"\n");
        let obj = table_to_json_object(&table);
        let inner = obj.get("inner").expect("key 'inner' not found");
        match inner {
            tombi_json_value::Value::Object(inner_obj) => {
                assert_eq!(
                    inner_obj.get("key"),
                    Some(&tombi_json_value::Value::String("val".to_string()))
                );
            }
            other => panic!("expected Object, got {other:?}"),
        }
    }

    #[test]
    fn test_mixed_types() {
        let source = r#"
name = "test"
count = 5
active = true
score = 9.5
created = 2024-06-01
tags = ["a", "b"]
"#;
        let table = parse_toml_to_table(source);
        let obj = table_to_json_object(&table);
        assert_eq!(obj.len(), 6);
        assert_eq!(
            obj.get("name"),
            Some(&tombi_json_value::Value::String("test".to_string()))
        );
        assert_eq!(
            obj.get("count"),
            Some(&tombi_json_value::Value::Number(5i64.into()))
        );
        assert_eq!(
            obj.get("active"),
            Some(&tombi_json_value::Value::Bool(true))
        );
        assert_eq!(
            obj.get("score"),
            Some(&tombi_json_value::Value::Number(9.5f64.into()))
        );
        assert_eq!(
            obj.get("created"),
            Some(&tombi_json_value::Value::String("2024-06-01".to_string()))
        );
        assert_eq!(
            obj.get("tags"),
            Some(&tombi_json_value::Value::Array(vec![
                tombi_json_value::Value::String("a".to_string()),
                tombi_json_value::Value::String("b".to_string()),
            ]))
        );
    }

    #[test]
    fn test_empty_table() {
        let table = parse_toml_to_table("");
        let obj = table_to_json_object(&table);
        assert!(obj.is_empty());
    }

    #[test]
    fn test_deeply_nested() {
        let source = "[a]\n[a.b]\n[a.b.c]\nval = 1\n";
        let table = parse_toml_to_table(source);
        let obj = table_to_json_object(&table);
        let a = obj.get("a").unwrap().as_object().unwrap();
        let b = a.get("b").unwrap().as_object().unwrap();
        let c = b.get("c").unwrap().as_object().unwrap();
        assert_eq!(
            c.get("val"),
            Some(&tombi_json_value::Value::Number(1i64.into()))
        );
    }
}
