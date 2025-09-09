use std::str::FromStr;

pub const X_TOMBI_TOML_VERSION: &str = "x-tombi-toml-version";
pub const X_TOMBI_ARRAY_VALUES_ORDER: &str = "x-tombi-array-values-order";
pub const X_TOMBI_TABLE_KEYS_ORDER: &str = "x-tombi-table-keys-order";
pub const X_TOMBI_STRING_FORMATS: &str = "x-tombi-string-formats";

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum ArrayValuesOrder {
    Ascending,
    Descending,
    VersionSort,
}

impl std::fmt::Display for ArrayValuesOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascending => write!(f, "ascending"),
            Self::Descending => write!(f, "descending"),
            Self::VersionSort => write!(f, "version-sort"),
        }
    }
}

impl<'a> TryFrom<&'a str> for ArrayValuesOrder {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "ascending" => Ok(Self::Ascending),
            "descending" => Ok(Self::Descending),
            "version-sort" => Ok(Self::VersionSort),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TableKeysOrderGroup {
    Keys,
    AdditionalKeys,
    PatternKeys,
}

impl std::fmt::Display for TableKeysOrderGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableKeysOrderGroup::Keys => write!(f, "Keys"),
            TableKeysOrderGroup::AdditionalKeys => write!(f, "Additional Keys"),
            TableKeysOrderGroup::PatternKeys => write!(f, "Pattern Keys"),
        }
    }
}

impl<'a> TryFrom<&'a str> for TableKeysOrderGroup {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "properties" => Ok(TableKeysOrderGroup::Keys),
            "additionalProperties" => Ok(TableKeysOrderGroup::AdditionalKeys),
            "patternProperties" => Ok(TableKeysOrderGroup::PatternKeys),
            _ => Err("Invalid table group"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TableKeysOrder {
    Ascending,
    Descending,
    Schema,
    VersionSort,
}

impl std::fmt::Display for TableKeysOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableKeysOrder::Ascending => write!(f, "ascending"),
            TableKeysOrder::Descending => write!(f, "descending"),
            TableKeysOrder::Schema => write!(f, "schema"),
            TableKeysOrder::VersionSort => write!(f, "version-sort"),
        }
    }
}

impl<'a> TryFrom<&'a str> for TableKeysOrder {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "ascending" => Ok(Self::Ascending),
            "descending" => Ok(Self::Descending),
            "schema" => Ok(Self::Schema),
            "version-sort" => Ok(Self::VersionSort),
            _ => Err("Invalid table keys order"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum StringFormat {
    /// [RFC 5322](https://datatracker.ietf.org/doc/html/rfc5322)
    Email,

    /// [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
    Hostname,

    /// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986)
    Uri,

    /// [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122)
    Uuid,
}

impl std::fmt::Display for StringFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::Hostname => write!(f, "hostname"),
            Self::Uri => write!(f, "uri"),
            Self::Uuid => write!(f, "uuid"),
        }
    }
}

impl FromStr for StringFormat {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "email" => Ok(Self::Email),
            "hostname" => Ok(Self::Hostname),
            "uri" => Ok(Self::Uri),
            "uuid" => Ok(Self::Uuid),
            _ => Err(()),
        }
    }
}
