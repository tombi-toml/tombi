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

/// The TOML value type represented by a supported string format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TomlDateTimeType {
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
}

macro_rules! define_string_formats {
    (
        $(
            $(#[$meta:meta])*
            $variant:ident => {
                name: $name:literal,
                aliases: [$($alias:literal),* $(,)?],
                toml_date_time_type: $toml_date_time_type:tt $(,)?
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
        #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
        pub enum StringFormat {
            $($(#[$meta])* $variant),*
        }

        impl StringFormat {
            /// All string formats recognized by Tombi, excluding aliases.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            /// The canonical JSON Schema format name.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }

            /// Non-canonical names accepted for compatibility.
            pub const fn aliases(self) -> &'static [&'static str] {
                match self {
                    $(Self::$variant => &[$($alias),*]),*
                }
            }

            /// The TOML date/time value type represented by this format, if any.
            pub const fn toml_date_time_type(self) -> Option<TomlDateTimeType> {
                match self {
                    $(Self::$variant => define_string_formats!(@toml_date_time_type $toml_date_time_type)),*
                }
            }
        }

        impl std::fmt::Display for StringFormat {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for StringFormat {
            type Err = ();

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($name $(| $alias)* => Ok(Self::$variant)),*,
                    _ => Err(()),
                }
            }
        }
    };
    (@toml_date_time_type None) => { None };
    (@toml_date_time_type $variant:ident) => { Some(TomlDateTimeType::$variant) };
}

define_string_formats! {
    /// [RFC 5322](https://datatracker.ietf.org/doc/html/rfc5322)
    Email => { name: "email", aliases: [], toml_date_time_type: None },
    /// [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
    Hostname => { name: "hostname", aliases: [], toml_date_time_type: None },
    /// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986)
    Uri => { name: "uri", aliases: [], toml_date_time_type: None },
    /// [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986)
    UriReference => { name: "uri-reference", aliases: [], toml_date_time_type: None },
    /// [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122)
    Uuid => { name: "uuid", aliases: [], toml_date_time_type: None },
    /// [RFC 2673 §3.2](https://datatracker.ietf.org/doc/html/rfc2673#section-3.2)
    Ipv4 => { name: "ipv4", aliases: [], toml_date_time_type: None },
    /// [RFC 4291 §2.2](https://datatracker.ietf.org/doc/html/rfc4291#section-2.2)
    Ipv6 => { name: "ipv6", aliases: [], toml_date_time_type: None },
    /// [RFC 3339 §5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6)
    DateTime => {
        name: "date-time",
        aliases: [],
        toml_date_time_type: OffsetDateTime,
    },
    /// [OpenAPI Format Registry](https://spec.openapis.org/registry/format/date-time-local.html)
    DateTimeLocal => {
        name: "date-time-local",
        aliases: ["partial-date-time"],
        toml_date_time_type: LocalDateTime,
    },
    /// [RFC 3339 §5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) full-date
    Date => { name: "date", aliases: [], toml_date_time_type: LocalDate },
    /// [RFC 3339 §5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6) full-time
    Time => { name: "time", aliases: [], toml_date_time_type: None },
    /// [OpenAPI Format Registry](https://spec.openapis.org/registry/format/time-local.html)
    TimeLocal => {
        name: "time-local",
        aliases: ["partial-time"],
        toml_date_time_type: LocalTime,
    },
    /// [ECMA-262](https://262.ecma-international.org/)
    Regex => { name: "regex", aliases: [], toml_date_time_type: None },
    /// [RFC 6901](https://datatracker.ietf.org/doc/html/rfc6901)
    JsonPointer => { name: "json-pointer", aliases: [], toml_date_time_type: None },
}

#[cfg(test)]
mod tests {
    use super::{StringFormat, TomlDateTimeType};

    #[test]
    fn string_format_names_round_trip() {
        for format in StringFormat::ALL {
            assert_eq!(format.as_str().parse(), Ok(*format));
            assert_eq!(format.to_string(), format.as_str());
        }
    }

    #[test]
    fn string_format_aliases_resolve_to_their_canonical_format() {
        for format in StringFormat::ALL {
            for alias in format.aliases() {
                assert_eq!(alias.parse(), Ok(*format));
            }
        }
    }

    #[test]
    fn toml_date_time_formats_are_classified_centrally() {
        assert_eq!(
            StringFormat::DateTime.toml_date_time_type(),
            Some(TomlDateTimeType::OffsetDateTime)
        );
        assert_eq!(
            StringFormat::DateTimeLocal.toml_date_time_type(),
            Some(TomlDateTimeType::LocalDateTime)
        );
        assert_eq!(
            StringFormat::Date.toml_date_time_type(),
            Some(TomlDateTimeType::LocalDate)
        );
        assert_eq!(
            StringFormat::TimeLocal.toml_date_time_type(),
            Some(TomlDateTimeType::LocalTime)
        );
        assert_eq!(StringFormat::Time.toml_date_time_type(), None);
    }
}
