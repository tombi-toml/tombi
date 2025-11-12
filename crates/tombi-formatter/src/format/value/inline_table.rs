use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use unicode_segmentation::UnicodeSegmentation;

use crate::{format::write_trailing_comment_alignment_space, types::WithAlignmentHint, Format};

impl Format for tombi_ast::InlineTable {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<'_, tombi_ast::InlineTable> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if !f.single_line_mode()
            && (self.value.should_be_multiline(f.toml_version())
                || exceeds_line_width(self.value, f)?)
        {
            format_multiline_inline_table(self, f)
        } else {
            format_singleline_inline_table(self, f)
        }
    }
}

pub(crate) fn exceeds_line_width(
    node: &tombi_ast::InlineTable,
    f: &mut crate::Formatter,
) -> Result<bool, std::fmt::Error> {
    if f.toml_version() < TomlVersion::V1_1_0_Preview {
        return Ok(false);
    }

    let mut length = f.current_line_width();
    length += 2; // '{' and '}'
    length += f.inline_table_brace_space().len() * 2;
    let mut first = true;

    for key_value in node.key_values() {
        // Check if nested value should be multiline
        if let Some(value) = key_value.value() {
            let should_be_multiline = match value {
                tombi_ast::Value::Array(array) => {
                    array.should_be_multiline(f.toml_version())
                        || crate::format::value::array::exceeds_line_width(&array, f)?
                }
                tombi_ast::Value::InlineTable(table) => {
                    table.should_be_multiline(f.toml_version()) || exceeds_line_width(&table, f)?
                }
                _ => false,
            };

            if should_be_multiline {
                return Ok(true);
            }
        }

        if !first {
            length += 1; // ","
            length += f.inline_table_comma_space().len();
        }
        length += f.format_to_string(&key_value)?.graphemes(true).count();
        first = false;
    }

    if let Some(trailing_comment) = node.trailing_comment() {
        length += f.trailing_comment_space().len();
        length += f
            .format_to_string(&trailing_comment)?
            .graphemes(true)
            .count();
    }

    Ok(length > f.line_width() as usize)
}

fn format_multiline_inline_table(
    WithAlignmentHint {
        value: table,
        trailing_comment_alignment_width,
        ..
    }: &WithAlignmentHint<'_, tombi_ast::InlineTable>,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    table.leading_comments().collect_vec().format(f)?;

    f.write_indent()?;
    write!(f, "{{{}", f.line_ending())?;

    f.inc_indent();

    let key_values_with_comma = table.key_values_with_comma().collect_vec();

    if key_values_with_comma.is_empty() {
        table.inner_dangling_comments().format(f)?;
    } else {
        table.inner_begin_dangling_comments().format(f)?;

        let has_last_key_value_trailing_comma = table.has_last_key_value_trailing_comma();
        let key_values_len = key_values_with_comma.len();

        let equal_alignment_width = f.key_value_equal_alignment_width(
            key_values_with_comma.iter().map(|(key_value, _)| key_value),
        );
        for (i, (key_value, comma)) in key_values_with_comma.into_iter().enumerate() {
            // value format
            {
                if i > 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                WithAlignmentHint {
                    value: &key_value,
                    equal_alignment_width,
                    trailing_comment_alignment_width: *trailing_comment_alignment_width,
                }
                .format(f)?;
            }

            // comma format
            {
                let (comma_leading_comments, comma_trailing_comment) = match comma {
                    Some(comma) => (
                        comma.leading_comments().collect_vec(),
                        comma.trailing_comment(),
                    ),
                    None => (vec![], None),
                };

                if !comma_leading_comments.is_empty() {
                    write!(f, "{}", f.line_ending())?;
                    comma_leading_comments.format(f)?;
                    f.write_indent()?;
                    write!(f, ",")?;
                } else if key_value.trailing_comment().is_some() {
                    write!(f, "{}", f.line_ending())?;
                    f.write_indent()?;
                    write!(f, ",")?;
                } else if has_last_key_value_trailing_comma || i + 1 != key_values_len {
                    write!(f, ",")?;
                }

                if let Some(comment) = comma_trailing_comment {
                    if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width
                    {
                        write_trailing_comment_alignment_space(
                            f,
                            *trailing_comment_alignment_width,
                        )?;
                    }
                    comment.format(f)?;
                }
            }
        }

        table.inner_end_dangling_comments().format(f)?;
    }

    f.dec_indent();

    write!(f, "{}", f.line_ending())?;
    f.write_indent()?;
    write!(f, "}}")?;

    if let Some(comment) = table.trailing_comment() {
        if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width {
            write_trailing_comment_alignment_space(f, *trailing_comment_alignment_width)?;
        }
        comment.format(f)?;
    }

    Ok(())
}

fn format_singleline_inline_table(
    WithAlignmentHint {
        value: table,
        trailing_comment_alignment_width,
        ..
    }: &WithAlignmentHint<'_, tombi_ast::InlineTable>,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    table.leading_comments().collect_vec().format(f)?;

    f.write_indent()?;
    write!(f, "{{{}", f.inline_table_brace_space())?;

    for (i, key_value) in table.key_values().enumerate() {
        if i > 0 {
            write!(f, ",{}", f.inline_table_comma_space())?;
        }
        f.skip_indent();
        WithAlignmentHint::new_with_trailing_comment_alignment_width(
            &key_value,
            *trailing_comment_alignment_width,
        )
        .format(f)?;
    }

    write!(f, "{}}}", f.inline_table_brace_space())?;

    if let Some(comment) = table.trailing_comment() {
        if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width {
            write_trailing_comment_alignment_space(f, *trailing_comment_alignment_width)?;
        }
        comment.format(f)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tombi_config::{format::FormatRules, FormatOptions};

    use crate::{test_format, Formatter};

    test_format! {
        #[tokio::test]
        async fn inline_table_key_value1(r#"name = { first = "Tom", last = "Preston-Werner" }"#) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_key_value2(r#"point = { x = 1, y = 2 }"#) -> Ok(source)

    }

    test_format! {
        #[tokio::test]
        async fn inline_table_key_value3(r#"animal = { type.name = "pug" }"#) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_inner_comment_only1(
            r#"
            inline_table = {
              # comment
            }"#,
            TomlVersion(TomlVersion::V1_1_0_Preview)
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_inner_comment_only2(
            r#"
            inline_table = {
              # comment 1-1
              # comment 1-2

              # comment 2-1
              # comment 2-2
              # comment 2-3

              # comment 3-1
            }"#,
            TomlVersion(TomlVersion::V1_1_0_Preview)
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_exceeds_line_width_v1_0_0(
            r#"table = { key1 = 1111111111, key2 = 2222222222, key3 = 3333333333 }"#,
            TomlVersion(TomlVersion::V1_0_0),
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        line_width: Some(30.try_into().unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_exceeds_line_width_v1_1_0(
            r#"table = { key1 = 1111111111, key2 = 2222222222, key3 = 3333333333 }"#,
            TomlVersion(TomlVersion::V1_1_0_Preview),
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        line_width: Some(30.try_into().unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(
            r#"
            table = {
              key1 = 1111111111,
              key2 = 2222222222,
              key3 = 3333333333
            }
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_with_nested_array_exceeds_line_width(
            r#"table = { key1 = [1111111111, 2222222222], key2 = [3333333333, 4444444444] }"#,
            TomlVersion(TomlVersion::V1_1_0_Preview),
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        line_width: Some(35.try_into().unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(
            r#"
            table = {
              key1 = [1111111111, 2222222222],
              key2 = [3333333333, 4444444444]
            }
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_with_nested_inline_table_exceeds_line_width(
            r#"table = { t1 = { key1 = 1111111111, key2 = 2222222222, }, t2 = { key3 = 3333333333, key4 = 4444444444 } }"#,
            TomlVersion(TomlVersion::V1_1_0_Preview),
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        line_width: Some(30.try_into().unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(
            r#"
            table = {
              t1 = {
                key1 = 1111111111,
                key2 = 2222222222,
              },
              t2 = {
                key3 = 3333333333,
                key4 = 4444444444
              }
            }
            "#
        )
    }
}
