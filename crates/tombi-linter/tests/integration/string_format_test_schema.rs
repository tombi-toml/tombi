use tombi_linter::test_lint;
use tombi_test_lib::string_format_test_schema_path;
use tombi_x_keyword::StringFormat;

// --- ipv4 ---

test_lint! {
    #[test]
    fn test_ipv4_valid(
        r#"
        ipv4_addr = "192.168.1.1"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_ipv4_invalid(
        r#"
        ipv4_addr = "999.999.999.999"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::Ipv4,
            actual: "\"999.999.999.999\"".to_string(),
        },
    ])
}

// --- ipv6 ---

test_lint! {
    #[test]
    fn test_ipv6_valid(
        r#"
        ipv6_addr = "::1"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_ipv6_invalid(
        r#"
        ipv6_addr = "not-an-ipv6"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::Ipv6,
            actual: "\"not-an-ipv6\"".to_string(),
        },
    ])
}

// --- uri-reference ---

test_lint! {
    #[test]
    fn test_uri_reference_absolute(
        r#"
        uri_ref = "https://example.com/path"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_uri_reference_relative(
        r#"
        uri_ref = "/path/to/resource"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

// --- date-time (RFC 3339, timezone required) ---

test_lint! {
    #[test]
    fn test_date_time_valid(
        r#"
        date_time_val = "2024-01-15T10:30:00Z"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_date_time_invalid(
        r#"
        date_time_val = "not-a-date-time"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::DateTime,
            actual: "\"not-a-date-time\"".to_string(),
        },
    ])
}

// --- date-time-local (no timezone) ---

test_lint! {
    #[test]
    fn test_date_time_local_valid(
        r#"
        date_time_local_val = "2024-01-15T10:30:00"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_date_time_local_with_frac_valid(
        r#"
        date_time_local_val = "2024-01-15T10:30:00.123"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_date_time_local_invalid(
        r#"
        date_time_local_val = "not-a-local-date-time"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::DateTimeLocal,
            actual: "\"not-a-local-date-time\"".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_date_time_local_with_tz_invalid(
        r#"
        date_time_local_val = "2024-01-15T10:30:00Z"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::DateTimeLocal,
            actual: "\"2024-01-15T10:30:00Z\"".to_string(),
        },
    ])
}

// --- date (RFC 3339 full-date) ---

test_lint! {
    #[test]
    fn test_date_valid(
        r#"
        date_val = "2024-01-15"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_date_invalid(
        r#"
        date_val = "2024-13-01"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::Date,
            actual: "\"2024-13-01\"".to_string(),
        },
    ])
}

// --- time (RFC 3339 full-time, timezone required) ---

test_lint! {
    #[test]
    fn test_time_valid(
        r#"
        time_val = "10:30:00Z"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_time_invalid(
        r#"
        time_val = "25:00:00Z"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::Time,
            actual: "\"25:00:00Z\"".to_string(),
        },
    ])
}

// --- time-local (no timezone) ---

test_lint! {
    #[test]
    fn test_time_local_valid(
        r#"
        time_local_val = "10:30:00"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_time_local_with_frac_valid(
        r#"
        time_local_val = "10:30:00.123456"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_time_local_invalid(
        r#"
        time_local_val = "25:00:00"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::TimeLocal,
            actual: "\"25:00:00\"".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_time_local_with_tz_invalid(
        r#"
        time_local_val = "10:30:00Z"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::TimeLocal,
            actual: "\"10:30:00Z\"".to_string(),
        },
    ])
}

// --- regex ---

test_lint! {
    #[test]
    fn test_regex_valid(
        r#"
        regex_val = "^[a-z]+$"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_regex_invalid(
        r#"
        regex_val = "[invalid("
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::Regex,
            actual: "\"[invalid(\"".to_string(),
        },
    ])
}

// --- json-pointer ---

test_lint! {
    #[test]
    fn test_json_pointer_valid(
        r#"
        json_pointer_val = "/foo/bar/0"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_json_pointer_empty(
        r#"
        json_pointer_val = ""
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_json_pointer_invalid(
        r#"
        json_pointer_val = "no-leading-slash"
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::JsonPointer,
            actual: "\"no-leading-slash\"".to_string(),
        },
    ])
}

// --- TOML native date/time types ---
// Even when x-tombi-string-formats includes these formats,
// TOML native date/time values must be accepted.

test_lint! {
    #[test]
    fn test_date_time_native_valid(
        r#"
        date_time_val = 2024-01-15T10:30:00Z
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_date_time_local_native_valid(
        r#"
        date_time_local_val = 2024-01-15T10:30:00
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_date_native_valid(
        r#"
        date_val = 2024-01-15
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_time_local_native_valid(
        r#"
        time_local_val = 10:30:00
        "#,
        SchemaPath(string_format_test_schema_path()),
    ) -> Ok(_)
}
