use crate::JsonSchemaDialect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonSchemaVocabulary {
    Core,
    Applicator,
    Validation,
    Unevaluated,
    MetaData,
    Format,
    Content,
}

pub fn keyword_vocabulary(keyword: &str) -> Option<JsonSchemaVocabulary> {
    match keyword {
        "$id" | "$schema" | "$ref" | "$defs" | "definitions" | "$anchor" | "$dynamicRef"
        | "$dynamicAnchor" | "$recursiveRef" | "$recursiveAnchor" | "$vocabulary" => {
            Some(JsonSchemaVocabulary::Core)
        }
        "allOf"
        | "anyOf"
        | "oneOf"
        | "not"
        | "if"
        | "then"
        | "else"
        | "items"
        | "additionalItems"
        | "prefixItems"
        | "contains"
        | "properties"
        | "patternProperties"
        | "additionalProperties"
        | "propertyNames"
        | "dependentSchemas" => Some(JsonSchemaVocabulary::Applicator),
        "type" | "enum" | "const" | "multipleOf" | "maximum" | "minimum" | "exclusiveMaximum"
        | "exclusiveMinimum" | "maxLength" | "minLength" | "pattern" | "maxItems" | "minItems"
        | "uniqueItems" | "maxProperties" | "minProperties" | "required" | "dependencies"
        | "dependentRequired" | "maxContains" | "minContains" => {
            Some(JsonSchemaVocabulary::Validation)
        }
        "unevaluatedProperties" | "unevaluatedItems" => Some(JsonSchemaVocabulary::Unevaluated),
        "title" | "description" | "default" | "deprecated" | "readOnly" | "writeOnly"
        | "examples" | "$comment" => Some(JsonSchemaVocabulary::MetaData),
        "format" => Some(JsonSchemaVocabulary::Format),
        "contentEncoding" | "contentMediaType" | "contentSchema" => {
            Some(JsonSchemaVocabulary::Content)
        }
        _ => None,
    }
}

pub fn dialect_supports_vocabulary(
    dialect: JsonSchemaDialect,
    vocabulary: JsonSchemaVocabulary,
) -> bool {
    match dialect {
        JsonSchemaDialect::Draft07 => !matches!(vocabulary, JsonSchemaVocabulary::Unevaluated),
        JsonSchemaDialect::Draft2019_09 | JsonSchemaDialect::Draft2020_12 => true,
    }
}

pub fn supports_keyword(dialect: JsonSchemaDialect, keyword: &str) -> bool {
    match keyword {
        // Added in 2020-12
        "prefixItems" | "$dynamicRef" | "$dynamicAnchor" => {
            dialect >= JsonSchemaDialect::Draft2020_12
        }
        // Deprecated/removed in 2020-12
        "additionalItems" | "dependencies" => dialect < JsonSchemaDialect::Draft2020_12,
        "$recursiveRef" | "$recursiveAnchor" => dialect == JsonSchemaDialect::Draft2019_09,
        // Available since 2019-09
        "dependentRequired"
        | "dependentSchemas"
        | "unevaluatedProperties"
        | "unevaluatedItems"
        | "minContains"
        | "maxContains"
        | "$vocabulary"
        | "$anchor" => dialect > JsonSchemaDialect::Draft07,
        _ => keyword_vocabulary(keyword)
            .map(|v| dialect_supports_vocabulary(dialect, v))
            .unwrap_or(true),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedKeywordUsage {
    pub keyword: String,
    pub pointer: String,
    pub replacement_hint: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedVocabularyUsage {
    pub uri: String,
    pub required: bool,
    pub pointer: String,
    pub issue: Option<&'static str>,
}

pub fn deprecated_in(keyword: &str) -> Option<JsonSchemaDialect> {
    match keyword {
        "definitions" => Some(JsonSchemaDialect::Draft2019_09),
        "dependencies" | "additionalItems" | "$recursiveRef" | "$recursiveAnchor" => {
            Some(JsonSchemaDialect::Draft2020_12)
        }
        _ => None,
    }
}

pub fn replacement_hint(keyword: &str) -> Option<&'static str> {
    match keyword {
        "definitions" => Some("Use `$defs` instead."),
        "dependencies" => Some("Use `dependentRequired` / `dependentSchemas` instead."),
        "additionalItems" => Some("Use `prefixItems` + new `items` semantics instead."),
        "$recursiveRef" | "$recursiveAnchor" => {
            Some("Use `$dynamicRef` / `$dynamicAnchor` instead.")
        }
        "tuple-items" => Some("Use `prefixItems` instead of array-form `items`."),
        _ => None,
    }
}

pub fn is_deprecated_in_dialect(dialect: JsonSchemaDialect, keyword: &str) -> bool {
    match deprecated_in(keyword) {
        Some(deprecated_from) => dialect_rank(dialect) >= dialect_rank(deprecated_from),
        None => false,
    }
}

pub fn collect_deprecated_keyword_usages(
    root_object: &tombi_json::ObjectNode,
    dialect: JsonSchemaDialect,
) -> Vec<DeprecatedKeywordUsage> {
    let mut usages = Vec::new();
    collect_from_object_node(root_object, "#", dialect, &mut usages);
    usages
}

pub fn is_supported_vocabulary_uri(dialect: JsonSchemaDialect, uri: &str) -> bool {
    let normalized_uri = uri.trim_end_matches('#');
    let expected_prefix = match dialect {
        JsonSchemaDialect::Draft07 => return false,
        JsonSchemaDialect::Draft2019_09 => "https://json-schema.org/draft/2019-09/vocab/",
        JsonSchemaDialect::Draft2020_12 => "https://json-schema.org/draft/2020-12/vocab/",
    };

    if !normalized_uri.starts_with(expected_prefix) {
        return false;
    }

    let Some(segment) = normalized_uri.rsplit('/').next() else {
        return false;
    };

    match dialect {
        JsonSchemaDialect::Draft07 => false,
        JsonSchemaDialect::Draft2019_09 => matches!(
            segment,
            "core" | "applicator" | "validation" | "meta-data" | "format" | "content"
        ),
        JsonSchemaDialect::Draft2020_12 => matches!(
            segment,
            "core"
                | "applicator"
                | "validation"
                | "meta-data"
                | "format-annotation"
                | "format-assertion"
                | "content"
        ),
    }
}

pub fn collect_unsupported_vocabulary_usages(
    root_object: &tombi_json::ObjectNode,
    dialect: JsonSchemaDialect,
) -> Vec<UnsupportedVocabularyUsage> {
    let Some(vocabulary_value) = root_object.get("$vocabulary") else {
        return Vec::new();
    };
    let Some(vocabulary_object) = vocabulary_value.as_object() else {
        return vec![UnsupportedVocabularyUsage {
            uri: "$vocabulary".to_string(),
            required: false,
            pointer: "#/$vocabulary".to_string(),
            issue: Some("`$vocabulary` must be an object"),
        }];
    };

    vocabulary_object
        .properties
        .iter()
        .filter_map(|(uri, required_value)| {
            let pointer = format!(
                "#/$vocabulary/{}",
                escape_json_pointer_token(uri.value.as_str())
            );
            match required_value.as_bool() {
                Some(required) => {
                    if is_supported_vocabulary_uri(dialect, uri.value.as_str()) {
                        return None;
                    }

                    Some(UnsupportedVocabularyUsage {
                        uri: uri.value.to_string(),
                        required,
                        pointer,
                        issue: None,
                    })
                }
                None => Some(UnsupportedVocabularyUsage {
                    uri: uri.value.to_string(),
                    required: false,
                    pointer,
                    issue: Some("`$vocabulary` entries must be boolean"),
                }),
            }
        })
        .collect()
}

fn collect_from_object_node(
    object: &tombi_json::ObjectNode,
    pointer: &str,
    dialect: JsonSchemaDialect,
    usages: &mut Vec<DeprecatedKeywordUsage>,
) {
    for (key, value) in &object.properties {
        let keyword = key.value.as_str();
        let child_pointer = format!("{pointer}/{}", escape_json_pointer_token(keyword));

        if is_deprecated_in_dialect(dialect, keyword) {
            usages.push(DeprecatedKeywordUsage {
                keyword: keyword.to_string(),
                pointer: child_pointer.clone(),
                replacement_hint: replacement_hint(keyword),
            });
        }

        // draft-2020-12: array-form items is deprecated in favor of prefixItems.
        if keyword == "items"
            && matches!(dialect, JsonSchemaDialect::Draft2020_12)
            && matches!(value, tombi_json::ValueNode::Array(_))
        {
            usages.push(DeprecatedKeywordUsage {
                keyword: "tuple-items".to_string(),
                pointer: child_pointer.clone(),
                replacement_hint: replacement_hint("tuple-items"),
            });
        }

        match value {
            tombi_json::ValueNode::Object(object) => {
                collect_from_object_node(object, &child_pointer, dialect, usages);
            }
            tombi_json::ValueNode::Array(array) => {
                collect_from_array_node(array, &child_pointer, dialect, usages);
            }
            _ => {}
        }
    }
}

fn collect_from_array_node(
    array: &tombi_json::ArrayNode,
    pointer: &str,
    dialect: JsonSchemaDialect,
    usages: &mut Vec<DeprecatedKeywordUsage>,
) {
    for (idx, value) in array.items.iter().enumerate() {
        let child_pointer = format!("{pointer}/{idx}");
        match value {
            tombi_json::ValueNode::Object(object) => {
                collect_from_object_node(object, &child_pointer, dialect, usages);
            }
            tombi_json::ValueNode::Array(array) => {
                collect_from_array_node(array, &child_pointer, dialect, usages);
            }
            _ => {}
        }
    }
}

fn escape_json_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

fn dialect_rank(dialect: JsonSchemaDialect) -> u8 {
    match dialect {
        JsonSchemaDialect::Draft07 => 0,
        JsonSchemaDialect::Draft2019_09 => 1,
        JsonSchemaDialect::Draft2020_12 => 2,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::JsonSchemaDialect;

    use super::{
        collect_deprecated_keyword_usages, collect_unsupported_vocabulary_usages,
        is_deprecated_in_dialect, is_supported_vocabulary_uri, replacement_hint, supports_keyword,
    };

    #[test]
    fn draft_07_rejects_2020_12_only_keywords() {
        assert!(!supports_keyword(JsonSchemaDialect::Draft07, "prefixItems"));
        assert!(!supports_keyword(JsonSchemaDialect::Draft07, "$dynamicRef"));
    }

    #[test]
    fn draft_2020_12_rejects_deprecated_keywords() {
        assert!(!supports_keyword(
            JsonSchemaDialect::Draft2020_12,
            "dependencies"
        ));
        assert!(!supports_keyword(
            JsonSchemaDialect::Draft2020_12,
            "additionalItems"
        ));
        assert!(!supports_keyword(
            JsonSchemaDialect::Draft2020_12,
            "$recursiveRef"
        ));
    }

    #[test]
    fn draft_2019_09_accepts_dependent_keywords() {
        assert!(supports_keyword(
            JsonSchemaDialect::Draft2019_09,
            "dependentRequired"
        ));
        assert!(supports_keyword(
            JsonSchemaDialect::Draft2019_09,
            "dependentSchemas"
        ));
        assert!(supports_keyword(
            JsonSchemaDialect::Draft2019_09,
            "$recursiveAnchor"
        ));
    }

    #[test]
    fn draft_07_rejects_recursive_keywords() {
        assert!(!supports_keyword(
            JsonSchemaDialect::Draft07,
            "$recursiveRef"
        ));
    }

    #[test]
    fn deprecation_hook_handles_version_thresholds() {
        assert!(is_deprecated_in_dialect(
            JsonSchemaDialect::Draft2019_09,
            "definitions"
        ));
        assert!(!is_deprecated_in_dialect(
            JsonSchemaDialect::Draft07,
            "definitions"
        ));
        assert!(is_deprecated_in_dialect(
            JsonSchemaDialect::Draft2020_12,
            "dependencies"
        ));
    }

    #[test]
    fn deprecation_hook_collects_nested_keywords() {
        let value = tombi_json::ValueNode::from_str(
            r#"{ "properties": { "x": { "dependencies": { "a": ["b"] } } } }"#,
        )
        .expect("schema should parse");
        let object = value.as_object().expect("schema should be object");
        let usages = collect_deprecated_keyword_usages(object, JsonSchemaDialect::Draft2020_12);
        assert!(usages.iter().any(|u| u.keyword == "dependencies"));
    }

    #[test]
    fn deprecation_hook_provides_replacement_hint() {
        assert_eq!(
            replacement_hint("dependencies"),
            Some("Use `dependentRequired` / `dependentSchemas` instead.")
        );
    }

    #[test]
    fn supports_known_2019_09_vocabulary_uris() {
        assert!(is_supported_vocabulary_uri(
            JsonSchemaDialect::Draft2019_09,
            "https://json-schema.org/draft/2019-09/vocab/core"
        ));
        assert!(is_supported_vocabulary_uri(
            JsonSchemaDialect::Draft2019_09,
            "https://json-schema.org/draft/2019-09/vocab/format#"
        ));
        assert!(!is_supported_vocabulary_uri(
            JsonSchemaDialect::Draft2019_09,
            "https://example.com/custom?x=/draft/2019-09/vocab/core"
        ));
        assert!(!is_supported_vocabulary_uri(
            JsonSchemaDialect::Draft2019_09,
            "https://json-schema.org/draft/2019-09/vocab/unevaluated"
        ));
    }

    #[test]
    fn collects_unsupported_required_and_optional_vocabularies() {
        let value = tombi_json::ValueNode::from_str(
            r#"{
                "$vocabulary": {
                    "https://json-schema.org/draft/2019-09/vocab/core": true,
                    "https://example.com/vocab/custom-required": true,
                    "https://example.com/vocab/custom-optional": false
                }
            }"#,
        )
        .expect("schema should parse");
        let object = value.as_object().expect("schema should be object");

        let usages = collect_unsupported_vocabulary_usages(object, JsonSchemaDialect::Draft2019_09);
        assert_eq!(usages.len(), 2);
        assert!(usages.iter().any(
            |usage| usage.required && usage.uri == "https://example.com/vocab/custom-required"
        ));
        assert!(usages.iter().any(
            |usage| !usage.required && usage.uri == "https://example.com/vocab/custom-optional"
        ));
        assert!(usages.iter().all(|usage| usage.issue.is_none()));
    }

    #[test]
    fn collects_invalid_vocabulary_shapes() {
        let value = tombi_json::ValueNode::from_str(
            r#"{
                "$vocabulary": {
                    "https://example.com/vocab/custom-required": "yes"
                }
            }"#,
        )
        .expect("schema should parse");
        let object = value.as_object().expect("schema should be object");

        let usages = collect_unsupported_vocabulary_usages(object, JsonSchemaDialect::Draft2019_09);
        assert_eq!(usages.len(), 1);
        assert_eq!(
            usages[0].issue,
            Some("`$vocabulary` entries must be boolean")
        );
    }
}
