use tombi_date_time::{LocalDate, LocalDateTime, LocalTime, OffsetDateTime, TimeZoneOffset};
use rstest::rstest;
use serde_json::json;

#[test]
fn test_local_date_deserialization() {
    let json = json!("2021-01-01");
    let date: LocalDate = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(date, LocalDate::from_ymd(2021, 1, 1));
}

#[test]
fn test_local_time_deserialization() {
    let json = json!("12:00:00");
    let time: LocalTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(time, LocalTime::from_hms(12, 0, 0));
}

#[rstest]
#[case("12:00:00.012", 12)]
#[case("12:00:00.123", 123)]
fn test_local_time_deserialization_with_milliseconds(
    #[case] input: &str,
    #[case] expected_milliseconds: u32,
) {
    let json = json!(input);
    let time: LocalTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(
        time,
        LocalTime::from_hms_milli(12, 0, 0, expected_milliseconds)
    );
}

#[rstest]
#[case("12:00:00.000001234", 1234)]
#[case("12:00:00.123456789", 123456789)]
#[case("12:00:00.999999999", 999999999)]
fn test_local_time_deserialization_with_nanoseconds(
    #[case] input: &str,
    #[case] expected_nanoseconds: u32,
) {
    let json = json!(input);
    let time: LocalTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(
        time,
        LocalTime::from_hms_nano(12, 0, 0, expected_nanoseconds)
    );
}

#[test]
fn test_local_date_time_deserialization() {
    let json = json!("2021-01-01T12:00:00");
    let date_time: LocalDateTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(date_time, LocalDateTime::from_ymd_hms(2021, 1, 1, 12, 0, 0));
}

#[rstest]
#[case("2021-01-01T12:00:00.012", 12)]
#[case("2021-01-01T12:00:00.123", 123)]
fn test_local_date_time_deserialization_with_milliseconds(
    #[case] input: &str,
    #[case] expected_milliseconds: u32,
) {
    let json = json!(input);
    let date_time: LocalDateTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(
        date_time,
        LocalDateTime::from_ymd_hms_milli(2021, 1, 1, 12, 0, 0, expected_milliseconds)
    );
}

#[rstest]
#[case("2021-01-01T12:00:00.000001234", 1234)]
#[case("2021-01-01T12:00:00.123456789", 123456789)]
#[case("2021-01-01T12:00:00.999999999", 999999999)]
fn test_local_date_time_deserialization_with_nanoseconds(
    #[case] input: &str,
    #[case] expected_nanoseconds: u32,
) {
    let json = json!(input);
    let date_time: LocalDateTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(
        date_time,
        LocalDateTime::from_ymd_hms_nano(2021, 1, 1, 12, 0, 0, expected_nanoseconds)
    );
}

#[rstest]
#[case("2021-01-01T12:00:00Z", TimeZoneOffset::Z)]
#[case("2021-01-01T12:00:00+00:00", TimeZoneOffset::Custom { minutes: 0 })]
#[case("2021-01-01T12:00:00+00:30", TimeZoneOffset::Custom { minutes: 30 })]
#[case("2021-01-01T12:00:00-00:30", TimeZoneOffset::Custom { minutes: -30 })]
fn test_offset_date_time_deserialization(
    #[case] input: &str,
    #[case] expected_offset: TimeZoneOffset,
) {
    let json = json!(input);
    let date_time: OffsetDateTime = serde_json::from_value(json).unwrap();
    pretty_assertions::assert_eq!(
        date_time,
        OffsetDateTime::from_ymd_hms(2021, 1, 1, 12, 0, 0, expected_offset)
    );
}

#[rstest]
#[case(json!("invalid-date-time"), tombi_date_time::parse::Error::InvalidFormat)]
#[case(json!("2021-01-01T12:00:00+25:00"), tombi_date_time::parse::Error::InvalidTimeZoneOffsetHour)]
#[case(json!(true), "invalid type: boolean `true`, expected a TOML DateTime")]
fn test_invalid_date_time_deserialization(
    #[case] input: serde_json::Value,
    #[case] expected_error: impl ToString,
) {
    let result: Result<LocalDateTime, _> = serde_json::from_value(input);
    assert!(result.is_err());
    pretty_assertions::assert_eq!(result.unwrap_err().to_string(), expected_error.to_string());
}
