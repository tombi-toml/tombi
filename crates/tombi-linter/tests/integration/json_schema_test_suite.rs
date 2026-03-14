use std::fs;

use itertools::Either;
use serde_json::Value as JsonValue;
use tempfile::tempdir;
use tombi_config::TomlVersion;
use tombi_linter::{LintOptions, Linter};
use tombi_schema_store::{
    AssociateSchemaOptions, Options as SchemaStoreOptions, SchemaStore, SchemaUri,
};
use tombi_severity_level::SeverityLevel;

fn normalize_toml_text(input: &str) -> String {
    let mut toml_text = textwrap::dedent(input).trim().to_string();
    if !toml_text.is_empty() {
        toml_text.push('\n');
    }
    toml_text
}

async fn validate_test_suite(
    schema: JsonValue,
    toml_text: &str,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
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

    linter.lint(&toml_text).await
}

macro_rules! suite_test {
    (#[tokio::test] async fn $name:ident(
        $data:expr,
        JsonSchema($schema:expr) $(,)?
    ) -> Ok(_);) => {
        #[tokio::test]
        async fn $name() {
            tombi_test_lib::init_log();
            let toml_text = normalize_toml_text($data);
            match validate_test_suite($schema, &toml_text).await {
                Ok(_) => {}
                Err(errors) => {
                    pretty_assertions::assert_eq!(
                        errors,
                        Vec::<tombi_diagnostic::Diagnostic>::new(),
                        "expected success but got errors"
                    );
                }
            }
        }
    };

    (#[tokio::test] async fn $name:ident(
        $data:expr,
        JsonSchema($schema:expr) $(,)?
    ) -> Err($errors:expr);) => {
        #[tokio::test]
        async fn $name() {
            tombi_test_lib::init_log();
            let toml_text = normalize_toml_text($data);
            match validate_test_suite($schema, &toml_text).await {
                Ok(_) => panic!("expected error but got success"),
                Err(errs) => {
                    let mut expected = Vec::new();
                    for diagnostic in $errors {
                        diagnostic.push_diagnostic_with_level(SeverityLevel::Error, &mut expected);
                    }
                    pretty_assertions::assert_eq!(errs, expected);
                }
            }
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
                r#"

                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependant(
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependency(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                r#"
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "bar".to_string(),
                        required_key: "foo".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
        );
    }

    mod empty_array {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"dependencies": {"bar": []}})
        }

        suite_test!(
            #[tokio::test] async fn empty_object(
                r#"

                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn object_with_one_property(
                r#"
                bar = 2
                "#,
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
                r#"

                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependants(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependencies(
                r#"
                foo = 1
                bar = 2
                quux = 3
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                r#"
                foo = 1
                quux = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "bar".to_string()
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn missing_other_dependency(
                r#"
                bar = 1
                quux = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "foo".to_string()
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn missing_both_dependencies(
                r#"
                quux = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "foo".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
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
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                r#"
                foo = "quux"
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type(
                r#"
                foo = "quux"
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((0, 6), (0, 12))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_other(
                r#"
                foo = 2
                bar = "quux"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((1, 6), (1, 12))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_both(
                r#"
                foo = "quux"
                bar = "quux"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((0, 6), (0, 12))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((1, 6), (1, 12))
                ),
            ]);
        );
    }

    mod boolean_subschemas {
        use super::*;

        fn schema() -> JsonValue {
            serde_json::json!({"dependencies": {"foo": true, "bar": false}})
        }

        suite_test!(
            #[tokio::test] async fn schema_true_is_valid(
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn schema_false_is_invalid(
                r#"
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Nothing,
                    ((0, 0), (1, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn both_properties_is_invalid(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Nothing,
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_is_valid(
                r#"

                "#,
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
                r#"
                "foo\nbar" = 1
                "foo\rbar" = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn valid_object_2(
                r#"
                "foo\tbar" = 1
                a = 2
                b = 3
                c = 4
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn valid_object_3(
                r#"
                "foo'bar" = 1
                "foo\"bar" = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_1(
                r#"
                "foo\nbar" = 1
                foo = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "foo\nbar".to_string(),
                        required_key: "foo\rbar".to_string()
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_2(
                r#"
                "foo\tbar" = 1
                a = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableMinKeys {
                        min_keys: 4,
                        actual: 2
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_3(
                r#"
                "foo'bar" = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableKeyRequired {
                        key: "foo\"bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn invalid_object_4(
                r#"
                "foo\"bar" = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "foo\"bar".to_string(),
                        required_key: "foo'bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::KeyNotAllowed {
                        key: "foo".to_string()
                    },
                    ((0, 0), (0, 7))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn matches_dependency(
                r#"
                bar = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn matches_both(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::KeyNotAllowed {
                        key: "foo".to_string()
                    },
                    ((0, 0), (0, 7))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::KeyNotAllowed {
                        key: "bar".to_string()
                    },
                    ((1, 0), (1, 7))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                r#"
                baz = 1
                "#,
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
                r#"
                f = {  }
                foo = {  }
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn some_property_names_invalid(
                r#"
                foo = {  }
                foobar = {  }
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::StringMaxLength {
                        maximum: 3,
                        actual: 6
                    },
                    ((1, 0), (1, 6))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn object_without_properties_is_valid(
                r#"

                "#,
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
                r#"
                a = {  }
                aa = {  }
                aaa = {  }
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn non_matching_invalid(
                r#"
                aaA = {  }
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::StringPattern {
                        pattern: "^a+$".to_string(),
                        actual: "aaA".to_string()
                    },
                    ((0, 0), (0, 3))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                r#"

                "#,
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                r#"

                "#,
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Nothing,
                    ((0, 0), (1, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                r#"

                "#,
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn other_property_invalid(
                r#"
                bar = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Const {
                        expected: "\"foo\"".to_string(),
                        actual: "bar".to_string()
                    },
                    ((0, 0), (0, 3))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                r#"

                "#,
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn foo_and_bar_valid(
                r#"
                foo = 1
                bar = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn other_property_invalid(
                r#"
                baz = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Enum {
                        expected: vec!["\"foo\"".to_string(), "\"bar\"".to_string()],
                        actual: "baz".to_string()
                    },
                    ((0, 0), (0, 3))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                r#"

                "#,
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
                r#""#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_unevaluated_properties(
                r#"
                foo = "foo"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
                        key: "foo".to_string()
                    },
                    ((0, 0), (0, 11))
                ),
            ]);
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
                r#"
                foo = "foo"
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_unevaluated_properties(
                r#"
                foo = "foo"
                bar = "bar"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
                        key: "bar".to_string()
                    },
                    ((1, 0), (1, 11))
                ),
            ]);
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
                r#"
                foo = "foo"
                bar = "bar"
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_additional_properties(
                r#"
                foo = "foo"
                bar = "bar"
                baz = "baz"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
                        key: "bar".to_string()
                    },
                    ((1, 0), (1, 11))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
                        key: "baz".to_string()
                    },
                    ((2, 0), (2, 11))
                ),
            ]);
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
                r#"
                foo = "a"
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn invalid_in_case_if_is_evaluated(
                r#"
                bar = "a"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
                        key: "bar".to_string()
                    },
                    ((0, 0), (0, 9))
                ),
            ]);
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
                r#"

                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependant(
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependency(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                r#"
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "bar".to_string(),
                        required_key: "foo".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
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
                r#"

                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn object_with_one_property(
                r#"
                bar = 2
                "#,
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
                r#"

                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn nondependants(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn with_dependencies(
                r#"
                foo = 1
                bar = 2
                quux = 3
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn missing_dependency(
                r#"
                foo = 1
                quux = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "bar".to_string()
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn missing_other_dependency(
                r#"
                bar = 1
                quux = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "foo".to_string()
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn missing_both_dependencies(
                r#"
                quux = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "foo".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "quux".to_string(),
                        required_key: "bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
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
                r#"
                "foo\nbar" = 1
                "foo\rbar" = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quotes(
                r#"
                "foo'bar" = 1
                "foo\"bar" = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn crlf_missing_dependent(
                r#"
                "foo\nbar" = 1
                foo = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "foo\nbar".to_string(),
                        required_key: "foo\rbar".to_string()
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quotes_missing_dependent(
                r#"
                "foo\"bar" = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableDependencyRequired {
                        dependent_key: "foo\"bar".to_string(),
                        required_key: "foo'bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
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
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                r#"
                foo = "quux"
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type(
                r#"
                foo = "quux"
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((0, 6), (0, 12))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_other(
                r#"
                foo = 2
                bar = "quux"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((1, 6), (1, 12))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn wrong_type_both(
                r#"
                foo = "quux"
                bar = "quux"
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((0, 6), (0, 12))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TypeMismatch {
                        expected: tombi_schema_store::ValueType::Integer,
                        actual: tombi_document_tree::ValueType::String
                    },
                    ((1, 6), (1, 12))
                ),
            ]);
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn schema_false_invalid(
                r#"
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Nothing,
                    ((0, 0), (1, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn both_properties_invalid(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::Nothing,
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn empty_object_valid(
                r#"

                "#,
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
                r#"
                "foo\tbar" = 1
                a = 2
                b = 3
                c = 4
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quote(
                r#"
                "foo'bar" = { "foo\"bar" = 1 }
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableKeyRequired {
                        key: "foo\"bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn quoted_tab_invalid(
                r#"
                "foo\tbar" = 1
                a = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableMinKeys {
                        min_keys: 4,
                        actual: 2
                    },
                    ((0, 0), (2, 0))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn quoted_quote_invalid(
                r#"
                "foo'bar" = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::TableKeyRequired {
                        key: "foo\"bar".to_string()
                    },
                    ((0, 0), (1, 0))
                ),
            ]);
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
                r#"
                foo = 1
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::KeyNotAllowed {
                        key: "foo".to_string()
                    },
                    ((0, 0), (0, 7))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn matches_dependency(
                r#"
                bar = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );

        suite_test!(
            #[tokio::test] async fn matches_both(
                r#"
                foo = 1
                bar = 2
                "#,
                JsonSchema(schema()),
            ) -> Err([
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::KeyNotAllowed {
                        key: "foo".to_string()
                    },
                    ((0, 0), (0, 7))
                ),
                tombi_validator::Diagnostic::new(
                    tombi_validator::DiagnosticKind::KeyNotAllowed {
                        key: "bar".to_string()
                    },
                    ((1, 0), (1, 7))
                ),
            ]);
        );

        suite_test!(
            #[tokio::test] async fn no_dependency(
                r#"
                baz = 1
                "#,
                JsonSchema(schema()),
            ) -> Ok(_);
        );
    }
}
