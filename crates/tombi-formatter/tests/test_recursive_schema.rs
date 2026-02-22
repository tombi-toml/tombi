mod recursive_schema {
    use tombi_formatter::{Formatter, test_format};
    use tombi_test_lib::recursive_schema_path;

    test_format! {
        #[tokio::test]
        async fn test_recursive_schema_simple(
            r#"
            name = "root"
            value = 1
            "#,
            SchemaPath(recursive_schema_path()),
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_recursive_schema_with_children(
            r#"
            name = "root"
            value = 1

            [[children]]
            name = "child1"
            value = 2

            [[children]]
            name = "child2"
            value = 3
            "#,
            SchemaPath(recursive_schema_path()),
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_recursive_schema_nested_children(
            r#"
            name = "root"
            value = 1

            [[children]]
            name = "child1"
            value = 2

            [[children.children]]
            name = "grandchild1"
            value = 4

            [[children]]
            name = "child2"
            value = 3
            "#,
            SchemaPath(recursive_schema_path()),
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_recursive_schema_with_metadata(
            r#"
            name = "root"
            value = 1

            [metadata]
            tags = ["a", "b"]
            "#,
            SchemaPath(recursive_schema_path()),
        ) -> Ok(source)
    }
}
