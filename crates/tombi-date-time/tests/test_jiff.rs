#![cfg(feature = "jiff")]

use jiff::{Zoned, civil, tz};
use rstest::rstest;
use tombi_date_time::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime, conversion};

#[rstest]
#[case("2024-02-29", "2024-02-29")]
#[case("0000-01-01", "0000-01-01")]
#[case("9999-12-31", "9999-12-31")]
fn local_date(#[case] input: &str, #[case] expected: &str) {
    let date = input.parse::<civil::Date>().unwrap();
    assert_eq!(LocalDate::try_from(date).unwrap().to_string(), expected);
}

#[rstest]
#[case("00:00:00")]
#[case("12:34:56.123456789")]
fn local_time(#[case] input: &str) {
    let time = input.parse::<civil::Time>().unwrap();
    assert_eq!(LocalTime::from(time).to_string(), input);
}

#[rstest]
#[case("2024-02-29T12:34:56.123456789")]
#[case("0000-01-01T00:00:00")]
fn local_date_time(#[case] input: &str) {
    let datetime = input.parse::<civil::DateTime>().unwrap();
    assert_eq!(
        LocalDateTime::try_from(datetime).unwrap().to_string(),
        input
    );
}

#[rstest]
#[case(-9999)]
#[case(-1)]
fn rejects_negative_year(#[case] year: i16) {
    let date = civil::Date::new(year, 1, 1).unwrap();
    assert_eq!(
        LocalDate::try_from(date),
        Err(conversion::Error::InvalidYear)
    );
    assert_eq!(
        LocalDateTime::try_from(date.at(0, 0, 0, 0)),
        Err(conversion::Error::InvalidYear),
    );
}

#[rstest]
#[case(0, "+00:00")]
#[case(19800, "+05:30")]
#[case(-12600, "-03:30")]
#[case(86400, "+24:00")]
#[case(-86400, "-24:00")]
fn offset_date_time_preserves_minutes_and_nanoseconds(#[case] seconds: i32, #[case] offset: &str) {
    let datetime = civil::date(2024, 2, 29).at(12, 34, 56, 123456789);
    let zoned = datetime
        .to_zoned(tz::TimeZone::fixed(
            tz::Offset::from_seconds(seconds).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        OffsetDateTime::try_from(zoned).unwrap().to_string(),
        format!("2024-02-29T12:34:56.123456789{offset}"),
    );
}

#[rstest]
#[case(1)]
#[case(-1)]
#[case(86460)]
#[case(-86460)]
fn rejects_unrepresentable_offset(#[case] seconds: i32) {
    let zoned = civil::date(2024, 2, 29)
        .at(12, 0, 0, 0)
        .to_zoned(tz::TimeZone::fixed(
            tz::Offset::from_seconds(seconds).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        OffsetDateTime::try_from(zoned),
        Err(conversion::Error::InvalidTimeZoneOffset)
    );
}

#[rstest]
#[case("2024-02-29T00:00:00+00:00[UTC]", "2024-02-29T00:00:00.000+00:00")]
#[case(
    "2024-02-29T12:34:56.123456789+05:30[+05:30]",
    "2024-02-29T12:34:56.123+05:30"
)]
fn completion_format(#[case] input: &str, #[case] expected: &str) {
    let zoned = input.parse::<Zoned>().unwrap();
    assert_eq!(
        zoned.strftime("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string(),
        expected
    );
    assert_eq!(
        LocalDate::try_from(zoned.clone()).unwrap().to_string(),
        &expected[..10]
    );
    assert_eq!(LocalDateTime::try_from(zoned.clone()).unwrap().year(), 2024);
    assert_eq!(OffsetDateTime::try_from(zoned).unwrap().year(), 2024);
}
