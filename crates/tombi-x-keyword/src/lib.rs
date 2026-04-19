use std::str::FromStr;

pub const X_TOMBI_TOML_VERSION: &str = "x-tombi-toml-version";
pub const X_TOMBI_ARRAY_VALUES_ORDER: &str = "x-tombi-array-values-order";
pub const X_TOMBI_ARRAY_VALUES_ORDER_BY: &str = "x-tombi-array-values-order-by";
pub const X_TOMBI_TABLE_KEYS_ORDER: &str = "x-tombi-table-keys-order";
pub const X_TOMBI_STRING_FORMATS: &str = "x-tombi-string-formats";
pub const X_TOMBI_ADDITIONAL_KEY_LABEL: &str = "x-tombi-additional-key-label";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum ArrayValuesOrderGroup {
    OneOf(Vec<ArrayValuesOrder>),
    AnyOf(Vec<ArrayValuesOrder>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayValuesOrderBy(String);

impl std::fmt::Display for ArrayValuesOrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> TryFrom<&'a str> for ArrayValuesOrderBy {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(value.to_string()))
    }
}

impl PartialEq<ArrayValuesOrderBy> for String {
    fn eq(&self, other: &ArrayValuesOrderBy) -> bool {
        self == &other.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum TableKeysOrderGroupKind {
    Keys,
    AdditionalKeys,
    PatternKeys,
}

impl std::fmt::Display for TableKeysOrderGroupKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableKeysOrderGroupKind::Keys => write!(f, "Keys"),
            TableKeysOrderGroupKind::AdditionalKeys => write!(f, "Additional Keys"),
            TableKeysOrderGroupKind::PatternKeys => write!(f, "Pattern Keys"),
        }
    }
}

impl<'a> TryFrom<&'a str> for TableKeysOrderGroupKind {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "properties" => Ok(TableKeysOrderGroupKind::Keys),
            "additionalProperties" => Ok(TableKeysOrderGroupKind::AdditionalKeys),
            "patternProperties" => Ok(TableKeysOrderGroupKind::PatternKeys),
            _ => Err("Invalid table group"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986)
    UriReference,

    /// [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122)
    Uuid,

    /// [RFC 2673 §3.2](https://datatracker.ietf.org/doc/html/rfc2673#section-3.2)
    Ipv4,

    /// [RFC 4291 §2.2](https://datatracker.ietf.org/doc/html/rfc4291#section-2.2)
    Ipv6,

    /// [RFC 3339 §5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6)
    DateTime,

    /// [OpenAPI Format Registry](https://spec.openapis.org/registry/format/date-time-local.html)
    DateTimeLocal,

    /// [RFC 3339 §5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) full-date
    Date,

    /// [RFC 3339 §5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) full-time
    Time,

    /// [OpenAPI Format Registry](https://spec.openapis.org/registry/format/time-local.html)
    TimeLocal,

    /// [ECMA-262](https://262.ecma-international.org/)
    Regex,

    /// [RFC 6901](https://datatracker.ietf.org/doc/html/rfc6901)
    JsonPointer,
}

impl std::fmt::Display for StringFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::Hostname => write!(f, "hostname"),
            Self::Uri => write!(f, "uri"),
            Self::UriReference => write!(f, "uri-reference"),
            Self::Uuid => write!(f, "uuid"),
            Self::Ipv4 => write!(f, "ipv4"),
            Self::Ipv6 => write!(f, "ipv6"),
            Self::DateTime => write!(f, "date-time"),
            Self::DateTimeLocal => write!(f, "date-time-local"),
            Self::Date => write!(f, "date"),
            Self::Time => write!(f, "time"),
            Self::TimeLocal => write!(f, "time-local"),
            Self::Regex => write!(f, "regex"),
            Self::JsonPointer => write!(f, "json-pointer"),
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
            "uri-reference" => Ok(Self::UriReference),
            "uuid" => Ok(Self::Uuid),
            "ipv4" => Ok(Self::Ipv4),
            "ipv6" => Ok(Self::Ipv6),
            "date-time" => Ok(Self::DateTime),
            "date-time-local" | "partial-date-time" => Ok(Self::DateTimeLocal),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "time-local" | "partial-time" => Ok(Self::TimeLocal),
            "regex" => Ok(Self::Regex),
            "json-pointer" => Ok(Self::JsonPointer),
            _ => Err(()),
        }
    }
}
