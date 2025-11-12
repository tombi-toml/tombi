mod format;
pub mod formatter;
mod types;

use format::Format;
pub use formatter::definitions::FormatDefinitions;
pub use formatter::Formatter;
pub use tombi_config::FormatOptions;

#[macro_export]
macro_rules! test_format {
    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
            FormatOptions($options:expr)$(,)?
            SchemaPath($schema_path:expr)$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                $options,
                Some($schema_path),
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
            FormatOptions($options:expr)$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                $options,
                Option::<&std::path::Path>::None,
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
            SchemaPath($schema_path:expr)$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                Default::default(),
                Some($schema_path),
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr)$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                Default::default(),
                Option::<&std::path::Path>::None,
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            FormatOptions($options:expr)$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                $options,
                Option::<&std::path::Path>::None,
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            SchemaPath($schema_path:expr)$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                Default::default(),
                Some($schema_path),
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
            FormatOptions($options:expr)$(,)?
            SchemaPath($schema_path:expr)$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                $options,
                Some($schema_path),
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
            FormatOptions($options:expr)$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                $options,
                Option::<&std::path::Path>::None,
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
            SchemaPath($schema_path:expr)$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                Default::default(),
                Some($schema_path),
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr)$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                $toml_version,
                Default::default(),
                Option::<&std::path::Path>::None,
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            FormatOptions($options:expr)$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                $options,
                Option::<&std::path::Path>::None,
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            SchemaPath($schema_path:expr)$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                Default::default(),
                Some($schema_path),
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr$(,)?
        ) -> Ok(source)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                Default::default(),
                Option::<&std::path::Path>::None,
            ) -> Ok($source)
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr$(,)?
        ) -> Ok($expected:expr)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                Default::default(),
                Option::<&std::path::Path>::None,
            ) -> Ok($expected)
        }
    };

    (
        #[tokio::test]
        async fn _$name:ident(
            $source:expr,
            $toml_version:expr,
            $options:expr,
            $schema_path:expr,
        ) -> Ok($expected:expr)
    ) => {
        #[tokio::test]
        async fn $name() {
            use textwrap::dedent;
            use tombi_config::TomlVersion;
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
            let format_options = $options;
            let source_path = tombi_test_lib::project_root_path().join("test.toml");
            let formatter = Formatter::new(
                $toml_version,
                &format_options,
                Some(itertools::Either::Right(source_path.as_path())),
                &schema_store,
            );

            // Test that keys are reordered according to schema order
            let source = dedent($source).to_string();
            let mut expected = dedent($expected).trim().to_string();
            if !source.trim().is_empty() {
                expected.push('\n');
            }

            let formatted = formatter.format(&source).await.unwrap();
            pretty_assertions::assert_eq!(formatted, expected);
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr$(,)?
        ) -> Err(_)
    ) => {
        test_format! {
            #[tokio::test]
            async fn _$name(
                $source,
                TomlVersion::default(),
                Default::default(),
                Option::<&std::path::Path>::None,
            ) -> Err(_)
        }
    };

    (
        #[tokio::test]
        async fn _$name:ident(
            $source:expr,
            $toml_version:expr,
            $options:expr,
            $schema_path:expr,
        ) -> Err(_)
    ) => {
        #[tokio::test]
        async fn $name() {
            use tombi_config::TomlVersion;
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
            let format_options = $options;
            let source_path = tombi_test_lib::project_root_path().join("test.toml");
            let formatter = Formatter::new(
                $toml_version,
                &format_options,
                Some(itertools::Either::Right(source_path.as_path())),
                &schema_store,
            );

            // Test that keys are reordered according to schema order
            let source = textwrap::dedent($source).trim().to_string();
            match formatter.format(&source).await {
                Ok(_) => panic!("Expected error, got success"),
                Err(errors) => {
                    pretty_assertions::assert_ne!(errors, vec![]);
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use tombi_config::{format::FormatRules, QuoteStyle};

    test_format! {
        #[tokio::test]
        async fn test_empty(
            r#""#,
            TomlVersion(TomlVersion::V1_0_0)
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_whitespace(
            r#"    "#,
            TomlVersion(TomlVersion::V1_0_0)
        ) -> Ok("")
    }

    test_format! {
        #[tokio::test]
        async fn test_only_comment1(
            r#"
            # comment1
            # comment2
            "#,
            TomlVersion(TomlVersion::V1_0_0)
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_only_comment2(
            r#"
            # comment1
            # comment2

            # comment3
            # comment4
            "#,
            TomlVersion(TomlVersion::V1_0_0)
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_key_values(r#"
            array5 = [
              1,
              {
                # inline begin dangling comment1
                # inline begin dangling comment2

                # key1 leading comment1
                # key1 leading comment2
                key1 = 1,  # key1 trailing comment
                # key2 leading comment1
                key2 = 2,  # key2 trailing comment

                # inline end dangling comment1
                # inline end dangling comment2
              },

              # comment
            ]
            "#,
            TomlVersion(TomlVersion::V1_1_0_Preview)
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_sample_toml(
r#"
# key values begin dangling comment1
# key values begin dangling comment2

# key values begin dangling comment3
# key values begin dangling comment4

key1 = 1
key2 = "2"

# key values end dangling comment1
# key values end dangling comment2

# key values end dangling comment3
# key values end dangling comment4

# table leading comment1
# table leading comment2
[aaaa]
# table leading comment1
# table leading comment2
[aaaa.bbb]
bool1 = true
bool2 = false
dec = 1  # dec trailing comment
bin = 0b01  # bin trailing comment
oct = 0o01  # oct trailing comment
hex = 0x01  # hex trailing comment
float1 = 0.1234  # float trailing comment
infa = inf
bs = "2"  # bs trailing comment
ls = '3'  # ls trailing comment
array1 = [
  # array begin dangling comment1
  # array begin dangling comment2

  # value1 leading comment1
  # value1 leading comment2
  # value1 leading comment3
  { key3 = 12, key4 = 2024-01-01T10:10:00 }
  # value1 comma leading comment1
  # value1 comma leading comment2
  # value1 comma leading comment3
  ,  # value1 comma trailing comment
  { key3 = 11, key4 = 2024-01-01T10:10:00 },

  # array end dangling comment1
  # array end dangling comment2
]  # array trailing comment
array2 = [1, 2, 3]
array3 = [
  1,
  2,
  3,
]
array4 = [
  [
    1,
    2,
    3,
  ],
  [1, 2, 3],
]
array5 = [
  1,
  {
    # inline begin dangling comment1
    # inline begin dangling comment2

    # key1 leading comment1
    # key1 leading comment2
    key1 = 1,  # key1 trailing comment
    # key2 leading comment1
    key2 = 2,  # key2 trailing comment

    # inline end dangling comment1
    # inline end dangling comment2
  },

  # comment
]
date = 2024-01-01  # date trailing comment
time = 10:11:00  # time trailing comment
odt1 = 1979-05-27T07:32:00Z  # odt1 trailing comment
odt2 = 1979-05-27T00:32:00-07:00  # odt2 trailing comment
odt3 = 1979-05-27T00:32:00.999999-07:00  # odt3 trailing comment
odt4 = 1979-05-27T07:32:00Z  # odt4 trailing comment
ldt1 = 1979-05-27T07:32:00  # ldt1 trailing comment
ldt2 = 1979-05-27T00:32:00.999999  # ldt2 trailing comment
ld1 = 1979-05-27  # ld1 trailing comment
lt1 = 07:32:00  # lt1 trailing comment
# lt2 leading trailing comment
lt2 = 00:32:00.999999  # lt2 trailing comment

# table key values end dangling comment1
# table key values end dangling comment2

# table key values end dangling comment3
# table key values end dangling comment4

# table leading comment1
# table leading comment2
[aaaa.ccc]
key1 = 11
key2 = "22"

[bbb]
key5 = true

[ddd.eee]  # header trailing comment
key5 = true

[[ffff.ggg]]
key6 = 1

[[ffff.ggg]]
key6 = 2

[[ffff.ggg]]  # header trailing comment
# table key values begin dangling comment1
# table key values begin dangling comment2

# table key values begin dangling comment3
# table key values begin dangling comment4

# key value leading comment1
# key value leading comment2
key6 = 3  # key value trailing comment

[ffff.ggg.kkk]
b = 3

# table key values end dangling comment1
# table key values end dangling comment2

# table key values end dangling comment3
# table key values end dangling comment4
"#,
        TomlVersion(TomlVersion::V1_1_0_Preview),
        FormatOptions(FormatOptions{
            rules: Some(FormatRules {
                quote_style: Some(QuoteStyle::Preserve),
                ..Default::default()
            }),
            ..Default::default()
        })) -> Ok(source)
    }
}
