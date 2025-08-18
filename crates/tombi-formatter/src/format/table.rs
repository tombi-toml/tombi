use std::fmt::Write;

use itertools::Itertools;

use crate::Format;

impl Format for tombi_ast::Table {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let header = self.header().unwrap();

        self.header_leading_comments().collect_vec().format(f)?;

        write!(f, "[{header}]")?;

        if let Some(comment) = self.header_trailing_comment() {
            comment.format(f)?;
        }

        let key_values = self.key_values().collect_vec();

        if key_values.is_empty() {
            let dangling_comments = self.key_values_dangling_comments();

            if !dangling_comments.is_empty() {
                write!(f, "{}", f.line_ending())?;
                dangling_comments.format(f)?;
            }

            return Ok(());
        } else {
            write!(f, "{}", f.line_ending())?;

            self.key_values_begin_dangling_comments().format(f)?;

            for (i, key_value) in key_values.into_iter().enumerate() {
                if i != 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                key_value.format(f)?;
            }

            self.key_values_end_dangling_comments().format(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_format;

    test_format! {
        #[test]
        fn table_only_header(
            r#"[package]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_only_header_with_basic_string_key(
            r#"[dependencies."unicase"]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_only_header_nexted_keys(
            r#"[dependencies.unicase]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table(
            r#"
            [package]
            name = "toml-rs"
            version = "0.4.0"
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_with_full_comment(
            r#"
            # header leading comment1
            # header leading comment2
            [header]  # header trailing comment
            # table begin dangling comment group 1-1
            # table begin dangling comment group 1-2

            # table begin dangling comment group 2-1
            # table begin dangling comment group 2-2
            # table begin dangling comment group 2-3

            # table begin dangling comment group 3-1

            # key value leading comment1
            # key value leading comment2
            key = "value"  # key trailing comment
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_begin_dangling_comment1(
            r#"
            [header]
            # key values begin dangling comment group 1-1
            # key values begin dangling comment group 1-2

            # key values begin dangling comment group 2-1
            # key values begin dangling comment group 2-2
            # key values begin dangling comment group 2-3

            # key values begin dangling comment group 3-1

            # key values leading comment1
            # key values leading comment2
            key = "value"  # key trailing comment
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_end_dangling_comment1(
            r#"
            [header]
            key = "value"  # key trailing comment

            # key values end dangling comment 1-1
            # key values end dangling comment 1-2
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_end_dangling_comment2(
            r#"
            [header]
            key = "value"  # key trailing comment

            # key values end dangling comment 1-1
            # key values end dangling comment 1-2

            # key values end dangling comment 2-1
            # key values end dangling comment 2-2
            # key values end dangling comment 2-3

            # key values end dangling comment 3-1
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn table_end_dangling_comment3(
            r#"
            [header]
            key = "value"  # key trailing comment

            # key values end dangling comment1
            # key values end dangling comment2
            "#
        ) -> Ok(source);
    }
}
