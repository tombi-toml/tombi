mod format;
pub mod formatter;

pub use tombi_config::FormatOptions;
use format::Format;
use formatter::definitions::FormatDefinitions;
pub use formatter::Formatter;

#[cfg(test)]
#[macro_export]
macro_rules! test_format {
    (#[test] fn $name:ident($source:expr) -> Ok(source);) => {
        $crate::test_format!(#[test] fn $name($source) -> Ok($source););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr) -> Ok(source);) => {
        $crate::test_format!(#[test] fn $name($source, $toml_version) -> Ok($source););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr, $definition:expr) -> Ok(source);) => {
        $crate::test_format!(#[test] fn $name($source, $toml_version, $definition) -> Ok($source););
    };

    (#[test] fn $name:ident($source:expr) -> Ok($expected:expr);) => {
        $crate::test_format!(#[test] fn $name($source, tombi_config::TomlVersion::default()) -> Ok($expected););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr) -> Ok($expected:expr);) => {
        $crate::test_format!(#[test] fn $name($source, $toml_version, $crate::FormatDefinitions::default()) -> Ok($expected););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr, $definitions:expr) -> Ok($expected:expr);) => {
        $crate::test_format!(#[test] fn $name($source, $toml_version, $definitions, &$crate::FormatOptions::default()) -> Ok($expected););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr, $definitions:expr, $options:expr) -> Ok($expected:expr);) => {
        #[tokio::test]
        async fn $name() {
            tombi_test_lib::init_tracing();

            match $crate::Formatter::new(
                $toml_version,
                $definitions,
                $options,
                None,
                &tombi_schema_store::SchemaStore::new()
            ).format($source).await {
                Ok(formatted_text) => {
                    pretty_assertions::assert_eq!(formatted_text, textwrap::dedent($expected).trim().to_string() + "\n");
                }
                Err(errors) => {
                    pretty_assertions::assert_eq!(errors, vec![]);
                }
            }
        }
    };

    (#[test] fn $name:ident($source:expr) -> Err(_);) => {
        $crate::test_format!(#[test] fn $name($source, tombi_config::TomlVersion::default()) -> Err(_););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr) -> Err(_);) => {
        $crate::test_format!(#[test] fn $name($source, $toml_version, $crate::FormatDefinitions::default()) -> Err(_););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr, $definitions:expr) -> Err(_);) => {
        $crate::test_format!(#[test] fn $name($source, $toml_version, $definitions, &$crate::FormatOptions::default()) -> Err(_););
    };

    (#[test] fn $name:ident($source:expr, $toml_version:expr, $definitions:expr, $options:expr) -> Err(_);) => {
        #[tokio::test]
        async fn $name() {
            tombi_test_lib::init_tracing();

            match $crate::Formatter::new(
                $toml_version,
                $definitions,
                $options,
                None,
                &tombi_schema_store::SchemaStore::new()
            ).format($source).await {
                Ok(_) => panic!("expected an error"),
                Err(errors) => {
                    pretty_assertions::assert_ne!(errors, vec![]);
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use tombi_config::{QuoteStyle, TomlVersion};

    use super::*;
    use crate::FormatDefinitions;

    test_format! {
        #[test]
        fn test_only_comment1(
            r#"
            # comment1
            # comment2
            "#,
            TomlVersion::V1_0_0
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn test_only_comment2(
            r#"
            # comment1
            # comment2

            # comment3
            # comment4
            "#,
            TomlVersion::V1_0_0
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn test_key_values(r#"
            array5 = [
              1,
              {
                # inline begin dangling comment1
                # inline begin dangling comment2

                # key1 leading comment1
                # key1 leading comment2
                key1 = 1,  # key1 tailing comment
                # key2 leading comment1
                key2 = 2,  # key2 tailing comment

                # inline end dangling comment1
                # inline end dangling comment2
              },

              # comment
            ]
            "#,
            TomlVersion::V1_1_0_Preview
        ) -> Ok(source);
    }

    test_format! {
    #[test]
    fn test_sample_toml(
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
dec = 1  # dec tailing comment
bin = 0b01  # bin tailing comment
oct = 0o01  # oct tailing comment
hex = 0x01  # hex tailing comment
float1 = 0.1234  # float tailing comment
infa = inf
bs = "2"  # bs tailing comment
ls = '3'  # ls tailing comment
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
  ,  # value1 comma tailing comment
  { key3 = 11, key4 = 2024-01-01T10:10:00 },

  # array end dangling comment1
  # array end dangling comment2
]  # array tailing comment
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
    key1 = 1,  # key1 tailing comment
    # key2 leading comment1
    key2 = 2,  # key2 tailing comment

    # inline end dangling comment1
    # inline end dangling comment2
  },

  # comment
]
date = 2024-01-01  # date tailing comment
time = 10:11:00  # time tailing comment
odt1 = 1979-05-27T07:32:00Z  # odt1 tailing comment
odt2 = 1979-05-27T00:32:00-07:00  # odt2 tailing comment
odt3 = 1979-05-27T00:32:00.999999-07:00  # odt3 tailing comment
odt4 = 1979-05-27T07:32:00Z  # odt4 tailing comment
ldt1 = 1979-05-27T07:32:00  # ldt1 tailing comment
ldt2 = 1979-05-27T00:32:00.999999  # ldt2 tailing comment
ld1 = 1979-05-27  # ld1 tailing comment
lt1 = 07:32:00  # lt1 tailing comment
# lt2 leading tailing comment
lt2 = 00:32:00.999999  # lt2 tailing comment

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

[ddd.eee]  # header tailing comment
key5 = true

[[ffff.ggg]]
key6 = 1

[[ffff.ggg]]
key6 = 2

[[ffff.ggg]]  # header tailing comment
# table key values begin dangling comment1
# table key values begin dangling comment2

# table key values begin dangling comment3
# table key values begin dangling comment4

# key value leading comment1
# key value leading comment2
key6 = 3  # key value tailing comment

[ffff.ggg.kkk]
b = 3

# table key values end dangling comment1
# table key values end dangling comment2

# table key values end dangling comment3
# table key values end dangling comment4
"#,
    TomlVersion::V1_1_0_Preview,
    FormatDefinitions{
        quote_style: Some(QuoteStyle::Preserve),
        ..Default::default()
    }) -> Ok(source);
    }
}
