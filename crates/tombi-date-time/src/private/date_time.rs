#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub(crate) struct DateTime {
    /// Optional date.
    /// Required for: *Offset Date-Time*, *Local Date-Time*, *Local Date*.
    pub date: Option<crate::private::Date>,

    /// Optional time.
    /// Required for: *Offset Date-Time*, *Local Date-Time*, *Local Time*.
    pub time: Option<crate::private::Time>,

    /// Optional offset.
    /// Required for: *Offset Date-Time*.
    pub offset: Option<crate::TimeZoneOffset>,
}

impl DateTime {
    #[cfg(feature = "serde")]
    pub(crate) fn type_name(&self) -> &'static str {
        match (
            self.date.is_some(),
            self.time.is_some(),
            self.offset.is_some(),
        ) {
            (true, true, true) => crate::OffsetDateTime::type_name(),
            (true, true, false) => crate::LocalDateTime::type_name(),
            (true, false, false) => crate::LocalDate::type_name(),
            (false, true, false) => crate::LocalTime::type_name(),
            _ => unreachable!("unsupported datetime combination"),
        }
    }
}

impl From<crate::private::Date> for DateTime {
    fn from(other: crate::private::Date) -> Self {
        DateTime {
            date: Some(other),
            time: None,
            offset: None,
        }
    }
}

impl From<crate::private::Time> for DateTime {
    fn from(other: crate::private::Time) -> Self {
        DateTime {
            date: None,
            time: Some(other),
            offset: None,
        }
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref date) = self.date {
            write!(f, "{date}")?;
        }
        if let Some(ref time) = self.time {
            if self.date.is_some() {
                write!(f, "T")?;
            }
            write!(f, "{time}")?;
        }
        if let Some(ref offset) = self.offset {
            write!(f, "{offset}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for DateTime {
    type Err = crate::parse::Error;

    fn from_str(date: &str) -> Result<DateTime, crate::parse::Error> {
        // Accepted formats:
        //
        // 0000-00-00T00:00:00.00Z
        // 0000-00-00T00:00:00.00
        // 0000-00-00
        // 00:00:00.00
        if date.len() < 3 {
            return Err(crate::parse::Error::TooShort);
        }
        let mut offset_allowed = true;
        let mut chars = date.chars();

        // First up, parse the full date if we can
        let full_date = if chars.clone().nth(2) == Some(':') {
            offset_allowed = false;
            None
        } else {
            let y1 = u16::from(digit(&mut chars)?);
            let y2 = u16::from(digit(&mut chars)?);
            let y3 = u16::from(digit(&mut chars)?);
            let y4 = u16::from(digit(&mut chars)?);

            match chars.next() {
                Some('-') => {}
                _ => return Err(crate::parse::Error::ExpectedDateOrTimeFormat),
            }

            let m1 = digit(&mut chars)?;
            let m2 = digit(&mut chars)?;

            match chars.next() {
                Some('-') => {}
                _ => return Err(crate::parse::Error::ExpectedYearFormat),
            }

            let d1 = digit(&mut chars)?;
            let d2 = digit(&mut chars)?;

            let date = crate::private::Date {
                year: y1 * 1000 + y2 * 100 + y3 * 10 + y4,
                month: m1 * 10 + m2,
                day: d1 * 10 + d2,
            };

            if date.month < 1 || date.month > 12 {
                return Err(crate::parse::Error::InvalidMonth);
            }
            let is_leap_year =
                (date.year % 4 == 0) && ((date.year % 100 != 0) || (date.year % 400 == 0));
            let max_days_in_month = match date.month {
                2 if is_leap_year => 29,
                2 => 28,
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            if date.day < 1 || date.day > max_days_in_month {
                return Err(crate::parse::Error::InvalidDay {
                    max_days: max_days_in_month,
                });
            }

            Some(date)
        };

        // Next parse the "time-local" if available
        let next = chars.clone().next();
        let partial_time = if full_date.is_some()
            && (next == Some('T') || next == Some('t') || next == Some(' '))
        {
            chars.next();
            true
        } else {
            full_date.is_none()
        };

        let time = if partial_time {
            let h1 = digit(&mut chars)?;
            let h2 = digit(&mut chars)?;
            match chars.next() {
                Some(':') => {}
                _ => return Err(crate::parse::Error::ExpectedTimeFormat),
            }
            let m1 = digit(&mut chars)?;
            let m2 = digit(&mut chars)?;
            let (s1, s2, nanosecond) = match chars.clone().next() {
                Some(':') => {
                    chars.next();
                    let s1 = digit(&mut chars)?;
                    let s2 = digit(&mut chars)?;

                    let mut nanosecond = 0;
                    if chars.clone().next() == Some('.') {
                        chars.next();
                        let whole = chars.as_str();

                        let mut end = whole.len();
                        for (i, byte) in whole.bytes().enumerate() {
                            #[allow(clippy::single_match_else)]
                            match byte {
                                b'0'..=b'9' => {
                                    if i < 9 {
                                        let p = 10_u32.pow(8 - i as u32);
                                        nanosecond += p * u32::from(byte - b'0');
                                    }
                                }
                                _ => {
                                    end = i;
                                    break;
                                }
                            }
                        }
                        if end == 0 {
                            return Err(crate::parse::Error::ExpectedNanoseconds);
                        }
                        chars = whole[end..].chars();
                    }

                    (s1, s2, nanosecond)
                }
                _ => (0, 0, 0),
            };

            let time = crate::private::Time {
                hour: h1 * 10 + h2,
                minute: m1 * 10 + m2,
                second: s1 * 10 + s2,
                nanosecond,
            };

            if time.hour >= 24 {
                return Err(crate::parse::Error::InvalidHour);
            }
            if time.minute >= 60 {
                return Err(crate::parse::Error::InvalidMinute);
            }
            // 00-58, 00-59, 00-60 based on leap second rules
            if time.second >= 60 {
                return Err(crate::parse::Error::InvalidSecond);
            }
            if time.nanosecond > 999_999_999 {
                return Err(crate::parse::Error::InvalidNanoseconds);
            }

            Some(time)
        } else {
            offset_allowed = false;
            None
        };

        // And finally, parse the offset
        let offset = if offset_allowed {
            let next = chars.clone().next();
            if next == Some('Z') || next == Some('z') {
                chars.next();
                Some(crate::TimeZoneOffset::Z)
            } else if next.is_none() {
                None
            } else {
                let sign = match next {
                    Some('+') => 1,
                    Some('-') => -1,
                    _ => return Err(crate::parse::Error::ExpectedTimeZoneOffsetSign),
                };
                chars.next();
                let h1 = digit(&mut chars)? as i16;
                let h2 = digit(&mut chars)? as i16;
                match chars.next() {
                    Some(':') => {}
                    _ => return Err(crate::parse::Error::ExpectedTimeZoneOffsetColon),
                }
                let m1 = digit(&mut chars)? as i16;
                let m2 = digit(&mut chars)? as i16;

                let hours = h1 * 10 + h2;
                let minutes = m1 * 10 + m2;

                if hours >= 24 {
                    return Err(crate::parse::Error::InvalidTimeZoneOffsetHour);
                }
                if minutes >= 60 {
                    return Err(crate::parse::Error::InvalidTimeZoneOffsetMinute);
                }

                let total_minutes = sign * (hours * 60 + minutes);

                if !((-24 * 60)..=(24 * 60)).contains(&total_minutes) {
                    return Err(crate::parse::Error::InvalidTimeZoneOffset);
                }

                Some(crate::TimeZoneOffset::Custom {
                    minutes: total_minutes,
                })
            }
        } else {
            None
        };

        // Return an error if we didn't hit eof, otherwise return our parsed
        // date
        if chars.next().is_some() {
            return Err(crate::parse::Error::TooLong);
        }

        Ok(DateTime {
            date: full_date,
            time,
            offset,
        })
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        struct DateTimeStr<'a>(&'a str);

        impl serde::ser::Serialize for DateTimeStr<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                serializer.serialize_str(self.0)
            }
        }

        serializer.serialize_newtype_struct(
            match self.type_name() {
                t if t == crate::OffsetDateTime::type_name() => {
                    crate::OFFSET_DATE_TIME_NEWTYPE_NAME
                }
                t if t == crate::LocalDateTime::type_name() => crate::LOCAL_DATE_TIME_NEWTYPE_NAME,
                t if t == crate::LocalDate::type_name() => crate::LOCAL_DATE_NEWTYPE_NAME,
                t if t == crate::LocalTime::type_name() => crate::LOCAL_TIME_NEWTYPE_NAME,
                _ => unreachable!("unsupported datetime combination"),
            },
            &DateTimeStr(&self.to_string()),
        )
    }
}

#[cfg(feature = "serde")]
pub(crate) struct DateTimeVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for DateTimeVisitor {
    type Value = DateTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a TOML DateTime")
    }

    fn visit_str<E>(self, s: &str) -> Result<DateTime, E>
    where
        E: serde::de::Error,
    {
        s.parse().map_err(serde::de::Error::custom)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<DateTime, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

fn digit(chars: &mut std::str::Chars<'_>) -> Result<u8, crate::parse::Error> {
    match chars.next() {
        Some(c) if c.is_ascii_digit() => Ok(c as u8 - b'0'),
        _ => Err(crate::parse::Error::InvalidFormat),
    }
}
