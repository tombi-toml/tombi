use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;

use crate::{Format, format::write_trailing_comment_alignment_space, types::WithAlignmentHint};

impl Format for tombi_ast::KeyValue {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<&tombi_ast::KeyValue> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let key_value = self.value;
        key_value.leading_comments().collect_vec().format(f)?;

        f.write_indent()?;

        WithAlignmentHint::new_with_equal_alignment_width(
            &key_value.keys().unwrap(),
            self.equal_alignment_width,
        )
        .format(f)?;

        write!(
            f,
            "{}={}",
            f.key_value_equal_space(),
            f.key_value_equal_space()
        )?;

        f.skip_indent();

        WithAlignmentHint::new_with_trailing_comment_alignment_width(
            &key_value.value().unwrap(),
            self.trailing_comment_alignment_width,
        )
        .format(f)?;

        // NOTE: trailing comment is output by `value.fmt(f)`.

        Ok(())
    }
}

impl Format for tombi_ast::KeyValueGroup {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let key_values_with_comma = self.key_values_with_comma().collect_vec();
        let equal_alignment_width = f.key_value_equal_alignment_width(
            key_values_with_comma.iter().map(|(key_value, _)| key_value),
        );
        let trailing_comment_alignment_width = f.trailing_comment_alignment_width(
            key_values_with_comma.iter().map(|(key_value, _)| key_value),
            equal_alignment_width,
        )?;

        for (i, (key_value, comma)) in key_values_with_comma.iter().enumerate() {
            if i != 0 {
                write!(f, "{}", f.line_ending())?;
            }

            WithAlignmentHint {
                value: key_value,
                equal_alignment_width,
                trailing_comment_alignment_width,
            }
            .format(f)?;

            if let Some(comma) = comma {
                let leading_comments = comma.leading_comments().collect_vec();
                let key_value_has_trailing_comment = key_value.trailing_comment().is_some();
                if let Some(trailing_comment) = comma.trailing_comment() {
                    if leading_comments.is_empty() && !key_value_has_trailing_comment {
                        if let Some(trailing_comment_alignment_width) =
                            trailing_comment_alignment_width
                        {
                            write_trailing_comment_alignment_space(
                                f,
                                trailing_comment_alignment_width,
                            )?;
                        }
                        trailing_comment.format(f)?;
                    } else {
                        write!(f, "{}", f.line_ending())?;
                        if !leading_comments.is_empty() {
                            leading_comments.format(f)?;
                        }
                        f.write_indent()?;
                        write!(f, ",")?;
                        if let Some(trailing_comment_alignment_width) =
                            trailing_comment_alignment_width
                        {
                            write_trailing_comment_alignment_space(
                                f,
                                trailing_comment_alignment_width,
                            )?;
                        }
                        trailing_comment.format(f)?;
                    }
                } else if !leading_comments.is_empty() {
                    write!(f, "{}", f.line_ending())?;
                    leading_comments.format(f)?;
                    f.write_indent()?;
                    write!(f, ",")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn bare_key_value1(r#"key = "value""#) -> Ok("key = \"value\"")
    }
    test_format! {
        #[tokio::test]
        async fn bare_key_value2(r#"key    = "value""#) -> Ok("key = \"value\"")
    }
    test_format! {
        #[tokio::test]
        async fn dotted_keys_value1(r#"key1.key2.key3 = "value""#) -> Ok(source)
    }
    test_format! {
        #[tokio::test]
        async fn dotted_keys_value2(r#"site."google.com" = true"#) -> Ok(source)
    }
    test_format! {
        #[tokio::test]
        async fn key_value_with_comment(
            r#"
            # leading comment1
            # leading comment2
            key = "value"  # trailing comment
            "#
        ) -> Ok(source)
    }
}
