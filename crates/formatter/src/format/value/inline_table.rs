use crate::format::comment::{BeginDanglingComment, EndDanglingComment};
use crate::{
    format::comment::{LeadingComment, TailingComment},
    Format,
};
use ast::AstNode;
use itertools::Itertools;
use std::fmt::Write;

impl Format for ast::InlineTable {
    fn fmt(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if self.should_be_multiline(f.toml_version()) || exceeds_line_width(self, f)? {
            format_multiline_inline_table(self, f)
        } else {
            format_singleline_inline_table(self, f)
        }
    }
}

pub(crate) fn exceeds_line_width(
    node: &ast::InlineTable,
    f: &mut crate::Formatter,
) -> Result<bool, std::fmt::Error> {
    let mut length = f.current_line_width();
    length += 2; // '{' and '}'
    length += f.defs().singleline_inline_table_brace_inner_space().len() * 2;
    let mut first = true;

    for key_value in node.key_values() {
        // Check if nested value should be multiline
        if let Some(value) = key_value.value() {
            let should_be_multiline = match value {
                ast::Value::Array(array) => {
                    array.should_be_multiline(f.toml_version())
                        || crate::format::value::array::exceeds_line_width(&array, f)?
                }
                ast::Value::InlineTable(table) => {
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
            length += f.defs().singleline_inline_table_space_after_comma().len();
        }
        length += f.format_to_string(&key_value)?.len();
        first = false;
    }

    Ok(length > f.line_width() as usize)
}

fn format_multiline_inline_table(
    table: &ast::InlineTable,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    for comment in table.leading_comments() {
        LeadingComment(comment).fmt(f)?;
    }

    f.write_indent()?;
    write!(f, "{{{}", f.line_ending())?;

    f.inc_indent();

    for comments in table.inner_begin_dangling_comments() {
        comments
            .into_iter()
            .map(BeginDanglingComment)
            .collect_vec()
            .fmt(f)?;
    }

    for (i, (key_value, comma)) in table.key_values_with_comma().enumerate() {
        // value format
        {
            if i > 0 {
                write!(f, "{}", f.line_ending())?;
            }
            key_value.fmt(f)?;
        }

        // comma format
        {
            let (comma_leading_comments, comma_tailing_comment) = match comma {
                Some(comma) => (
                    comma.leading_comments().collect_vec(),
                    comma.tailing_comment(),
                ),
                None => (vec![], None),
            };

            if !comma_leading_comments.is_empty() {
                write!(f, "{}", f.line_ending())?;
                for comment in comma_leading_comments {
                    LeadingComment(comment).fmt(f)?;
                }
                f.write_indent()?;
                write!(f, ",")?;
            } else if key_value.tailing_comment().is_some() {
                write!(f, "{}", f.line_ending())?;
                f.write_indent()?;
                write!(f, ",")?;
            } else {
                write!(f, ",")?;
            }

            if let Some(comment) = comma_tailing_comment {
                TailingComment(comment).fmt(f)?;
            }
        }
    }

    table
        .inner_end_dangling_comments()
        .map(EndDanglingComment)
        .collect_vec()
        .fmt(f)?;

    f.dec_indent();

    write!(f, "{}", f.line_ending())?;
    f.write_indent()?;
    write!(f, "}}")?;

    if let Some(comment) = table.tailing_comment() {
        TailingComment(comment).fmt(f)?;
    }

    Ok(())
}

fn format_singleline_inline_table(
    table: &ast::InlineTable,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    for comment in table.leading_comments() {
        LeadingComment(comment).fmt(f)?;
    }

    f.write_indent()?;
    write!(
        f,
        "{{{}",
        f.defs().singleline_inline_table_brace_inner_space()
    )?;

    for (i, key_value) in table.key_values().enumerate() {
        if i > 0 {
            write!(
                f,
                ",{}",
                f.defs().singleline_inline_table_space_after_comma()
            )?;
        }
        f.skip_indent();
        key_value.fmt(f)?;
    }

    write!(
        f,
        "{}}}",
        f.defs().singleline_inline_table_brace_inner_space()
    )?;

    if let Some(comment) = table.tailing_comment() {
        TailingComment(comment).fmt(f)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test_format;
    use config::{FormatOptions, TomlVersion};

    test_format! {
        #[test]
        fn inline_table_key_value1(r#"name = { first = "Tom", last = "Preston-Werner" }"#) -> Ok(source);
    }

    test_format! {
        #[test]
        fn inline_table_key_value2(r#"point = { x = 1, y = 2 }"#) -> Ok(source);

    }

    test_format! {
        #[test]
        fn inline_table_key_value3(r#"animal = { type.name = "pug" }"#) -> Ok(source);
    }

    test_format! {
        #[test]
        fn inline_table_exceeds_line_width(
            r#"table = { key1 = 1111111111, key2 = 2222222222, key3 = 3333333333 }"#,
            TomlVersion::default(),
            FormatOptions {
                line_width: Some(30.try_into().unwrap()),
                ..Default::default()
            }
        ) -> Ok(
            r#"
            table = {
              key1 = 1111111111,
              key2 = 2222222222,
              key3 = 3333333333,
            }
            "#
        );
    }

    test_format! {
        #[test]
        fn inline_table_with_nested_array_exceeds_line_width(
            r#"table = { key1 = [1111111111, 2222222222], key2 = [3333333333, 4444444444] }"#,
            TomlVersion::default(),
            FormatOptions {
                line_width: Some(35.try_into().unwrap()),
                ..Default::default()
            }
        ) -> Ok(
            r#"
            table = {
              key1 = [1111111111, 2222222222],
              key2 = [3333333333, 4444444444],
            }
            "#
        );
    }

    test_format! {
        #[test]
        fn inline_table_with_nested_inline_table_exceeds_line_width(
            r#"table = { t1 = { key1 = 1111111111, key2 = 2222222222 }, t2 = { key3 = 3333333333, key4 = 4444444444 } }"#,
            TomlVersion::default(),

            FormatOptions {
                line_width: Some(30.try_into().unwrap()),
                ..Default::default()
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
                key4 = 4444444444,
              },
            }
            "#
        );
    }
}
