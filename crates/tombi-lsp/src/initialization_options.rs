use serde::{Deserialize, Deserializer, de::DeserializeOwned};

/// `initializationOptions` sent by the client in the `initialize` request.
///
/// Every item is optional. A missing, `null` or ill-typed item falls back to its default
/// (with a warning for an ill-typed one), so that `initialize` never fails because of it.
///
/// ```json
/// {
///   "semanticTokens": {
///     "strings": { "enabled": true },
///     "comments": { "enabled": true }
///   }
/// }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InitializationOptions {
    #[serde(deserialize_with = "lenient")]
    pub semantic_tokens: SemanticTokensSettings,
}

impl InitializationOptions {
    pub fn from_value(value: Option<serde_json::Value>) -> Self {
        match value {
            None | Some(serde_json::Value::Null) => Self::default(),
            Some(value) => lenient_value(value),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SemanticTokensSettings {
    /// Whether to send the semantic tokens of string values.
    #[serde(deserialize_with = "lenient")]
    pub strings: TokenKindSettings,
    /// Whether to send the semantic tokens of ordinary comments.
    #[serde(deserialize_with = "lenient")]
    pub comments: TokenKindSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TokenKindSettings {
    #[serde(deserialize_with = "lenient_enabled")]
    pub enabled: bool,
}

impl Default for TokenKindSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

fn lenient_value<T: DeserializeOwned + Default>(value: serde_json::Value) -> T {
    match serde_json::from_value(value) {
        Ok(value) => value,
        Err(error) => {
            log::warn!("invalid initializationOptions, using the default: {error}");
            T::default()
        }
    }
}

fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    Ok(
        match Option::<serde_json::Value>::deserialize(deserializer)? {
            None | Some(serde_json::Value::Null) => T::default(),
            Some(value) => lenient_value(value),
        },
    )
}

fn lenient_enabled<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(
        match Option::<serde_json::Value>::deserialize(deserializer)? {
            None | Some(serde_json::Value::Null) => true,
            Some(serde_json::Value::Bool(enabled)) => enabled,
            Some(value) => {
                log::warn!(
                    "invalid `enabled` in initializationOptions (expected a boolean): {value}"
                );
                true
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn enables(value: Option<serde_json::Value>) -> (bool, bool) {
        let settings = InitializationOptions::from_value(value).semantic_tokens;
        (settings.strings.enabled, settings.comments.enabled)
    }

    #[test]
    fn missing_or_null_enables_everything() {
        assert_eq!(enables(None), (true, true));
        assert_eq!(enables(Some(json!(null))), (true, true));
        assert_eq!(enables(Some(json!({}))), (true, true));
        assert_eq!(enables(Some(json!({"semanticTokens": null}))), (true, true));
        assert_eq!(
            enables(Some(
                json!({"semanticTokens": {"strings": null, "comments": {"enabled": null}}})
            )),
            (true, true)
        );
        assert_eq!(
            enables(Some(json!({"semanticTokens": {"strings": {}}}))),
            (true, true)
        );
    }

    #[test]
    fn partial_and_full_specification() {
        assert_eq!(
            enables(Some(
                json!({"semanticTokens": {"strings": {"enabled": false}}})
            )),
            (false, true)
        );
        assert_eq!(
            enables(Some(
                json!({"semanticTokens": {"comments": {"enabled": false}}})
            )),
            (true, false)
        );
        assert_eq!(
            enables(Some(
                json!({"semanticTokens": {"strings": {"enabled": false}, "comments": {"enabled": false}}})
            )),
            (false, false)
        );
    }

    #[test]
    fn ill_typed_values_fall_back_to_the_default() {
        assert_eq!(enables(Some(json!("yes"))), (true, true));
        assert_eq!(enables(Some(json!({"semanticTokens": 1}))), (true, true));
        assert_eq!(
            enables(Some(json!({"semanticTokens": {"strings": "off"}}))),
            (true, true)
        );
        // A valid sibling is kept.
        assert_eq!(
            enables(Some(
                json!({"semanticTokens": {"strings": {"enabled": "yes"}, "comments": {"enabled": false}}})
            )),
            (true, false)
        );
    }
}
