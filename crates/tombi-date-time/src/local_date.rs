#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub struct LocalDate(crate::private::Date);

impl LocalDate {
    #[cfg(feature = "serde")]
    pub fn type_name() -> &'static str {
        "local date"
    }

    pub fn from_ymd(year: u16, month: u8, day: u8) -> Self {
        Self(crate::private::Date { year, month, day })
    }

    pub fn year(&self) -> u16 {
        self.0.year
    }

    pub fn month(&self) -> u8 {
        self.0.month
    }

    pub fn day(&self) -> u8 {
        self.0.day
    }
}

impl std::str::FromStr for LocalDate {
    type Err = crate::parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match crate::private::DateTime::from_str(s) {
            Ok(crate::private::DateTime {
                date: Some(date),
                time: None,
                offset: None,
            }) => Ok(Self(date)),
            Ok(_) => Err(crate::parse::Error::ExpectedLocalDate),
            Err(error) => Err(error),
        }
    }
}

impl std::fmt::Display for LocalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for LocalDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        crate::private::DateTime::from(self.0).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for LocalDate {
    fn deserialize<D>(deserializer: D) -> Result<LocalDate, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        match deserializer.deserialize_newtype_struct(
            crate::LOCAL_DATE_NEWTYPE_NAME,
            crate::private::DateTimeVisitor,
        )? {
            crate::private::DateTime {
                date: Some(date),
                time: None,
                offset: None,
            } => Ok(LocalDate(date)),
            datetime => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other(datetime.type_name()),
                &Self::type_name(),
            )),
        }
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::NaiveDate> for LocalDate {
    fn from(value: chrono::NaiveDate) -> Self {
        use chrono::Datelike;

        Self(crate::private::Date {
            year: value.year() as u16,
            month: value.month() as u8,
            day: value.day() as u8,
        })
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::Local>> for LocalDate {
    fn from(value: chrono::DateTime<chrono::Local>) -> Self {
        value.naive_local().date().into()
    }
}
