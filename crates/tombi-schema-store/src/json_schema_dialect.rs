use tombi_uri::Uri;

const JSON_SCHEMA_HOST: &str = "json-schema.org";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JsonSchemaDialect {
    #[default]
    Draft07,
    Draft2019_09,
    Draft2020_12,
}

impl TryFrom<&Uri> for JsonSchemaDialect {
    type Error = ();

    fn try_from(uri: &Uri) -> Result<Self, Self::Error> {
        if uri.host_str() != Some(JSON_SCHEMA_HOST) {
            return Err(());
        }

        match uri.path() {
            "/draft-07/schema" => Ok(Self::Draft07),
            "/draft/2019-09/schema" => Ok(Self::Draft2019_09),
            "/draft/2020-12/schema" => Ok(Self::Draft2020_12),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for JsonSchemaDialect {
    type Error = ();

    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        let url: Uri = uri.parse().map_err(|_| ())?;
        Self::try_from(&url)
    }
}

impl std::fmt::Display for JsonSchemaDialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft07 => write!(f, "draft-07"),
            Self::Draft2019_09 => write!(f, "draft-2019-09"),
            Self::Draft2020_12 => write!(f, "draft-2020-12"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_draft_07_http() {
        assert_eq!(
            JsonSchemaDialect::try_from("http://json-schema.org/draft-07/schema#"),
            Ok(JsonSchemaDialect::Draft07),
        );
    }

    #[test]
    fn from_str_draft_07_https_no_fragment() {
        assert_eq!(
            JsonSchemaDialect::try_from("https://json-schema.org/draft-07/schema"),
            Ok(JsonSchemaDialect::Draft07),
        );
    }

    #[test]
    fn from_str_draft_2019_09() {
        assert_eq!(
            JsonSchemaDialect::try_from("https://json-schema.org/draft/2019-09/schema"),
            Ok(JsonSchemaDialect::Draft2019_09),
        );
    }

    #[test]
    fn from_str_draft_2020_12() {
        assert_eq!(
            JsonSchemaDialect::try_from("https://json-schema.org/draft/2020-12/schema"),
            Ok(JsonSchemaDialect::Draft2020_12),
        );
    }

    #[test]
    fn from_str_unknown_host() {
        assert_eq!(
            JsonSchemaDialect::try_from("https://example.com/draft-07/schema"),
            Err(()),
        );
    }

    #[test]
    fn from_str_unknown_path() {
        assert_eq!(
            JsonSchemaDialect::try_from("https://json-schema.org/draft-04/schema"),
            Err(()),
        );
    }

    #[test]
    fn from_str_invalid_uri() {
        assert_eq!(JsonSchemaDialect::try_from("not a uri"), Err(()));
    }

    #[test]
    fn from_uri_draft_2020_12() {
        let uri: Uri = "https://json-schema.org/draft/2020-12/schema"
            .parse()
            .unwrap();
        assert_eq!(
            JsonSchemaDialect::try_from(&uri),
            Ok(JsonSchemaDialect::Draft2020_12),
        );
    }
}
