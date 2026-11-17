pub mod parse {
    #[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum Error {
        #[error("date-time string is too short")]
        TooShort,

        #[error("date-time string is too long")]
        TooLong,

        #[error("expected number")]
        ExpectedNumber,

        #[error("expected OffsetDateTime")]
        ExpectedOffsetDateTime,

        #[error("expected LocalDateTime")]
        ExpectedLocalDateTime,

        #[error("expected LocalDate")]
        ExpectedLocalDate,

        #[error("expected LocalTime")]
        ExpectedLocalTime,

        #[error("expected yyyy-mm-dd or hh:mm:ss")]
        ExpectedDateOrTimeFormat,

        #[error("expected yyyy-mm-dd")]
        ExpectedYearFormat,

        #[error("expected hh:mm:ss")]
        ExpectedTimeFormat,

        #[error("expected nanoseconds")]
        ExpectedNanoseconds,

        #[error("expected time zone offset sign '+' or '-'")]
        ExpectedTimeZoneOffsetSign,

        #[error("expected time zone offset ':'")]
        ExpectedTimeZoneOffsetColon,

        #[error("month must be between 1 and 12")]
        InvalidMonth,

        #[error("day must be between 1 and {max_days}")]
        InvalidDay { max_days: u8 },

        #[error("hour must be between 0 and 24")]
        InvalidHour,

        #[error("minute must be between 0 and 59")]
        InvalidMinute,

        #[error("second must be between 0 and 60")]
        InvalidSecond,

        #[error("nanosecond must be between 0 and 999_999_999")]
        InvalidNanoseconds,

        #[error("time zone offset hour must be between 0 and 24")]
        InvalidTimeZoneOffsetHour,

        #[error("time zone offset minute must be between 0 and 59")]
        InvalidTimeZoneOffsetMinute,

        #[error("time zone offset must be between -24:00 and 24:00")]
        InvalidTimeZoneOffset,

        #[error("invalid TOML DateTime format")]
        InvalidFormat,
    }
}

pub mod de {
    #[derive(Debug, Clone, thiserror::Error)]
    #[non_exhaustive]
    pub enum Error {}
}

#[cfg(feature = "jiff")]
pub mod conversion {
    #[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum Error {
        #[error("year must be between 0 and 9999")]
        InvalidYear,
        #[error("time zone offset must be whole minutes between -24:00 and 24:00")]
        InvalidTimeZoneOffset,
    }
}
