use crate::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime, TimeZoneOffset, conversion};

impl TryFrom<jiff::civil::Date> for LocalDate {
    type Error = conversion::Error;

    fn try_from(value: jiff::civil::Date) -> Result<Self, Self::Error> {
        let year = u16::try_from(value.year()).map_err(|_| conversion::Error::InvalidYear)?;
        Ok(Self::from_ymd(year, value.month() as u8, value.day() as u8))
    }
}

impl TryFrom<jiff::Zoned> for LocalDate {
    type Error = conversion::Error;

    fn try_from(value: jiff::Zoned) -> Result<Self, Self::Error> {
        value.date().try_into()
    }
}

impl From<jiff::civil::Time> for LocalTime {
    fn from(value: jiff::civil::Time) -> Self {
        Self::from_hms_nano(
            value.hour() as u8,
            value.minute() as u8,
            value.second() as u8,
            value.subsec_nanosecond() as u32,
        )
    }
}

impl TryFrom<jiff::civil::DateTime> for LocalDateTime {
    type Error = conversion::Error;

    fn try_from(value: jiff::civil::DateTime) -> Result<Self, Self::Error> {
        let date = LocalDate::try_from(value.date())?;
        let time = LocalTime::from(value.time());
        Ok(Self::from_ymd_hms_nano(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
            time.nanosecond(),
        ))
    }
}

impl TryFrom<jiff::Zoned> for LocalDateTime {
    type Error = conversion::Error;

    fn try_from(value: jiff::Zoned) -> Result<Self, Self::Error> {
        value.datetime().try_into()
    }
}

impl TryFrom<jiff::Zoned> for OffsetDateTime {
    type Error = conversion::Error;

    fn try_from(value: jiff::Zoned) -> Result<Self, Self::Error> {
        let date = LocalDate::try_from(value.date())?;
        let time = LocalTime::from(value.time());
        let seconds = value.offset().seconds();
        if seconds % 60 != 0 || !(-86_400..=86_400).contains(&seconds) {
            return Err(conversion::Error::InvalidTimeZoneOffset);
        }
        Ok(Self::from_ymd_hms_nano(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
            time.nanosecond(),
            TimeZoneOffset::Custom {
                minutes: (seconds / 60) as i16,
            },
        ))
    }
}
