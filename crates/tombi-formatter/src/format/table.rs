use std::fmt::Write;

use itertools::Itertools;

use crate::{Format, format::filter_map_unique_keys};

impl Format for tombi_ast::Table {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let header = self.header().unwrap();
        let toml_version = f.toml_version();

        if f.indent_sub_tables() {
            filter_map_unique_keys(
                header.keys(),
                self.parent_table_or_array_of_table_keys(toml_version),
                toml_version,
            )
            .for_each(|_| f.inc_indent());
        }

        self.header_leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        write!(f, "[{header}]")?;

        if let Some(comment) = self.header_trailing_comment() {
            comment.format(f)?;
        }

        if f.indent_table_key_values() {
            f.inc_indent();
        }

        let dangling_comment_groups = self.dangling_comment_groups().collect_vec();
        if !dangling_comment_groups.is_empty() {
            write!(f, "{}", f.line_ending())?;
            dangling_comment_groups.format(f)?;
        }

        let key_value_groups = self.key_value_groups().collect_vec();
        if !key_value_groups.is_empty() {
            if !dangling_comment_groups.is_empty() {
                write!(f, "{}", f.line_ending())?;
            }
            write!(f, "{}", f.line_ending())?;
            key_value_groups.format(f)?;
        }

        if f.indent_table_key_values() {
            f.dec_indent();
        }

        if f.indent_sub_tables() {
            filter_map_unique_keys(
                header.keys(),
                self.parent_table_or_array_of_table_keys(toml_version),
                toml_version,
            )
            .for_each(|_| f.dec_indent());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn table_only_header(
            r#"[package]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_only_header_with_basic_string_key(
            r#"[dependencies."unicase"]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_only_header_nested_keys(
            r#"[dependencies.unicase]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table(
            r#"
            [package]
            name = "toml-rs"
            version = "0.4.0"
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_with_full_comment(
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
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_begin_dangling_comment1(
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
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_end_dangling_comment1(
            r#"
            [header]
            key = "value"  # key trailing comment

            # key values end dangling comment 1-1
            # key values end dangling comment 1-2
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_end_dangling_comment2(
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
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_end_dangling_comment3(
            r#"
            [header]
            key = "value"  # key trailing comment

            # key values end dangling comment1
            # key values end dangling comment2
            "#
        ) -> Ok(source)
    }
}
