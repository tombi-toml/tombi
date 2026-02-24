use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Format,
    format::{has_empty_line_before, write_trailing_comment_alignment_space},
    types::WithAlignmentHint,
};

impl Format for tombi_ast::Array {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<'_, tombi_ast::Array> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if !f.single_line_mode()
            && (self.value.should_be_multiline(f.toml_version())
                || exceeds_line_width(self.value, f)?)
        {
            format_multiline_array(self, f)
        } else {
            format_singleline_array(self, f)
        }
    }
}

pub(crate) fn exceeds_line_width(
    node: &tombi_ast::Array,
    f: &mut crate::Formatter,
) -> Result<bool, std::fmt::Error> {
    let mut length = f.current_line_width();
    length += 2; // '[' and ']'
    length += f.array_bracket_space().len() * 2; // Space after '[' and before ']'
    let mut first = true;

    for group in node.value_with_comma_groups() {
        let tombi_ast::DanglingCommentGroupOr::ItemGroup(item_group) = group else {
            continue;
        };

        for value in item_group.values() {
            // Check if nested value should be multiline
            let should_be_multiline = match &value {
                tombi_ast::Value::Array(array) => {
                    array.should_be_multiline(f.toml_version()) || exceeds_line_width(array, f)?
                }
                tombi_ast::Value::InlineTable(table) => {
                    table.should_be_multiline(f.toml_version())
                        || crate::format::value::inline_table::exceeds_line_width(table, f)?
                }
                _ => false,
            };

            if should_be_multiline {
                return Ok(true);
            }

            if !first {
                length += 1; // ","
                length += f.array_comma_space().len();
            }
            length += f.format_to_string(&value)?.graphemes(true).count();
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

fn format_multiline_array(
    WithAlignmentHint {
        value: array,
        trailing_comment_alignment_width,
        ..
    }: &WithAlignmentHint<'_, tombi_ast::Array>,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    array.leading_comments().collect_vec().format(f)?;

    f.write_indent()?;
    write!(f, "[")?;

    if let Some(trailing_comment) = array.bracket_start_trailing_comment() {
        trailing_comment.format(f)?;
    }

    write!(f, "{}", f.line_ending())?;

    f.inc_indent();

    let dangling_comment_groups = array.dangling_comment_groups().collect_vec();
    dangling_comment_groups.format(f)?;

    let groups = array.value_with_comma_groups().collect_vec();
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
                    WithAlignmentHint::new_with_trailing_comment_alignment_width(
                        item_group,
                        *trailing_comment_alignment_width,
                    )
                    .format(f)?;
                }
            }
        }
    }

    f.dec_indent();

    write!(f, "{}", f.line_ending())?;
    f.write_indent()?;
    write!(f, "]")?;

    if let Some(comment) = array.trailing_comment() {
        if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width {
            write_trailing_comment_alignment_space(f, *trailing_comment_alignment_width)?;
        }
        comment.format(f)?;
    }

    Ok(())
}

fn format_singleline_array(
    WithAlignmentHint {
        value: array,
        trailing_comment_alignment_width,
        ..
    }: &WithAlignmentHint<'_, tombi_ast::Array>,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    array.leading_comments().collect_vec().format(f)?;

    f.write_indent()?;

    let mut values = array.values().peekable();

    if values.peek().is_none() {
        write!(f, "[]")?;
    } else {
        write!(f, "[{}", f.array_bracket_space())?;

        for (i, value) in values.enumerate() {
            if i > 0 {
                write!(f, ",{}", f.array_comma_space())?;
            }
            f.skip_indent();
            WithAlignmentHint::new_with_trailing_comment_alignment_width(
                &value,
                *trailing_comment_alignment_width,
            )
            .format(f)?;
        }

        write!(f, "{}]", f.array_bracket_space())?;
    }

    if let Some(comment) = array.trailing_comment() {
        if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width {
            write_trailing_comment_alignment_space(f, *trailing_comment_alignment_width)?;
        }
        comment.format(f)?;
    }

    Ok(())
}

impl Format for WithAlignmentHint<'_, tombi_ast::ValueWithCommaGroup> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let WithAlignmentHint {
            value: value_group,
            trailing_comment_alignment_width,
            ..
        } = self;

        let has_last_value_trailing_comma = value_group
            .values_with_comma()
            .last()
            .is_some_and(|(_, comma)| comma.is_some());

        let mut values = value_group.values_with_comma().enumerate().peekable();
        while let Some((i, (value, comma))) = values.next() {
            if i > 0 {
                write!(f, "{}", f.line_ending())?;
            }

            WithAlignmentHint::new_with_trailing_comment_alignment_width(
                &value,
                *trailing_comment_alignment_width,
            )
            .format(f)?;

            if let Some(comma) = &comma {
                let leading_comments = comma.leading_comments().collect_vec();
                if !leading_comments.is_empty() {
                    write!(f, "{}", f.line_ending())?;
                    leading_comments.format(f)?;
                    f.write_indent()?;
                } else if value.trailing_comment().is_some() {
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
            } else if has_last_value_trailing_comma || values.peek().is_some() {
                write!(f, ",")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tombi_config::{StringQuoteStyle, format::FormatRules};

    use super::*;
    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn singleline_array1(
            "array=[1,2,3]"
        ) -> Ok("array = [1, 2, 3]")
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array2(
            "array=[ 1 ]"
        ) -> Ok("array = [1]")
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array3(
            "array=[ 1, 2, 3 ]"
        ) -> Ok("array = [1, 2, 3]")
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array4(
            r#"colors = [ "red", "yellow", "green" ]"#
        ) -> Ok(r#"colors = ["red", "yellow", "green"]"#)
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array5(
            "nested_arrays_of_ints = [ [ 1, 2 ], [ 3, 4, 5 ] ]"
        ) -> Ok("nested_arrays_of_ints = [[1, 2], [3, 4, 5]]")
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array6(
            r#"nested_mixed_array = [ [ 1, 2 ], [ "a", "b", "c" ] ]"#
        ) -> Ok(r#"nested_mixed_array = [[1, 2], ["a", "b", "c"]]"#)
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array7(
            r#"string_array = [ "all", 'strings', """are the same""", '''type''' ]"#,
            FormatOptions{
                rules: Some(FormatRules {
                    string_quote_style: Some(StringQuoteStyle::Preserve),
                    ..Default::default()
                }),
            }
        ) -> Ok(r#"string_array = ["all", 'strings', """are the same""", '''type''']"#)
    }

    test_format! {
        #[tokio::test]
        async fn singleline_array_missing_comma(
            "array = [1 2, 3]"
        ) -> Ok("array = [1, 2, 3]")
    }

    test_format! {
        #[tokio::test]
        async fn multiline_array1(
            "array = [1, 2, 3,]"
        ) -> Ok(
            r#"
            array = [
              1,
              2,
              3,
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn multiline_array2(
            "array = [1, ]"
        ) -> Ok(
            r#"
            array = [
              1,
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn multiline_array3(
            r#"
            array = [
              1  # comment
            ]
            "#
        ) -> Ok(
            r#"
            array = [
              1,  # comment
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn multiline_array4(
            r#"
            array = [
              1,  # comment
            ]
            "#
        ) -> Ok(
            r#"
            array = [
              1,  # comment
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn multiline_array5(
            r#"
            array = [
              1  # comment
              ,
            ]
            "#
        ) -> Ok(
            r#"
            array = [
              1,  # comment
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn multiline_array_with_full_comment(
            r#"
            # array leading comment1
            # array leading comment2
            array = [

              # inner array begin dangling comment group 1-1
              # inner array begin dangling comment group 1-2


              # inner array begin dangling comment group 2-1

              # value1 leading comment1
              # value1 leading comment2
              1 # value1 trailing comment
              , # value1 comma trailing comment
              2 # value2 trailing comment
              # value2 comma leading comment1
              # value2 comma leading comment2
              , # value2 comma trailing comment
              # value3 leading comment1
              # value3 leading comment2
              3 # value3 trailing comment
              # array end dangling comment group 1-1
              # array end dangling comment group 1-2

              # array end dangling comment group 2-1

            ] # array trailing comment
            "#
        ) -> Ok(
            r#"
            # array leading comment1
            # array leading comment2
            array = [
              # inner array begin dangling comment group 1-1
              # inner array begin dangling comment group 1-2

              # inner array begin dangling comment group 2-1

              # value1 leading comment1
              # value1 leading comment2
              1  # value1 trailing comment
              ,  # value1 comma trailing comment
              2  # value2 trailing comment
              # value2 comma leading comment1
              # value2 comma leading comment2
              ,  # value2 comma trailing comment
              # value3 leading comment1
              # value3 leading comment2
              3,  # value3 trailing comment
              # array end dangling comment group 1-1
              # array end dangling comment group 1-2

              # array end dangling comment group 2-1
            ]  # array trailing comment
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn nested_multiline_array(
            "array = [ [1,2,3,], [4,5,6], [7,8,9,] ]"
        ) -> Ok(
            r#"
            array = [
              [
                1,
                2,
                3,
              ],
              [4, 5, 6],
              [
                7,
                8,
                9,
              ]
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn nested_multiline_array_with_trailing_comma(
            "array = [ [1,2,3,], [4,5,6], [7,8,9,], ]"
        ) -> Ok(
            r#"
            array = [
              [
                1,
                2,
                3,
              ],
              [4, 5, 6],
              [
                7,
                8,
                9,
              ],
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_only_inner_comment_only1(
            r#"
            array = [
              # comment
            ]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn array_only_inner_comment_only2(
            r#"
            array = [
              # comment 1-1
              # comment 1-2

              # comment 2-1
              # comment 2-2
              # comment 2-3

              # comment 3-1
            ]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn array_exceeds_line_width(
            r#"array = [1111111111, 2222222222, 3333333333]"#,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(20.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            array = [
              1111111111,
              2222222222,
              3333333333
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_with_nested_array_exceeds_line_width(
            r#"array = [[1111111111, 2222222222], [3333333333, 4444444444]]"#,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(30.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            array = [
              [1111111111, 2222222222],
              [3333333333, 4444444444]
            ]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_with_nested_inline_table_exceeds_line_width(
            r#"array = [{ key1 = 1111111111, key2 = 2222222222 }, { key3 = [3333333333, 4444444444], key4 = [5555555555, 6666666666, 7777777777] }]"#,
            TomlVersion::V1_1_0,
            FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(35.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            array = [
              {
                key1 = 1111111111,
                key2 = 2222222222
              },
              {
                key3 = [3333333333, 4444444444],
                key4 = [
                  5555555555,
                  6666666666,
                  7777777777
                ]
              }
            ]
            "#
        )
    }

    #[rstest]
    #[case("[1, 2, 3,]", true)]
    #[case("[1, 2, 3]", false)]
    fn has_last_value_trailing_comma(#[case] source: &str, #[case] expected: bool) {
        let p = tombi_parser::parse_as::<tombi_ast::Array>(source);
        pretty_assertions::assert_eq!(p.errors, Vec::<tombi_parser::Error>::new());

        let ast = tombi_ast::Array::cast(p.syntax_node()).unwrap();
        pretty_assertions::assert_eq!(ast.has_last_value_trailing_comma(), expected);
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_no_space(
            r#"empty = [ ]"#
        ) -> Ok(r#"empty = []"#)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_with_elements(
            r#"filled = [1, 2, 3]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn nested_empty_arrays(
            r#"nested = [[ ], [  ], [1, 2]]"#
        ) -> Ok(r#"nested = [[], [], [1, 2]]"#)
    }

    test_format! {
        #[tokio::test]
        async fn nested_empty_arrays_with_bracket_space_width_one(
            r#"nested = [[ ], [  ], [1, 2]]"#,
            FormatOptions {
                rules: Some(FormatRules {
                    array_bracket_space_width: Some(1.into()),
                    ..Default::default()
                }),
                ..Default::default()
            }
        ) -> Ok(r#"nested = [ [], [], [ 1, 2 ] ]"#)
    }
}
