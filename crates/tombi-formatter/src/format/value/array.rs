use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;
use unicode_segmentation::UnicodeSegmentation;

use crate::{format::write_trailing_comment_alignment_space, types::WithAlignmentHint, Format};

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

    for value in node.values() {
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

        // Calculate total length
        if !first {
            length += 1; // ","
            length += f.array_comma_space().len();
        }
        length += f.format_to_string(&value)?.graphemes(true).count();
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
    write!(f, "[{}", f.line_ending())?;

    f.inc_indent();

    let values_with_comma = array.values_with_comma().collect_vec();

    if values_with_comma.is_empty() {
        array.inner_dangling_comments().format(f)?;
    } else {
        array.inner_begin_dangling_comments().format(f)?;

        let has_last_value_trailing_comma = array.has_last_value_trailing_comma();
        let values_len = values_with_comma.len();

        for (i, (value, comma)) in values_with_comma.into_iter().enumerate() {
            // value format
            {
                if i > 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                WithAlignmentHint::new_with_trailing_comment_alignment_width(
                    &value,
                    *trailing_comment_alignment_width,
                )
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
                } else if value.trailing_comment().is_some() {
                    write!(f, "{}", f.line_ending())?;
                    f.write_indent()?;
                    write!(f, ",")?;
                } else if has_last_value_trailing_comma || i + 1 != values_len {
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

        array.inner_end_dangling_comments().format(f)?;
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
    write!(f, "[{}", f.array_bracket_space())?;

    for (i, value) in array.values().enumerate() {
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

    if let Some(comment) = array.trailing_comment() {
        if let Some(trailing_comment_alignment_width) = trailing_comment_alignment_width {
            write_trailing_comment_alignment_space(f, *trailing_comment_alignment_width)?;
        }
        comment.format(f)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tombi_config::{format::FormatRules, FormatOptions, StringQuoteStyle, TomlVersion};

    use super::*;
    use crate::{test_format, Formatter};

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
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        string_quote_style: Some(StringQuoteStyle::Preserve),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
        ) -> Ok(r#"string_array = ["all", 'strings', """are the same""", '''type''']"#)
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
            FormatOptions(
                FormatOptions {
                    rules: Some(FormatRules {
                        line_width: Some(20.try_into().unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            )
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
            FormatOptions(
                FormatOptions {
                rules: Some(FormatRules {
                    line_width: Some(30.try_into().unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            })
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
        let p = tombi_parser::parse_as::<tombi_ast::Array>(source, TomlVersion::default());
        pretty_assertions::assert_eq!(p.errors, Vec::<tombi_parser::Error>::new());

        let ast = tombi_ast::Array::cast(p.syntax_node()).unwrap();
        pretty_assertions::assert_eq!(ast.has_last_value_trailing_comma(), expected);
    }
}
