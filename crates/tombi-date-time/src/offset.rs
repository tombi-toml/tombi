#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub enum TimeZoneOffset {
    /// > A suffix which, when applied to a time, denotes a UTC offset of 00:00;
    /// > often spoken "Zulu" from the ICAO phonetic alphabet representation of
    /// > the letter "Z". --- [RFC 3339 section 2]
    ///
    /// [RFC 3339 section 2]: https://datatracker.ietf.org/doc/html/rfc3339#section-2
    Z,

    /// Offset between local time and UTC
    Custom {
        /// Minutes: -`1_440..1_440`
        minutes: i16,
    },
}

impl std::fmt::Display for TimeZoneOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TimeZoneOffset::Z => write!(f, "Z"),
            TimeZoneOffset::Custom { mut minutes } => {
                let mut sign = '+';
                if minutes < 0 {
                    minutes *= -1;
                    sign = '-';
                }
                let hours = minutes / 60;
                let minutes = minutes % 60;
                write!(f, "{sign}{hours:02}:{minutes:02}")
            }
        }
    }
}
