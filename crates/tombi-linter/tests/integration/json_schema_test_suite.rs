use std::fs;

use itertools::Either;
use serde_json::Value as JsonValue;
use tempfile::tempdir;
use tombi_config::TomlVersion;
use tombi_diagnostic::Level;
use tombi_linter::{LintOptions, Linter};
use tombi_schema_store::{
    AssociateSchemaOptions, Options as SchemaStoreOptions, SchemaStore, SchemaUri,
};

fn json_object_to_toml_document(object: &serde_json::Map<String, JsonValue>) -> String {
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
            let entries = object
                .iter()
                .map(|(key, value)| format!("{} = {}", toml_key(key), json_value_to_toml(value)))
                .collect::<Vec<_>>();
            format!("{{ {} }}", entries.join(", "))
        }
    }
}

fn toml_key(key: &str) -> String {
    if key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !key.is_empty()
    {
        key.to_string()
    } else {
        serde_json::to_string(key).expect("key must serialize")
    }
}

async fn assert_suite_validation(schema: JsonValue, data: JsonValue, expected_valid: bool) {
    let toml_text = json_object_to_toml_document(
        data.as_object()
            .expect("suite test data must be a JSON object"),
    );

    let temp = tempdir().expect("failed to create temp directory");
    let schema_path = temp.path().join("schema.json");
    let source_path = temp.path().join("test.toml");

    fs::write(&schema_path, serde_json::to_vec_pretty(&schema).unwrap()).unwrap();
    fs::write(&source_path, &toml_text).unwrap();

    let schema_store = SchemaStore::new_with_options(SchemaStoreOptions {
        strict: Some(false),
        offline: Some(true),
        cache: None,
    });
    let schema_uri = SchemaUri::from_file_path(&schema_path)
        .expect("failed to convert suite schema path to schema uri");
    schema_store
        .associate_schema(
            schema_uri,
            vec!["*.toml".to_string()],
            &AssociateSchemaOptions::default(),
        )
        .await;

    let lint_options = LintOptions::default();
    let linter = Linter::new(
        TomlVersion::default(),
        &lint_options,
        Some(Either::Right(source_path.as_path())),
        &schema_store,
    );

    let actual_valid = match linter.lint(&toml_text).await {
        Ok(()) => true,
        Err(diagnostics) => diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level() != Level::ERROR),
    };

    assert_eq!(
        actual_valid, expected_valid,
        "expected valid={expected_valid}, got valid={actual_valid}\nschema: {schema}\ntoml:\n{toml_text}"
    );
}

macro_rules! suite_test {
    (#[tokio::test] async fn $name:ident(
        $data:tt,
        JsonSchema($schema:expr) $(,)?
    ) -> Ok(_);) => {
        #[tokio::test]
        async fn $name() {
            tombi_test_lib::init_log();
            assert_suite_validation($schema, serde_json::json!($data), true).await;
        }
    };
    (#[tokio::test] async fn $name:ident(
        $data:tt,
        JsonSchema($schema:expr) $(,)?
    ) -> Err(_);) => {
        #[tokio::test]
        async fn $name() {
            tombi_test_lib::init_log();
            assert_suite_validation($schema, serde_json::json!($data), false).await;
        }
    };
}

// =============================================================================
// Draft 7: dependencies
// =============================================================================
mod draft7_dependencies {
    use super::*;

    mod basic {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"dependencies": {"bar": ["foo"]}})
        }

        suite_test!(
            #[tokio::test] async fn neither(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependant(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependency(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                {"bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod empty_array {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"dependencies": {"bar": []}})
        }

        suite_test!(
            #[tokio::test] async fn empty_object(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn object_with_one_property(
                {"bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod multiple {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"dependencies": {"quux": ["foo", "bar"]}})
        }

        suite_test!(
            #[tokio::test] async fn neither(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependants(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependencies(
                {"foo": 1, "bar": 2, "quux": 3},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                {"foo": 1, "quux": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_other_dependency(
                {"bar": 1, "quux": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_both_dependencies(
                {"quux": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod subschema {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "dependencies": {
                    "bar": {
                        "properties": {
                            "foo": {"type": "integer"},
                            "bar": {"type": "integer"}
                        }
                    }
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn valid(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                {"foo": "quux"},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type(
                {"foo": "quux", "bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_other(
                {"foo": 2, "bar": "quux"},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_both(
                {"foo": "quux", "bar": "quux"},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod boolean_subschemas {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"dependencies": {"foo": true, "bar": false}})
        }

        suite_test!(
            #[tokio::test] async fn schema_true_is_valid(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn schema_false_is_invalid(
                {"bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn both_properties_is_invalid(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_is_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod escaped_characters {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "dependencies": {
                    "foo\nbar": ["foo\rbar"],
                    "foo\tbar": {"minProperties": 4},
                    "foo'bar": {"required": ["foo\"bar"]},
                    "foo\"bar": ["foo'bar"]
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn valid_object_1(
                {"foo\nbar": 1, "foo\rbar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn valid_object_2(
                {"foo\tbar": 1, "a": 2, "b": 3, "c": 4},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn valid_object_3(
                {"foo'bar": 1, "foo\"bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_1(
                {"foo\nbar": 1, "foo": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_2(
                {"foo\tbar": 1, "a": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_3(
                {"foo'bar": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_4(
                {"foo\"bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod incompatible_with_root {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "properties": {"foo": {}},
                "dependencies": {
                    "foo": {
                        "properties": {"bar": {}},
                        "additionalProperties": false
                    }
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn matches_root(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn matches_dependency(
                {"bar": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn matches_both(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                {"baz": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }
}

// =============================================================================
// Draft 7: propertyNames
// =============================================================================
mod draft7_property_names {
    use super::*;

    mod validation {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"propertyNames": {"maxLength": 3}})
        }

        suite_test!(
            #[tokio::test] async fn all_property_names_valid(
                {"f": {}, "foo": {}},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn some_property_names_invalid(
                {"foo": {}, "foobar": {}},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn object_without_properties_is_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod with_pattern {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"propertyNames": {"pattern": "^a+$"}})
        }

        suite_test!(
            #[tokio::test] async fn matching_valid(
                {"a": {}, "aa": {}, "aaa": {}},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn non_matching_invalid(
                {"aaA": {}},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod boolean_schema_true {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"propertyNames": true})
        }

        suite_test!(
            #[tokio::test] async fn any_properties_valid(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod boolean_schema_false {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"propertyNames": false})
        }

        suite_test!(
            #[tokio::test] async fn any_properties_invalid(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod with_const {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"propertyNames": {"const": "foo"}})
        }

        suite_test!(
            #[tokio::test] async fn foo_valid(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn other_property_invalid(
                {"bar": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod with_enum {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"propertyNames": {"enum": ["foo", "bar"]}})
        }

        suite_test!(
            #[tokio::test] async fn foo_valid(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn foo_and_bar_valid(
                {"foo": 1, "bar": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn other_property_invalid(
                {"baz": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }
}

// =============================================================================
// Draft 2019-09: unevaluatedProperties
// =============================================================================
mod draft2019_09_unevaluated_properties {
    use super::*;

    mod unevaluated_properties_false {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2019-09/schema",
                "type": "object",
                "unevaluatedProperties": false
            })
        }

        suite_test!(
            #[tokio::test] async fn no_unevaluated_properties(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_unevaluated_properties(
                {"foo": "foo"},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod with_adjacent_properties {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2019-09/schema",
                "type": "object",
                "properties": {"foo": {"type": "string"}},
                "unevaluatedProperties": false
            })
        }

        suite_test!(
            #[tokio::test] async fn no_unevaluated_properties(
                {"foo": "foo"},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_unevaluated_properties(
                {"foo": "foo", "bar": "bar"},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod with_nested_properties {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2019-09/schema",
                "type": "object",
                "properties": {"foo": {"type": "string"}},
                "allOf": [{"properties": {"bar": {"type": "string"}}}],
                "unevaluatedProperties": false
            })
        }

        suite_test!(
            #[tokio::test] async fn no_additional_properties(
                {"foo": "foo", "bar": "bar"},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_additional_properties(
                {"foo": "foo", "bar": "bar", "baz": "baz"},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod if_without_then_and_else {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2019-09/schema",
                "if": {"patternProperties": {"foo": {"type": "string"}}},
                "unevaluatedProperties": false
            })
        }

        suite_test!(
            #[tokio::test] async fn valid_in_case_if_is_evaluated(
                {"foo": "a"},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_in_case_if_is_evaluated(
                {"bar": "a"},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }
}

// =============================================================================
// Draft 2020-12: dependentRequired
// =============================================================================
mod draft2020_12_dependent_required {
    use super::*;

    mod single {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentRequired": {"bar": ["foo"]}
            })
        }

        suite_test!(
            #[tokio::test] async fn neither(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependant(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependency(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                {"bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod empty {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentRequired": {"bar": []}
            })
        }

        suite_test!(
            #[tokio::test] async fn empty_object(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn object_with_one_property(
                {"bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod multiple {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentRequired": {"quux": ["foo", "bar"]}
            })
        }

        suite_test!(
            #[tokio::test] async fn neither(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependants(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependencies(
                {"foo": 1, "bar": 2, "quux": 3},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                {"foo": 1, "quux": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_other_dependency(
                {"bar": 1, "quux": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_both_dependencies(
                {"quux": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod escaped_characters {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentRequired": {
                    "foo\nbar": ["foo\rbar"],
                    "foo\"bar": ["foo'bar"]
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn crlf(
                {"foo\nbar": 1, "foo\rbar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quotes(
                {"foo'bar": 1, "foo\"bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn crlf_missing_dependent(
                {"foo\nbar": 1, "foo": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quotes_missing_dependent(
                {"foo\"bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }
}

// =============================================================================
// Draft 2020-12: dependentSchemas
// =============================================================================
mod draft2020_12_dependent_schemas {
    use super::*;

    mod single {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentSchemas": {
                    "bar": {
                        "properties": {
                            "foo": {"type": "integer"},
                            "bar": {"type": "integer"}
                        }
                    }
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn valid(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                {"foo": "quux"},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type(
                {"foo": "quux", "bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_other(
                {"foo": 2, "bar": "quux"},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_both(
                {"foo": "quux", "bar": "quux"},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod boolean_subschemas {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentSchemas": {"foo": true, "bar": false}
            })
        }

        suite_test!(
            #[tokio::test] async fn schema_true_valid(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn schema_false_invalid(
                {"bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn both_properties_invalid(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                {},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }

    mod escaped_characters {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "dependentSchemas": {
                    "foo\tbar": {"minProperties": 4},
                    "foo'bar": {"required": ["foo\"bar"]}
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn quoted_tab(
                {"foo\tbar": 1, "a": 2, "b": 3, "c": 4},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quote(
                {"foo'bar": {"foo\"bar": 1}},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_tab_invalid(
                {"foo\tbar": 1, "a": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quote_invalid(
                {"foo'bar": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );
    }

    mod incompatible_with_root {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "properties": {"foo": {}},
                "dependentSchemas": {
                    "foo": {
                        "properties": {"bar": {}},
                        "additionalProperties": false
                    }
                }
            })
        }

        suite_test!(
            #[tokio::test] async fn matches_root(
                {"foo": 1},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn matches_dependency(
                {"bar": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn matches_both(
                {"foo": 1, "bar": 2},
                JsonSchema(schema()),
            ) -> Err(_);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                {"baz": 1},
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }
}
