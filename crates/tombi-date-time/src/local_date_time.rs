#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub struct LocalDateTime {
    date: crate::private::Date,
    time: crate::private::Time,
}

impl LocalDateTime {
    #[cfg(feature = "serde")]
    pub fn type_name() -> &'static str {
        "local date time"
    }

    pub fn from_ymd_hms(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            date: crate::private::Date { year, month, day },
            time: crate::private::Time {
                hour,
                minute,
                second,
                nanosecond: 0,
            },
        }
    }

    pub fn from_ymd_hms_milli(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u32,
    ) -> Self {
        assert!(millisecond < 1_000);

        Self {
            date: crate::private::Date { year, month, day },
            time: crate::private::Time {
                hour,
                minute,
                second,
                nanosecond: millisecond * 1_000_000,
            },
        }
    }

    pub fn from_ymd_hms_nano(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Self {
        assert!(nanosecond < 1_000_000_000);
        Self {
            date: crate::private::Date { year, month, day },
            time: crate::private::Time {
                hour,
                minute,
                second,
                nanosecond,
            },
        }
    }

    pub fn year(&self) -> u16 {
        self.date.year
    }

    pub fn month(&self) -> u8 {
        self.date.month
    }

    pub fn day(&self) -> u8 {
        self.date.day
    }

    pub fn hour(&self) -> u8 {
        self.time.hour
    }

    pub fn minute(&self) -> u8 {
        self.time.minute
    }

    pub fn second(&self) -> u8 {
        self.time.second
    }

    pub fn nanosecond(&self) -> u32 {
        self.time.nanosecond
    }
}

impl std::str::FromStr for LocalDateTime {
    type Err = crate::parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match crate::private::DateTime::from_str(s) {
            Ok(crate::private::DateTime {
                date: Some(date),
                time: Some(time),
                offset: None,
            }) => Ok(Self { date, time }),
            Ok(_) => Err(crate::parse::Error::ExpectedLocalDateTime),
            Err(error) => Err(error),
        }
    }
}

impl std::fmt::Display for LocalDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.date.fmt(f)?;
        write!(f, "T")?;
        self.time.fmt(f)?;

        Ok(())
    }
}

// NOTE: `chrono::DateTime<chrono::Local>` is not enough to represent local date time.
//       `chrono::Local.from_local_datetime(native_date_time)` cannot uniquely determine the time zone in some cases, so we handle NativeDateTime.
#[cfg(feature = "chrono")]
impl From<chrono::NaiveDateTime> for LocalDateTime {
    fn from(value: chrono::NaiveDateTime) -> Self {
        use chrono::Datelike;
        use chrono::Timelike;

        Self::from_ymd_hms_nano(
            value.year() as u16,
            value.month() as u8,
            value.day() as u8,
            value.hour() as u8,
            value.minute() as u8,
            value.second() as u8,
            value.nanosecond(),
        )
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::Local>> for LocalDateTime {
    fn from(value: chrono::DateTime<chrono::Local>) -> Self {
        value.naive_local().into()
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for LocalDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        crate::private::DateTime {
            date: Some(self.date),
            time: Some(self.time),
            offset: None,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for LocalDateTime {
    fn deserialize<D>(deserializer: D) -> Result<LocalDateTime, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        match deserializer.deserialize_newtype_struct(
            crate::LOCAL_DATE_TIME_NEWTYPE_NAME,
            crate::private::DateTimeVisitor,
        )? {
            crate::private::DateTime {
                date: Some(date),
                time: Some(time),
                offset: None,
            } => Ok(LocalDateTime { date, time }),
            datetime => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other(datetime.type_name()),
                &Self::type_name(),
            )),
        }
    }
}
