use std::path::PathBuf;

use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("numeric-bounds-test.schema.json")
}

test_lint! {
    #[test]
    fn test_numeric_bounds_accept_satisfied_relations(
        r#"
        integer_maximum = 10
        integer_minimum = 10
        integer_exclusive_maximum_normal = 9
        integer_exclusive_minimum_normal = 11
        float_maximum = 10.0
        float_minimum = 10.0
        float_exclusive_maximum = 9.5
        float_exclusive_minimum = 10.5
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_integer_maximum_is_inclusive(
        "integer_maximum = 11",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be ≤ 10, but found 11"])
}

test_lint! {
    #[test]
    fn test_integer_minimum_is_inclusive(
        "integer_minimum = 9",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be ≥ 10, but found 9"])
}

test_lint! {
    #[test]
    fn test_integer_exclusive_maximum_uses_schema_boundary(
        "integer_exclusive_maximum = -9223372036854775808",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be < -9223372036854775808, but found -9223372036854775808"])
}

test_lint! {
    #[test]
    fn test_integer_exclusive_minimum_uses_schema_boundary(
        "integer_exclusive_minimum = 9223372036854775807",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be > 9223372036854775807, but found 9223372036854775807"])
}

test_lint! {
    #[test]
    fn test_float_maximum_is_inclusive(
        "float_maximum = 10.5",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be ≤ 10, but found 10.5"])
}

test_lint! {
    #[test]
    fn test_float_minimum_is_inclusive(
        "float_minimum = 9.5",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be ≥ 10, but found 9.5"])
}

test_lint! {
    #[test]
    fn test_float_exclusive_maximum_is_exclusive(
        "float_exclusive_maximum = 10.0",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be < 10, but found 10"])
}

test_lint! {
    #[test]
    fn test_float_exclusive_minimum_is_exclusive(
        "float_exclusive_minimum = 10.0",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be > 10, but found 10"])
}

test_lint! {
    #[test]
    fn test_float_maximum_rejects_nan(
        "float_maximum = nan",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be ≤ 10, but found NaN"])
}

test_lint! {
    #[test]
    fn test_float_minimum_rejects_nan(
        "float_minimum = nan",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be ≥ 10, but found NaN"])
}

test_lint! {
    #[test]
    fn test_float_exclusive_maximum_rejects_nan(
        "float_exclusive_maximum = nan",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be < 10, but found NaN"])
}

test_lint! {
    #[test]
    fn test_float_exclusive_minimum_rejects_nan(
        "float_exclusive_minimum = nan",
        SchemaPath(schema_path()),
    ) -> Err(["the value must be > 10, but found NaN"])
}
