#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HoverMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
}

pub fn append_latest_version(
    description: Option<String>,
    latest_version: Option<String>,
) -> Option<String> {
    match (description, latest_version) {
        (Some(description), Some(latest_version)) => Some(format!(
            "{description}\n\nLatest Version: `{latest_version}`"
        )),
        (None, Some(latest_version)) => Some(format!("Latest Version: `{latest_version}`")),
        (description, None) => description,
    }
}

#[cfg(test)]
mod tests {
    use super::append_latest_version;

    #[test]
    fn appends_latest_version_to_description() {
        assert_eq!(
            append_latest_version(Some("serde".to_string()), Some("1.0.228".to_string())),
            Some("serde\n\nLatest Version: `1.0.228`".to_string())
        );
    }

    #[test]
    fn returns_latest_version_without_description() {
        assert_eq!(
            append_latest_version(None, Some("1.0.228".to_string())),
            Some("Latest Version: `1.0.228`".to_string())
        );
    }

    #[test]
    fn returns_description_when_latest_version_is_missing() {
        assert_eq!(
            append_latest_version(Some("serde".to_string()), None),
            Some("serde".to_string())
        );
    }
}
