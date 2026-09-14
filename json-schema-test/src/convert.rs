use serde_json::Value as JsonValue;

/// Returns true when `value` can be represented as TOML under Definition A.
///
/// Definition A requires an object root (TOML documents are tables) and no
/// `null` anywhere in the instance tree.
pub fn is_toml_representable(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => false,
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => true,
        JsonValue::Array(items) => items.iter().all(is_toml_representable),
        JsonValue::Object(map) => map.values().all(is_toml_representable),
    }
}

pub fn supports_instance(data: &JsonValue) -> bool {
    matches!(data, JsonValue::Object(_)) && is_toml_representable(data)
}

pub fn json_object_to_toml_document(object: &serde_json::Map<String, JsonValue>) -> String {
    let mut out = String::new();
    for (key, value) in object {
        out.push_str(&toml_key(key));
        out.push_str(" = ");
        out.push_str(&json_value_to_toml(value));
        out.push('\n');
    }
    out
}

fn json_value_to_toml(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => unreachable!("null values are filtered by support checks"),
        JsonValue::Bool(boolean) => boolean.to_string(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::String(string) => serde_json::to_string(string).expect("string must serialize"),
        JsonValue::Array(items) => {
            let values = items.iter().map(json_value_to_toml).collect::<Vec<_>>();
            format!("[{}]", values.join(", "))
        }
        JsonValue::Object(object) => {
            if object.is_empty() {
                return "{}".to_string();
            }
            let entries = object
                .iter()
                .map(|(key, value)| format!("{} = {}", toml_key(key), json_value_to_toml(value)))
                .collect::<Vec<_>>();
            format!("{{ {} }}", entries.join(", "))
        }
    }
}

fn toml_key(key: &str) -> String {
    if !key.is_empty()
        && key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        key.to_string()
    } else {
        serde_json::to_string(key).expect("key must serialize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_null_and_non_object_roots() {
        assert!(!supports_instance(&json!(null)));
        assert!(!supports_instance(&json!([])));
        assert!(!supports_instance(&json!("x")));
        assert!(!supports_instance(&json!({"a": null})));
        assert!(supports_instance(&json!({"a": 1, "b": {"c": true}})));
    }

    #[test]
    fn converts_object_to_toml() {
        let object = json!({"foo": 1, "bar": {"baz": "qux"}, "arr": [1, true]})
            .as_object()
            .unwrap()
            .clone();
        let toml = json_object_to_toml_document(&object);
        assert!(toml.contains("foo = 1"));
        assert!(toml.contains("bar = { baz = \"qux\" }"));
        assert!(toml.contains("arr = [1, true]"));
    }
}
