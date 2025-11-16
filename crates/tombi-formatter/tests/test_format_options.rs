mod format_options {
    use tombi_config::{
        format::FormatRules, DateTimeDelimiter, FormatOptions, IndentStyle, LineEnding,
        StringQuoteStyle,
    };
    use tombi_formatter::{test_format, Formatter};

    mod array_bracket_space_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_array_bracket_space_width_zero(
                r#"
                key = [1, 2, 3]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        array_bracket_space_width: Some(0.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [1, 2, 3]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_bracket_space_width_one(
                r#"
                key = [1, 2, 3]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        array_bracket_space_width: Some(1.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [ 1, 2, 3 ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_bracket_space_width_two(
                r#"
                key = [1, 2, 3]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        array_bracket_space_width: Some(2.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [  1, 2, 3  ]
                "#
            )
        }
    }

    mod array_comma_space_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_array_comma_space_width_zero(
                r#"
                key = [1, 2,  3]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        array_comma_space_width: Some(0.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [1,2,3]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_comma_space_width_one(
                r#"
                key = [1,2,3]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        array_comma_space_width: Some(1.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [1, 2, 3]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_array_comma_space_width_two(
                r#"
                key = [1,2,3]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        array_comma_space_width: Some(2.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [1,  2,  3]
                "#
            )
        }
    }

    mod date_time_delimiter {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_date_time_delimiter_t(
                r#"
                key = 2024-01-01 10:00:00
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        date_time_delimiter: Some(DateTimeDelimiter::T),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = 2024-01-01T10:00:00
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_date_time_delimiter_space(
                r#"
                key = 2024-01-01T10:00:00
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        date_time_delimiter: Some(DateTimeDelimiter::Space),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = 2024-01-01 10:00:00
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_date_time_delimiter_preserve(
                r#"
                key = 2024-01-01T10:00:00
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        date_time_delimiter: Some(DateTimeDelimiter::Preserve),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = 2024-01-01T10:00:00
                "#
            )
        }
    }

    mod indent_style {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_indent_style_space(
                r#"
                [table]
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        indent_style: Some(IndentStyle::Space),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                key = "value"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_indent_style_tab(
                r#"
                [table]
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        indent_style: Some(IndentStyle::Tab),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                key = "value"
                "#
            )
        }
    }

    mod indent_table_key_values {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_indent_table_key_values_false(
                r#"
                [table]
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        indent_table_key_values: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                key = "value"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_indent_table_key_values_true(
                r#"
                [table]
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        indent_table_key_values: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                  key = "value"
                "#
            )
        }
    }

    mod indent_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_indent_width_two(
                r#"
                key = [
                   1,
                  2,
                    3,
                ]
                "#,
            ) -> Ok(
                r#"
                key = [
                  1,
                  2,
                  3,
                ]
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_indent_width_four(
                r#"
                key = [
                  1,
                  2,
                  3,
                ]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        indent_width: Some(4.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [
                    1,
                    2,
                    3,
                ]
                "#
            )
        }
    }

    mod inline_table_brace_space_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_inline_table_brace_space_width_zero(
                r#"
                key = {a = 1, b = 2}
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        inline_table_brace_space_width: Some(0.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = {a = 1, b = 2}
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_inline_table_brace_space_width_one(
                r#"
                key = {a = 1, b = 2}
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        inline_table_brace_space_width: Some(1.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = { a = 1, b = 2 }
                "#
            )
        }
    }

    mod inline_table_comma_space_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_inline_table_comma_space_width_zero(
                r#"
                key = {a = 1,b = 2}
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        inline_table_comma_space_width: Some(0.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = { a = 1,b = 2 }
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_inline_table_comma_space_width_two(
                r#"
                key = {a = 1,b = 2}
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        inline_table_comma_space_width: Some(2.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = { a = 1,  b = 2 }
                "#
            )
        }
    }

    mod key_value_equal_alignment {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_alignment_false(
                r#"
                key = "value"
                key2 = "value2"
                key3.key4 = "value3"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_alignment: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"
                key2 = "value2"
                key3.key4 = "value3"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_alignment_true(
                r#"
                key = "value"
                key2 = "value2"
                key3.key4 = "value3"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key       = "value"
                key2      = "value2"
                key3.key4 = "value3"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_alignment_true_in_table(
                r#"
                [table]
                key = "value"
                key2 = "value2"
                key3.key4 = "value3"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                key       = "value"
                key2      = "value2"
                key3.key4 = "value3"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_alignment_true_in_array_of_table(
                r#"
                [[table]]
                key = "value"
                key2 = "value2"
                key3.key4 = "value3"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [[table]]
                key       = "value"
                key2      = "value2"
                key3.key4 = "value3"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_alignment_true_in_multi_line_inline_table(
                r#"
                inline-table = {
                  key = "value",
                  key2 = "value2",
                  key3.key4 = "value3",
                }
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                inline-table = {
                  key       = "value",
                  key2      = "value2",
                  key3.key4 = "value3",
                }
                "#
            )
        }
    }

    mod key_value_equal_space_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_space_width_one(
                r#"
                key="value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_space_width: Some(1.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_key_value_equal_space_width_two(
                r#"
                key="value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        key_value_equal_space_width: Some(2.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key  =  "value"
                "#
            )
        }
    }

    mod line_ending {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_line_ending_lf(
                r#"
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        line_ending: Some(LineEnding::Lf),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"
                "#
            )
        }
    }

    mod line_width {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_line_width_short(
                r#"
                key = ["very long string value that should wrap", "another long string"]
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        line_width: Some(40.try_into().unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = [
                  "very long string value that should wrap",
                  "another long string"
                ]
                "#
            )
        }
    }

    mod string_quote_style {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_string_quote_style_double(
                r#"
                key = 'value'
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        string_quote_style: Some(StringQuoteStyle::Double),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_string_quote_style_single(
                r#"
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        string_quote_style: Some(StringQuoteStyle::Single),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = 'value'
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_string_quote_style_preserve(
                r#"
                key = "value"
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        string_quote_style: Some(StringQuoteStyle::Preserve),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"
                "#
            )
        }
    }

    mod trailing_comment_alignment {
        use super::*;

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_false(
                r#"
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = "value3" # comment 3
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"  # comment 1
                key2 = "value2"  # comment 2
                key3.key4 = "value3"  # comment 3
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true(
                r#"
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = "value3" # comment 3
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"         # comment 1
                key2 = "value2"       # comment 2
                key3.key4 = "value3"  # comment 3
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true_in_table(
                r#"
                [table]
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = "value3" # comment 3
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                key = "value"         # comment 1
                key2 = "value2"       # comment 2
                key3.key4 = "value3"  # comment 3
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true_in_array_of_table(
                r#"
                [[table]]
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = "value3" # comment 3
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [[table]]
                key = "value"         # comment 1
                key2 = "value2"       # comment 2
                key3.key4 = "value3"  # comment 3
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true_in_array(
                r#"
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = [
                  1, # comment 3-1
                  2, # comment 3-2
                  3 # comment 3-3
                ] # comment 4
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"    # comment 1
                key2 = "value2"  # comment 2
                key3.key4 = [
                  1,             # comment 3-1
                  2,             # comment 3-2
                  3,             # comment 3-3
                ]                # comment 4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true_in_array_with_trailing_comma(
                r#"
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = [
                  1, # comment 3-1
                  2, # comment 3-2
                  3, # comment 3-3
                ] # comment 4
                "#,
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"    # comment 1
                key2 = "value2"  # comment 2
                key3.key4 = [
                  1,             # comment 3-1
                  2,             # comment 3-2
                  3,             # comment 3-3
                ]                # comment 4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true_in_inline_table(
                r#"
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = {
                    a = 1, # comment 3-1
                    b = 2, # comment 3-2
                    c = 3  # comment 3-3
                } # comment 4
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"    # comment 1
                key2 = "value2"  # comment 2
                key3.key4 = {
                  a = 1,         # comment 3-1
                  b = 2,         # comment 3-2
                  c = 3,         # comment 3-3
                }                # comment 4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_true_in_inline_table_with_trailing_comma(
                r#"
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = {
                    a = 1, # comment 3-1
                    b = 2, # comment 3-2
                    c = 3, # comment 3-3
                } # comment 4
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                key = "value"    # comment 1
                key2 = "value2"  # comment 2
                key3.key4 = {
                  a = 1,         # comment 3-1
                  b = 2,         # comment 3-2
                  c = 3,         # comment 3-3
                }                # comment 4
                "#
            )
        }

        test_format! {
            #[tokio::test]
            async fn test_trailing_comment_alignment_and_indent_table_key_values_true_in_inline_table(
                r#"
                [table]
                key = "value"  # comment 1
                key2 = "value2" # comment 2
                key3.key4 = {
                    a = 1, # comment 3-1
                    b = 2, # comment 3-2
                    c = 3, # comment 3-3
                } # comment 4
                "#,
                TomlVersion(TomlVersion::V1_1_0_Preview),
                FormatOptions(FormatOptions{
                    rules: Some(FormatRules {
                        trailing_comment_alignment: Some(true),
                        indent_table_key_values: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            ) -> Ok(
                r#"
                [table]
                  key = "value"    # comment 1
                  key2 = "value2"  # comment 2
                  key3.key4 = {
                    a = 1,         # comment 3-1
                    b = 2,         # comment 3-2
                    c = 3,         # comment 3-3
                  }                # comment 4
                "#
            )
        }
    }
}
