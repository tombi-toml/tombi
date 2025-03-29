#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetDateTime {
    value: chrono::DateTime<chrono::FixedOffset>,
}

impl OffsetDateTime {
    #[inline]
    pub fn value(&self) -> &chrono::DateTime<chrono::FixedOffset> {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDateTime {
    value: chrono::NaiveDateTime,
}

impl LocalDateTime {
    #[inline]
    pub fn value(&self) -> &chrono::NaiveDateTime {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDate {
    value: chrono::NaiveDate,
}

impl LocalDate {
    #[inline]
    pub fn value(&self) -> &chrono::NaiveDate {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTime {
    value: chrono::NaiveTime,
}

impl LocalTime {
    #[inline]
    pub fn value(&self) -> &chrono::NaiveTime {
        &self.value
    }
}

impl From<document_tree::OffsetDateTime> for OffsetDateTime {
    fn from(node: document_tree::OffsetDateTime) -> Self {
        Self {
            value: *node.value(),
        }
    }
}

impl From<document_tree::LocalDateTime> for LocalDateTime {
    fn from(node: document_tree::LocalDateTime) -> Self {
        Self {
            value: *node.value(),
        }
    }
}

impl From<document_tree::LocalDate> for LocalDate {
    fn from(node: document_tree::LocalDate) -> Self {
        Self {
            value: *node.value(),
        }
    }
}

impl From<document_tree::LocalTime> for LocalTime {
    fn from(node: document_tree::LocalTime) -> Self {
        Self {
            value: *node.value(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for OffsetDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LocalDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LocalDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LocalTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for OffsetDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        chrono::DateTime::deserialize(deserializer).map(|value| Self { value })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LocalDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        chrono::NaiveDateTime::deserialize(deserializer).map(|value| Self { value })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LocalDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        chrono::NaiveDate::deserialize(deserializer).map(|value| Self { value })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LocalTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        chrono::NaiveTime::deserialize(deserializer).map(|value| Self { value })
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use toml_version::TomlVersion;

    use crate::test_deserialize;

    test_deserialize! {
        #[test]
        fn offset_date_time(r#"odt = 1979-05-27T07:32:00Z"#) -> Ok(json!({
            "odt": "1979-05-27T07:32:00Z"
        }))
    }

    test_deserialize! {
        #[test]
        fn offset_date_time2(r#"odt = 1979-05-27T07:32:00.9999Z"#) -> Ok(json!({
            "odt": "1979-05-27T07:32:00.999900Z"
        }))
    }

    test_deserialize! {
        #[test]
        fn offset_date_time3(r#"odt = 1979-05-27 07:32:00.99999999Z"#) -> Ok(json!({
            "odt": "1979-05-27T07:32:00.999999990Z"
        }))
    }

    test_deserialize! {
        #[test]
        fn offset_date_time4(r#"odt = 1979-05-27t07:32:00.9999z"#) -> Ok(json!({
            "odt": "1979-05-27T07:32:00.999900Z"
        }))
    }

    test_deserialize! {
        #[test]
        fn offset_date_time_optional_seconds1_in_toml_v1_0_0(r#"odt = 1979-05-27T07:32Z"#, TomlVersion::V1_0_0) -> Err([
            ("invalid offset date time: optional seconds are allowed in TOML v1.1.0 or later", ((0, 6), (0, 23)))
        ])
    }

    test_deserialize! {
        #[test]
        fn offset_date_time_optional_seconds_in_toml_v1_1_0(r#"odt = 1979-05-27T07:32Z"#, TomlVersion::V1_1_0_Preview) -> Ok(json!({
            "odt": "1979-05-27T07:32:00Z"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_date_time(r#"ldt = 1979-05-27 07:32:00"#) -> Ok(json!({
            "ldt": "1979-05-27T07:32:00"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_date_time2(r#"ldt = 1979-05-27 07:32:00.9999"#) -> Ok(json!({
            "ldt": "1979-05-27T07:32:00.999900"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_date_time3(r#"ldt = 1979-05-27 07:32:00.99999999"#) -> Ok(json!({
            "ldt": "1979-05-27T07:32:00.999999990"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_date_time4(r#"ldt = 1979-05-27T07:32:00"#) -> Ok(json!({
            "ldt": "1979-05-27T07:32:00"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_date_time5(r#"ldt = 1979-05-27t07:32:00"#) -> Ok(json!({
            "ldt": "1979-05-27T07:32:00"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_date_time_optional_seconds_in_toml_v1_0_0(r#"ldt = 1979-05-27 07:32"#, TomlVersion::V1_0_0) -> Err([
            ("invalid local date time: optional seconds are allowed in TOML v1.1.0 or later", ((0, 6), (0, 22)))
        ])
    }

    test_deserialize! {
        #[test]
        fn local_date_time_optional_seconds_in_toml_v1_1_0(r#"ldt = 1979-05-27 07:32"#, TomlVersion::V1_1_0_Preview) -> Ok(json!({
            "ldt": "1979-05-27T07:32:00"
        }))
    }

    test_deserialize! {
        #[test]
        fn invalid_date(r#"date = 0000-00-00"#) -> Err([
            ("invalid local date: input is out of range", ((0, 7), (0, 17)))
        ])
    }

    test_deserialize! {
        #[test]
        fn local_time(r#"lt = 07:32:00"#) -> Ok(json!({
            "lt": "07:32:00"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_time2(r#"lt = 07:32:00.9999"#) -> Ok(json!({
            "lt": "07:32:00.999900"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_time3(r#"lt = 07:32:00.99999999"#) -> Ok(json!({
            "lt": "07:32:00.999999990"
        }))
    }

    test_deserialize! {
        #[test]
        fn local_time_optional_seconds_in_toml_v1_0_0(r#"lt = 07:32"#, TomlVersion::V1_0_0) -> Err([
            ("invalid local time: optional seconds are allowed in TOML v1.1.0 or later", ((0, 5), (0, 10)))
        ])
    }

    test_deserialize! {
        #[test]
        fn local_time_optional_seconds_in_toml_v1_1_0(r#"lt = 07:32"#, TomlVersion::V1_1_0_Preview) -> Ok(json!({
            "lt": "07:32:00"
        }))
    }
}
