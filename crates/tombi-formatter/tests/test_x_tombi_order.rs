mod table_keys_order {
    use tombi_formatter::{test_format, Formatter};

    mod pyproject {
        use super::*;
        use tombi_test_lib::pyproject_schema_path;

        test_format! {
            #[tokio::test]
            async fn test_project(
                r#"
                [project]
                version = "0.1.0"
                readme = "README.md"
                description = "A test project"
                name = "test-project"
                requires-python = ">=3.8"
                authors = [
                    {name = "Test Author", email = "test@example.com"}
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "test-project"
                version = "0.1.0"
                description = "A test project"
                readme = "README.md"
                requires-python = ">=3.8"
                authors = [{ name = "Test Author", email = "test@example.com" }]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_project_dependencies_single_line(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = ["tombi-cli>=0.0.0", "maturin>=1.5,<2.0"]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = ["maturin>=1.5,<2.0", "tombi-cli>=0.0.0"]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_project_dependencies_single_line_with_comma(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = ["tombi-cli>=0.0.0", "maturin>=1.5,<2.0",]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = [
                  "maturin>=1.5,<2.0",
                  "tombi-cli>=0.0.0",
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_project_dependencies_multiple_lines(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = [
                  "tombi-linter>=0.0.0",
                  "tombi-formatter>=0.0.0",
                  "maturin>=1.5,<2.0",
                  "tombi-cli>=0.0.0"
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = [
                  "maturin>=1.5,<2.0",
                  "tombi-cli>=0.0.0",
                  "tombi-formatter>=0.0.0",
                  "tombi-linter>=0.0.0"
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_project_dependencies_multiple_lines_with_comment(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = [
                  "tombi-linter>=0.0.0",
                  "tombi-formatter>=0.0.0",
                  # maturin leading comment1
                  # maturin leading comment2
                  "maturin>=1.5,<2.0", # maturin trailing comment
                  # tombi-cli leading comment1
                  # tombi-cli leading comment2
                  "tombi-cli>=0.0.0" # tombi-cli trailing comment
                  ,
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                description = "Reserved package for tombi"
                requires-python = ">=3.10"
                dependencies = [
                  # maturin leading comment1
                  # maturin leading comment2
                  "maturin>=1.5,<2.0",  # maturin trailing comment
                  # tombi-cli leading comment1
                  # tombi-cli leading comment2
                  "tombi-cli>=0.0.0",  # tombi-cli trailing comment
                  "tombi-formatter>=0.0.0",
                  "tombi-linter>=0.0.0",
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_dependency_groups_multiple_lines_with_comment(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                requires-python = ">=3.10"
                dependencies = []

                [dependency-groups]
                dev = [
                  "pytest>=8.3.3", # pytest trailing comment
                  "ruff>=0.7.4"
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                requires-python = ">=3.10"
                dependencies = []

                [dependency-groups]
                dev = [
                  "pytest>=8.3.3",  # pytest trailing comment
                  "ruff>=0.7.4"
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_dependency_groups_multiple_lines_include_group(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                requires-python = ">=3.10"
                dependencies = []

                [dependency-groups]
                dev = [
                  { include-group = "stub" },
                  "pytest>=8.3.3",
                  { include-group = "ci" },
                  "ruff>=0.7.4",
                ]
                ci = [
                  "ruff>=0.7.4",
                  "pytest-ci>=0.0.0",
                ]
                stub = [
                  "pytest-stub>=1.1.0",
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "tombi"
                version = "1.0.0"
                requires-python = ">=3.10"
                dependencies = []

                [dependency-groups]
                dev = [
                  "pytest>=8.3.3",
                  "ruff>=0.7.4",
                  { include-group = "ci" },
                  { include-group = "stub" },
                ]
                ci = [
                  "pytest-ci>=0.0.0",
                  "ruff>=0.7.4",
                ]
                stub = [
                  "pytest-stub>=1.1.0",
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_dependency_groups_multiple_lines_include_group_with_comment_directive_array_values_order_ascending(
                r#"
                [dependency-groups]
                # tombi: format.rules.array-values-order = "ascending"
                dev = [
                  { include-group = "stub" },
                  "pytest>=8.3.3",
                  { include-group = "ci" },
                  "ruff>=0.7.4",
                ]
                ci = [
                  "ruff>=0.7.4",
                  "pytest-ci>=0.0.0",
                ]
                stub = [
                  "pytest-stub>=1.1.0",
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [dependency-groups]
                # tombi: format.rules.array-values-order = "ascending"
                dev = [
                  { include-group = "ci" },
                  "pytest>=8.3.3",
                  "ruff>=0.7.4",
                  { include-group = "stub" },
                ]
                ci = [
                  "pytest-ci>=0.0.0",
                  "ruff>=0.7.4",
                ]
                stub = [
                  "pytest-stub>=1.1.0",
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_tool_poetry_dependencies(
                r#"
                [project]
                name = "test-project"
                version = "0.1.0"
                description = "A test project"
                authors = [{ name = "test-user" }]
                readme = "README.md"

                [tool.poetry.dependencies]
                python = ">=3.11 <3.13"
                pydantic = "^2.5"
                pandas = "^2.2.0"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [project]
                name = "test-project"
                version = "0.1.0"
                description = "A test project"
                readme = "README.md"
                authors = [{ name = "test-user" }]

                [tool.poetry.dependencies]
                pandas = "^2.2.0"
                pydantic = "^2.5"
                python = ">=3.11 <3.13"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_tool_mypy_overrides(
                r#"
                [[tool.mypy.overrides]]
                module = [
                    "pendulum.mixins.default",
                    "tests.test_parsing",
                    "tests.date.test_add",
                    "tests.date.test_behavior",
                    "tests.date.test_construct",
                    "tests.date.test_comparison",
                    "tests.date.test_day_of_week_modifiers",
                    "tests.date.test_diff",
                ]
                ignore_errors = true
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [[tool.mypy.overrides]]
                module = [
                  "pendulum.mixins.default",
                  "tests.test_parsing",
                  "tests.date.test_add",
                  "tests.date.test_behavior",
                  "tests.date.test_construct",
                  "tests.date.test_comparison",
                  "tests.date.test_day_of_week_modifiers",
                  "tests.date.test_diff",
                ]
                ignore_errors = true
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_tool_maturin_include(
                r#"
                [tool.maturin]
                include = [
                  { path = "www.schemastore.org/**/*.json", format = "sdist" },
                  { path = "json.tombi.dev/**/*.json", format = "sdist" },
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(
                r#"
                [tool.maturin]
                include = [
                  { format = "sdist", path = "www.schemastore.org/**/*.json" },
                  { format = "sdist", path = "json.tombi.dev/**/*.json" },
                ]
                "#
            )
        }
    }

    mod cargo {
        use tombi_test_lib::cargo_schema_path;

        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_cargo_package(
                r#"
                [package]
                name = "toml-version"
                authors.workspace = true
                edition.workspace = true
                license.workspace = true
                repository.workspace = true
                version.workspace = true
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [package]
                name = "toml-version"
                version.workspace = true
                authors.workspace = true
                edition.workspace = true
                repository.workspace = true
                license.workspace = true
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_cargo_package2(
                r#"
                [package]
                name = "toml-version"
                authors = { workspace = true }
                edition = { workspace = true }
                license = { workspace = true }
                repository = { workspace = true }
                version = { workspace = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [package]
                name = "toml-version"
                version = { workspace = true }
                authors = { workspace = true }
                edition = { workspace = true }
                repository = { workspace = true }
                license = { workspace = true }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_cargo_package_with_disabled_comment_directive(
                r#"
                # tombi: format.rules.table-keys-order.disabled = true
                [package]
                name = "toml-version"
                authors = { workspace = true }
                edition = { workspace = true }
                license = { workspace = true }
                repository = { workspace = true }
                version = { workspace = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                # tombi: format.rules.table-keys-order.disabled = true
                [package]
                name = "toml-version"
                authors = { workspace = true }
                edition = { workspace = true }
                license = { workspace = true }
                repository = { workspace = true }
                version = { workspace = true }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_cargo_package_with_ascending_comment_directive(
                r#"
                # tombi: format.rules.table-keys-order = "ascending"
                [package]
                name = "toml-version"
                authors = { workspace = true }
                edition = { workspace = true }
                license = { workspace = true }
                repository = { workspace = true }
                version = { workspace = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                # tombi: format.rules.table-keys-order = "ascending"
                [package]
                authors = { workspace = true }
                edition = { workspace = true }
                license = { workspace = true }
                name = "toml-version"
                repository = { workspace = true }
                version = { workspace = true }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_dependencies_and_features(
                r#"
                [features]
                default = ["clap"]
                clap = ["clap/derive"]

                [dependencies]
                serde = { features = ["derive"], version = "^1.0.0" }
                clap = { version = "4.5.0" }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [dependencies]
                clap = { version = "4.5.0" }
                serde = { version = "^1.0.0", features = ["derive"] }

                [features]
                default = ["clap"]
                clap = ["clap/derive"]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_default_feature(
                r#"
                [features]
                wasm = ["tombi-schema-store/wasm"]
                clap = ["dep:clap"]
                default = ["clap", "native"]
                native = ["tombi-schema-store/native"]
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [features]
                default = ["clap", "native"]
                clap = ["dep:clap"]
                native = ["tombi-schema-store/native"]
                wasm = ["tombi-schema-store/wasm"]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_cargo_dependencies(
                r#"
                [dependencies]
                serde = { features = ["derive"], version = "^1.0.0" }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = ["derive"] }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_cargo_dependencies_trailing_comma(
                r#"
                [dependencies]
                serde = { features = ["std", "derive",], version = "^1.0.0" }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = [
                  "derive",
                  "std",
                ] }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_cargo_dependencies_trailing_comma_with_comment_directive(
                r#"
                [dependencies]
                serde = { features = [
                  # tombi: format.rules.array-values-order.disabled = true

                  "std", "derive",
                ], version = "^1.0.0" }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = [
                  # tombi: format.rules.array-values-order.disabled = true

                  "std",
                  "derive",
                ] }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_workspace_dependencies(
                r#"
                [workspace.dependencies]
                serde.version = "^1.0.0"
                serde.features = ["derive"]
                serde.workspace = true
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [workspace.dependencies]
                serde.workspace = true
                serde.version = "^1.0.0"
                serde.features = ["derive"]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_workspace_dependencies_complex(
                r#"
                [workspace.dependencies]
                serde.version = "^1.0.0"
                serde.workspace = true
                serde.features = ["derive"]
                anyhow = "1.0.89"
                chrono = { version = "0.4.38", features = ["serde"] }
                reqwest.default-features = false
                reqwest.version = "0.12.9"
                reqwest.features = ["json", "rustls-tls"]
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok(
                r#"
                [workspace.dependencies]
                anyhow = "1.0.89"
                chrono = { version = "0.4.38", features = ["serde"] }
                reqwest.version = "0.12.9"
                reqwest.default-features = false
                reqwest.features = ["json", "rustls-tls"]
                serde.workspace = true
                serde.version = "^1.0.0"
                serde.features = ["derive"]
                "#
            )
        }
    }

    mod tombi {
        use super::*;
        use tombi_test_lib::tombi_schema_path;

        test_format! {
            #[tokio::test]
            async fn test_tombi(
                r#"
                [[schemas]]
                include = ["*.toml"]
                path = "pyproject.toml"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok(
                r#"
                [[schemas]]
                path = "pyproject.toml"
                include = ["*.toml"]
                "#
            )
        }
    }

    mod type_test {
        use tombi_test_lib::type_test_schema_path;

        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_array_sort(
                r#"
                [[array]]
                integer = 1

                [[array]]
                integer = 2

                [array.table]
                key = "value"

                [[array]]
                integer = 3
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok(source)
        }

        test_format! {
            #[tokio::test]
            async fn test_nested_array_sort(
                r#"
                [[array1]]
                integer = 1

                [[array1]]
                integer = 2

                [array1.table1]
                key1 = "2"

                [[array1.table1.array2]]
                key2 = "1"

                [[array1.table1.array2]]
                key2 = "2"

                [array1.table1.array2.table2]
                key3 = "1"

                [[array1]]
                integer = 3
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok(source)
        }
    }

    mod non_schema {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_header_order(
                r#"
                key2.key3 = "value1"
                key1 = "value2"
                key2.key4 = "value3"
                key4 = "value4"
                key5 = "value5"

                [aaa]
                key1 = "value1"
                key2 = "value2"

                [bbb]
                key3 = "value3"
                key4 = "value4"

                [aaa.ccc]
                key5 = "value5"
                "#,
            ) -> Ok(r#"
                key2.key3 = "value1"
                key2.key4 = "value3"
                key1 = "value2"
                key4 = "value4"
                key5 = "value5"

                [aaa]
                key1 = "value1"
                key2 = "value2"

                [aaa.ccc]
                key5 = "value5"

                [bbb]
                key3 = "value3"
                key4 = "value4"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_comment_directive_sort(
                r#"
                # tombi: format.rules.table-keys-order = "descending"

                key2.key3 = "value1"
                key1 = "value2"
                key2.key4 = "value3"
                key4 = "value4"
                key5 = "value5"

                [aaa]
                key1 = "value1"

                [bbb]
                key2 = "value2"

                [ccc]
                key3 = "value3"

                [ccc.ddd]
                key4 = "value4"

                [ccc.eee]
                key5 = "value5"
                "#,
            ) -> Ok(r#"
                # tombi: format.rules.table-keys-order = "descending"

                key5 = "value5"
                key4 = "value4"
                key2.key4 = "value3"
                key2.key3 = "value1"
                key1 = "value2"

                [ccc]
                key3 = "value3"

                [ccc.eee]
                key5 = "value5"

                [ccc.ddd]
                key4 = "value4"

                [bbb]
                key2 = "value2"

                [aaa]
                key1 = "value1"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_of_tables(
                r#"
                [[aaa]]
                key1 = "value1"
                key2 = "value2"

                [[aaa]]
                key1 = "value3"
                key2 = "value4"

                [aaa.key3]
                key4 = "value5"

                [[aaa]]
                key1 = "value6"
                key2 = "value7"

                [[aaa]]
                key1 = "value8"
                key2 = "value9"
                "#,
            ) -> Ok(source)
        }

        test_format! {
            #[tokio::test]
            async fn test_array_with_leading_comment_directive(
                r#"
                # tombi: format.rules.array-values-order = "ascending"
                key = [

                  5, 4, 3
                ]
                "#,
            ) -> Ok(
                r#"
                # tombi: format.rules.array-values-order = "ascending"
                key = [3, 4, 5]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_with_inner_comment_directive(
                r#"
                key = [
                  # tombi: format.rules.array-values-order = "ascending"

                  5, 4, 3
                ]
                "#,
            ) -> Ok(
                r#"
                key = [
                  # tombi: format.rules.array-values-order = "ascending"

                  3,
                  4,
                  5
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_with_trailing_comment_directive(
                r#"
                key = [

                  5, 4, 3,
                ]  # tombi: format.rules.array-values-order = "ascending"
                "#,
            ) -> Ok(
                r#"
                key = [
                  3,
                  4,
                  5,
                ]  # tombi: format.rules.array-values-order = "ascending"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_with_inner_comment_directive_with_trailing_comment(
                r#"
                key = [
                  # tombi: format.rules.array-values-order = "ascending"

                  # leading comment1
                  5 # trailing comment1

                  # leading comment2
                  , # trailing comment2
                  4, # trailing comment3
                  3 # trailing comment4
                ]
                "#,
            ) -> Ok(
                r#"
                key = [
                  # tombi: format.rules.array-values-order = "ascending"

                  3,  # trailing comment4
                  4,  # trailing comment3
                  # leading comment1
                  5  # trailing comment1
                  # leading comment2
                  ,  # trailing comment2
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_inline_table_with_leading_comment_directive(
                r#"
                # tombi: format.rules.table-keys-order = "ascending"
                key = { key5 = 5, key4 = 4, key3 = 3 }
                "#,
            ) -> Ok(
                r#"
                # tombi: format.rules.table-keys-order = "ascending"
                key = { key3 = 3, key4 = 4, key5 = 5 }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_inline_table_with_inner_comment_directive(
                r#"
                key = {
                  # tombi: format.rules.table-keys-order = "ascending"

                  key5 = 5, key4 = 4, key3 = 3
                }
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
            ) -> Ok(
                r#"
                key = {
                  # tombi: format.rules.table-keys-order = "ascending"

                  key3 = 3,
                  key4 = 4,
                  key5 = 5
                }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_inline_table_with_trailing_comment_directive(
                r#"
                key = {
                  key5 = 5, key4 = 4, key3 = 3,
                }  # tombi: format.rules.table-keys-order = "ascending"
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
            ) -> Ok(
                r#"
                key = {
                  key3 = 3,
                  key4 = 4,
                  key5 = 5,
                }  # tombi: format.rules.table-keys-order = "ascending"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_inline_table_with_inner_comment_directive_with_trailing_comment(
                r#"
                key = {
                  # tombi: format.rules.table-keys-order = "ascending"

                  # leading comment1
                  key5 = 5 # trailing comment1

                  # leading comment2
                  , # trailing comment2
                  key4 = 4, # trailing comment3
                  key3 = 3 # trailing comment4
                }
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
            ) -> Ok(
                r#"
                key = {
                  # tombi: format.rules.table-keys-order = "ascending"

                  key3 = 3,  # trailing comment4
                  key4 = 4,  # trailing comment3
                  # leading comment1
                  key5 = 5  # trailing comment1
                  # leading comment2
                  ,  # trailing comment2
                }
                "#
            )
        }
    }

    mod file_schema {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_comment_sort1(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json

                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # root key values begin dangling comment3
                # root key values begin dangling comment4

                # table b header leading comment
                [b] # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b" # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4

                # table a header leading comment
                [a] # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a" # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4
                "#,
            ) -> Ok(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json

                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # root key values begin dangling comment3
                # root key values begin dangling comment4

                # table a header leading comment
                [a]  # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a"  # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4

                # table b header leading comment
                [b]  # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b"  # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_comment_sort2(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json

                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # root key values begin dangling comment3
                # root key values begin dangling comment4

                key1 = "value1"
                key2 = "value2"

                # table b header leading comment
                [b] # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b" # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4

                # table a header leading comment
                [a] # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a" # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4
                "#,
            ) -> Ok(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json

                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # root key values begin dangling comment3
                # root key values begin dangling comment4

                key1 = "value1"
                key2 = "value2"

                # table a header leading comment
                [a]  # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a"  # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4

                # table b header leading comment
                [b]  # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b"  # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_comment_sort3(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json

                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # root key values begin dangling comment3
                # root key values begin dangling comment4

                key1 = "value1"
                key2 = "value2"

                # root key values end dangling comment1
                # root key values end dangling comment2

                # root key values end dangling comment3
                # root key values end dangling comment4

                # table b header leading comment
                [b] # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b" # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4

                # table a header leading comment
                [a] # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a" # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4
                "#,
            ) -> Ok(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json

                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # root key values begin dangling comment3
                # root key values begin dangling comment4

                key1 = "value1"
                key2 = "value2"

                # root key values end dangling comment1
                # root key values end dangling comment2

                # root key values end dangling comment3
                # root key values end dangling comment4

                # table a header leading comment
                [a]  # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a"  # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4

                # table b header leading comment
                [b]  # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b"  # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_comment_sort4(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json
                # root key values begin dangling comment1
                # root key values begin dangling comment2
                [b] # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b" # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4

                # table a header leading comment
                [a] # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a" # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4
                "#,
            ) -> Ok(
                r#"
                #:schema ./schemas/x-tombi-table-keys-order.schema.json
                # root key values begin dangling comment1
                # root key values begin dangling comment2

                # table a header leading comment
                [a]  # table a header trailing comment
                # table a key values begin dangling comment1
                # table a key values begin dangling comment2

                # table a key values begin dangling comment3
                # table a key values begin dangling comment4

                # key_a leading comment1
                key_a = "a"  # key_a trailing comment1

                # table a key values end dangling comment1
                # table a key values end dangling comment2

                # table a key values end dangling comment3
                # table a key values end dangling comment4

                [b]  # table b header trailing comment
                # table b key values begin dangling comment1
                # table b key values begin dangling comment2

                # table b key values begin dangling comment3
                # table b key values begin dangling comment4

                # key_b leading comment1
                key_b = "b"  # key_b trailing comment1

                # table b key values end dangling comment1
                # table b key values end dangling comment2

                # table b key values end dangling comment3
                # table b key values end dangling comment4
                "#
            )
        }
    }

    #[macro_export]
    macro_rules! test_format {
        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                TomlVersion($toml_version:expr),
                $schema_path:expr$(,)?
            ) -> Ok(source)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    $toml_version,
                    Some($schema_path),
                ) -> Ok($source)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                TomlVersion($toml_version:expr),
            ) -> Ok(source)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    $toml_version,
                    Option::<&std::path::Path>::None,
                ) -> Ok($source)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                $schema_path:expr$(,)?
            ) -> Ok(source)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    TomlVersion::default(),
                    Some($schema_path),
                ) -> Ok($source)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                TomlVersion($toml_version:expr),
                $schema_path:expr$(,)?
            ) -> Ok($expected:expr$(,)?)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    $toml_version,
                    Some($schema_path),
                ) -> Ok($expected)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                TomlVersion($toml_version:expr),
            ) -> Ok($expected:expr$(,)?)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    $toml_version,
                    Option::<&std::path::Path>::None,
                ) -> Ok($expected)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                $schema_path:expr$(,)?
            ) -> Ok($expected:expr$(,)?)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    TomlVersion::default(),
                    Some($schema_path),
                ) -> Ok($expected)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
            ) -> Ok(source)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    TomlVersion::default(),
                    Option::<&std::path::Path>::None,
                ) -> Ok($source)
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
            ) -> Ok($expected:expr$(,)?)
        ) => {
            test_format! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    TomlVersion::default(),
                    Option::<&std::path::Path>::None,
                ) -> Ok($expected)
            }
        };

        (
            #[tokio::test]
            async fn _$name:ident(
                $source:expr,
                $toml_version:expr,
                $schema_path:expr,
            ) -> Ok($expected:expr$(,)?)
        ) => {
            #[tokio::test]
            async fn $name() {
                use textwrap::dedent;
                use tombi_config::TomlVersion;
                use tombi_formatter::{FormatOptions, Formatter};
                use tombi_schema_store::SchemaStore;

                tombi_test_lib::init_tracing();

                // Initialize schema store
                let schema_store = SchemaStore::new();

                if let Some(schema_path) = $schema_path {
                    let path = tombi_uri::Uri::from_file_path(schema_path)
                        .unwrap()
                        .to_string();
                    // Load schemas
                    schema_store
                        .load_config_schemas(
                            &[tombi_config::Schema::Root(tombi_config::RootSchema {
                                toml_version: None,
                                path,
                                include: vec!["*.toml".to_string()],
                            })],
                            None,
                        )
                        .await;
                }

                // Initialize formatter
                let format_options = FormatOptions::default();
                let source_path = tombi_test_lib::project_root_path().join("test.toml");
                let formatter = Formatter::new(
                    $toml_version,
                    &format_options,
                    Some(itertools::Either::Right(source_path.as_path())),
                    &schema_store,
                );

                // Test that keys are reordered according to schema order
                let source = dedent($source).trim().to_string();
                let expected = dedent($expected).trim().to_string() + "\n";

                let formatted = formatter.format(&source).await.unwrap();
                pretty_assertions::assert_eq!(formatted, expected);
            }
        };
    }
}
