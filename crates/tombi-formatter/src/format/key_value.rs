use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::AstNode;

use crate::{types::WithAlignmentHint, Format};

impl Format for tombi_ast::KeyValue {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        WithAlignmentHint::new(self).format(f)
    }
}

impl Format for WithAlignmentHint<'_, tombi_ast::KeyValue> {
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

#[cfg(test)]
mod tests {
    use crate::{test_format, Formatter};

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
