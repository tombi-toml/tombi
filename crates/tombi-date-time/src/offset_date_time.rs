#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub struct OffsetDateTime {
    date: crate::private::Date,
    time: crate::private::Time,
    offset: crate::TimeZoneOffset,
}

impl OffsetDateTime {
    #[cfg(feature = "serde")]
    pub fn type_name() -> &'static str {
        "offset date time"
    }

    pub fn from_ymd_hms(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        offset: crate::TimeZoneOffset,
    ) -> Self {
        Self::from_ymd_hms_milli(year, month, day, hour, minute, second, 0, offset)
    }

    pub fn from_ymd_hms_milli(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        milli: u32,
        offset: crate::TimeZoneOffset,
    ) -> Self {
        Self::from_ymd_hms_nano(
            year,
            month,
            day,
            hour,
            minute,
            second,
            milli * 1_000_000,
            offset,
        )
    }

    pub fn from_ymd_hms_nano(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
        offset: crate::TimeZoneOffset,
    ) -> Self {
        debug_assert!(nanosecond < 1_000_000_000);
        Self {
            date: crate::private::Date { year, month, day },
            time: crate::private::Time {
                hour,
                minute,
                second,
                nanosecond,
            },
            offset,
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

    pub fn offset(&self) -> crate::TimeZoneOffset {
        self.offset
    }
}

impl std::str::FromStr for OffsetDateTime {
    type Err = crate::parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match crate::private::DateTime::from_str(s) {
            Ok(crate::private::DateTime {
                date: Some(date),
                time: Some(time),
                offset: Some(offset),
            }) => Ok(Self { date, time, offset }),
            Ok(_) => Err(crate::parse::Error::ExpectedOffsetDateTime),
            Err(error) => Err(error),
        }
    }
}

impl std::fmt::Display for OffsetDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.date.fmt(f)?;
        write!(f, "T")?;
        self.time.fmt(f)?;
        self.offset.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for OffsetDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        crate::private::DateTime {
            date: Some(self.date),
            time: Some(self.time),
            offset: Some(self.offset),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for OffsetDateTime {
    fn deserialize<D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        match deserializer.deserialize_newtype_struct(
            crate::OFFSET_DATE_TIME_NEWTYPE_NAME,
            crate::private::DateTimeVisitor,
        )? {
            crate::private::DateTime {
                date: Some(date),
                time: Some(time),
                offset: Some(offset),
            } => Ok(OffsetDateTime { date, time, offset }),
            datetime => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other(datetime.type_name()),
                &Self::type_name(),
            )),
        }
    }
}
