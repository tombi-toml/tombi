use tombi_date_time::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime, TimeZoneOffset};
use rstest::rstest;
use serde_json::json;

#[test]
fn test_local_date_serialization() {
    let date = LocalDate::from_ymd(2021, 1, 1);

    pretty_assertions::assert_eq!(serde_json::to_value(date).unwrap(), json!("2021-01-01"));
}

#[test]
fn test_local_time_serialization() {
    let time = LocalTime::from_hms(12, 0, 0);

    pretty_assertions::assert_eq!(serde_json::to_value(time).unwrap(), json!("12:00:00"));
}

#[rstest]
#[case(12, "12:00:00.012")]
#[case(123, "12:00:00.123")]
fn test_local_time_serialization_with_milliseconds(
    #[case] milliseconds: u32,
    #[case] expected: &str,
) {
    let time = LocalTime::from_hms_milli(12, 0, 0, milliseconds);

    pretty_assertions::assert_eq!(serde_json::to_value(time).unwrap(), json!(expected));
}

#[rstest]
#[case(1234, "12:00:00.000001234")]
#[case(123456789, "12:00:00.123456789")]
#[case(999999999, "12:00:00.999999999")]
fn test_local_time_serialization_with_nanoseconds(
    #[case] nanoseconds: u32,
    #[case] expected: &str,
) {
    let time = LocalTime::from_hms_nano(12, 0, 0, nanoseconds);

    pretty_assertions::assert_eq!(serde_json::to_value(time).unwrap(), json!(expected));
}

#[test]
fn test_local_date_time_serialization() {
    let date_time = LocalDateTime::from_ymd_hms(2021, 1, 1, 12, 0, 0);

    pretty_assertions::assert_eq!(
        serde_json::to_value(date_time).unwrap(),
        json!("2021-01-01T12:00:00")
    );
}

#[rstest]
#[case(12, "2021-01-01T12:00:00.012")]
#[case(123, "2021-01-01T12:00:00.123")]
fn test_local_date_time_serialization_with_milliseconds(
    #[case] milliseconds: u32,
    #[case] expected: &str,
) {
    let date_time = LocalDateTime::from_ymd_hms_milli(2021, 1, 1, 12, 0, 0, milliseconds);

    pretty_assertions::assert_eq!(serde_json::to_value(date_time).unwrap(), json!(expected));
}

#[rstest]
#[case(1234, "2021-01-01T12:00:00.000001234")]
#[case(123456789, "2021-01-01T12:00:00.123456789")]
#[case(999999999, "2021-01-01T12:00:00.999999999")]
fn test_local_date_time_serialization_with_nanoseconds(
    #[case] nanoseconds: u32,
    #[case] expected: &str,
) {
    let date_time = LocalDateTime::from_ymd_hms_nano(2021, 1, 1, 12, 0, 0, nanoseconds);

    pretty_assertions::assert_eq!(serde_json::to_value(date_time).unwrap(), json!(expected));
}

#[rstest]
#[case(TimeZoneOffset::Z, "2021-01-01T12:00:00Z")]
#[case(TimeZoneOffset::Custom { minutes: 0 }, "2021-01-01T12:00:00+00:00")]
#[case(TimeZoneOffset::Custom { minutes: 30 }, "2021-01-01T12:00:00+00:30")]
#[case(TimeZoneOffset::Custom { minutes: -30 }, "2021-01-01T12:00:00-00:30")]
fn test_offset_date_time_serialization(#[case] offset: TimeZoneOffset, #[case] expected: &str) {
    let date_time = OffsetDateTime::from_ymd_hms(2021, 1, 1, 12, 0, 0, offset);

    pretty_assertions::assert_eq!(serde_json::to_value(date_time).unwrap(), json!(expected));
}
