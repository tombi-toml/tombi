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

pub fn log_keyword_dialect_notes(
    object: &tombi_json::ObjectNode,
    dialect: Option<JsonSchemaDialect>,
) {
    let dialect = dialect.unwrap_or_default();
    for (key, value) in &object.properties {
        let keyword = key.value.as_str();

        if is_deprecated_in_dialect(dialect, keyword) {
            let hint = replacement_hint(keyword).unwrap_or("No replacement hint.");
            let compat = if keyword == "dependencies" {
                " compat=legacy-keyword-parsed"
            } else {
                ""
            };
            log::debug!(
                "deprecated-json-schema-keyword: dialect={} keyword={} hint={}{}",
                dialect,
                keyword,
                hint,
                compat
            );
        }

        if keyword == "items"
            && matches!(dialect, JsonSchemaDialect::Draft2020_12)
            && matches!(value, tombi_json::ValueNode::Array(_))
        {
            let hint = replacement_hint("tuple-items").unwrap_or("No replacement hint.");
            log::debug!(
                "deprecated-json-schema-keyword: dialect={} keyword=tuple-items hint={}",
                dialect,
                hint
            );
        }

        if keyword_vocabulary(keyword).is_some() && !supports_keyword(dialect, keyword) {
            log::debug!(
                "unsupported-json-schema-keyword: dialect={} keyword={}",
                dialect,
                keyword
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedKeywordUsage {
    pub keyword: String,
    pub pointer: String,
    pub replacement_hint: Option<&'static str>,
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
        collect_deprecated_keyword_usages, is_deprecated_in_dialect, replacement_hint,
        supports_keyword,
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
}
