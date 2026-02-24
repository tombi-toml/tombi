use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Format,
    format::{has_empty_line_before, write_trailing_comment_alignment_space},
    types::WithAlignmentHint,
};

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
    if f.toml_version() == TomlVersion::V1_0_0 {
        return Ok(false);
    }

    let mut length = f.current_line_width();
    length += 2; // '{' and '}'
    length += f.inline_table_brace_space().len() * 2;
    let mut first = true;

    for group in node.key_value_with_comma_groups() {
        let tombi_ast::DanglingCommentGroupOr::ItemGroup(item_group) = group else {
            continue;
        };
        for key_value in item_group.key_values() {
            // Check if nested value should be multiline
            if let Some(value) = key_value.value() {
                let should_be_multiline = match value {
                    tombi_ast::Value::Array(array) => {
                        array.should_be_multiline(f.toml_version())
                            || crate::format::value::array::exceeds_line_width(&array, f)?
                    }
                    tombi_ast::Value::InlineTable(table) => {
                        table.should_be_multiline(f.toml_version())
                            || exceeds_line_width(&table, f)?
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
    write!(f, "{{")?;

    if let Some(trailing_comment) = table.brace_start_trailing_comment() {
        trailing_comment.format(f)?;
    }

    write!(f, "{}", f.line_ending())?;

    f.inc_indent();

    let dangling_comment_groups = table.dangling_comment_groups().collect_vec();
    dangling_comment_groups.format(f)?;

    let key_values = table.key_values().collect_vec();
    let equal_alignment_width = f.key_value_equal_alignment_width(key_values.iter());

    let groups = table.key_value_with_comma_groups().collect_vec();
    if !groups.is_empty() {
        if !dangling_comment_groups.is_empty() {
            write!(f, "{}", f.line_ending())?;
            write!(f, "{}", f.line_ending())?;
        }

        for (i, group) in groups.iter().enumerate() {
            match group {
                tombi_ast::DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    if f.skip_comment() {
                        return Ok(());
                    }
                    if i != 0 {
                        if has_empty_line_before(comment_group) {
                            write!(f, "{}", f.line_ending())?;
                        }
                        write!(f, "{}", f.line_ending())?;
                    }
                    comment_group.format(f)?;
                }
                tombi_ast::DanglingCommentGroupOr::ItemGroup(item_group) => {
                    if i != 0 {
                        write!(f, "{}", f.line_ending())?;
                        write!(f, "{}", f.line_ending())?;
                    }
                    WithAlignmentHint {
                        value: item_group,
                        equal_alignment_width,
                        trailing_comment_alignment_width: *trailing_comment_alignment_width,
                    }
                    .format(f)?;
                }
            }
        }
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

    let mut key_values = table.key_values().peekable();

    if key_values.peek().is_none() {
        write!(f, "{{}}")?;
    } else {
        write!(f, "{{{}", f.inline_table_brace_space())?;

        for (i, key_value) in key_values.enumerate() {
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
    }

    if let Some(comment) = table.trailing_comment() {
        if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width {
            write_trailing_comment_alignment_space(f, *trailing_comment_alignment_width)?;
        }
        comment.format(f)?;
    }

    Ok(())
}

impl Format for WithAlignmentHint<'_, tombi_ast::KeyValueWithCommaGroup> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let WithAlignmentHint {
            value: key_value_group,
            equal_alignment_width,
            trailing_comment_alignment_width,
        } = self;

        let mut key_values = key_value_group
            .key_values_with_comma()
            .enumerate()
            .peekable();
        while let Some((i, (key_value, comma))) = key_values.next() {
            if i > 0 {
                write!(f, "{}", f.line_ending())?;
            }

            WithAlignmentHint {
                value: &key_value,
                equal_alignment_width: *equal_alignment_width,
                trailing_comment_alignment_width: *trailing_comment_alignment_width,
            }
            .format(f)?;

            if let Some(comma) = &comma {
                let leading_comments = comma.leading_comments().collect_vec();
                if !leading_comments.is_empty() {
                    write!(f, "{}", f.line_ending())?;
                    leading_comments.format(f)?;
                    f.write_indent()?;
                } else if key_value.trailing_comment().is_some() {
                    write!(f, "{}", f.line_ending())?;
                    f.write_indent()?;
                }
                write!(f, ",")?;
                if let Some(trailing_comment) = comma.trailing_comment() {
                    if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width
                    {
                        write_trailing_comment_alignment_space(
                            f,
                            *trailing_comment_alignment_width,
                        )?;
                    }
                    trailing_comment.format(f)?;
                }
            } else {
                if key_values.peek().is_some() {
                    write!(f, ",")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tombi_config::format::FormatRules;

    use crate::{Formatter, test_format};

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
        async fn inline_table_missing_comma_single_line(
            r#"table = { a = 1 b = 2 }"#
        ) -> Ok(r#"table = { a = 1, b = 2 }"#)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_inner_comment_only1(
            r#"
            inline_table = {
              # comment
            }"#,
            TomlVersion::V1_1_0
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
            TomlVersion::V1_1_0
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_exceeds_line_width_v1_0_0(
            r#"table = { key1 = 1111111111, key2 = 2222222222, key3 = 3333333333 }"#,
            TomlVersion::V1_0_0,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(30.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn inline_table_exceeds_line_width_v1_1_0(
            r#"table = { key1 = 1111111111, key2 = 2222222222, key3 = 3333333333 }"#,
            TomlVersion::V1_1_0,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(30.try_into().unwrap()),
                    ..Default::default()
                }),
            }
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
            TomlVersion::V1_1_0,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(35.try_into().unwrap()),
                    ..Default::default()
                }),
            }
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
            TomlVersion::V1_1_0,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(30.try_into().unwrap()),
                    ..Default::default()
                }),
            }
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

    test_format! {
        #[tokio::test]
        async fn inline_table_with_nested_inline_table_exceeds_line_width_v1_0_0(
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
            "#,
            TomlVersion::V1_0_0,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(30.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"table = { t1 = { key1 = 1111111111, key2 = 2222222222 }, t2 = { key3 = 3333333333, key4 = 4444444444 } }"#
        )
    }

    test_format! {
        #[tokio::test]
        async fn empty_inline_table_no_space(
            r#"empty = { }"#
        ) -> Ok(r#"empty = {}"#)
    }

    test_format! {
        #[tokio::test]
        async fn empty_inline_table_with_elements(
            r#"filled = { key = "value" }"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn nested_empty_inline_tables(
            r#"nested = { empty1 = { }, empty2 = {  }, filled = { x = 1 } }"#
        ) -> Ok(r#"nested = { empty1 = {}, empty2 = {}, filled = { x = 1 } }"#)
    }
}
