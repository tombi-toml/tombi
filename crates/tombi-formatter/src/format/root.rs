use std::fmt::Write;

use itertools::Itertools;

use super::Format;

impl Format for tombi_ast::Root {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        f.reset();

        let dangling_comment_groups = self.dangling_comment_groups().collect_vec();
        dangling_comment_groups.format(f)?;

        let key_value_groups = self.key_value_groups().collect_vec();
        if !dangling_comment_groups.is_empty() && !key_value_groups.is_empty() {
            write!(f, "{}", f.line_ending())?;
            write!(f, "{}", f.line_ending())?;
        }
        key_value_groups.format(f)?;

        let table_or_array_of_tables = self.table_or_array_of_tables().collect_vec();

        if (!dangling_comment_groups.is_empty() || !key_value_groups.is_empty())
            && !table_or_array_of_tables.is_empty()
        {
            write!(f, "{}", f.line_ending())?;
            write!(f, "{}", f.line_ending())?;
        }
        table_or_array_of_tables.format(f)?;

        Ok(())
    }
}

impl Format for Vec<tombi_ast::TableOrArrayOfTable> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let mut header = Header::Root;
        for (i, table_or_array_of_table) in self.iter().enumerate() {
            if i != 0 {
                write!(f, "{}", f.line_ending())?;
            }
            match table_or_array_of_table {
                tombi_ast::TableOrArrayOfTable::Table(table) => {
                    let header_keys = table.header().unwrap().keys();
                    let key_value_size = table.key_values().count();
                    let has_dangling_comments = table.dangling_comment_groups().next().is_some();

                    match header {
                        Header::Root => {}
                        Header::Table {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        }
                        | Header::ArrayOfTable {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        } => {
                            if key_value_size > 0
                                || !header_keys.starts_with(&pre_header_keys)
                                || has_dangling_comments
                            {
                                write!(f, "{}", f.line_ending())?;
                            }
                        }
                    };
                    table.format(f)?;

                    header = Header::Table {
                        header_keys,
                        key_value_size,
                        has_dangling_comments,
                    };
                }
                tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                    let header_keys = array_of_table.header().unwrap().keys();
                    let key_value_size = array_of_table.key_values().count();
                    let has_dangling_comments =
                        array_of_table.dangling_comment_groups().next().is_some();

                    match header {
                        Header::Root => {}
                        Header::Table {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        } => {
                            if key_value_size > 0
                                || !header_keys.starts_with(&pre_header_keys)
                                || has_dangling_comments
                            {
                                write!(f, "{}", f.line_ending())?;
                            }
                        }
                        Header::ArrayOfTable {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        } => {
                            if key_value_size > 0
                                || !header_keys.starts_with(&pre_header_keys)
                                || pre_header_keys.same_as(&header_keys)
                                || has_dangling_comments
                            {
                                write!(f, "{}", f.line_ending())?;
                            }
                        }
                    };

                    array_of_table.format(f)?;

                    header = Header::ArrayOfTable {
                        header_keys,
                        key_value_size,
                        has_dangling_comments,
                    };
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Header {
    Root,

    Table {
        header_keys: tombi_ast::AstChildren<tombi_ast::Key>,
        key_value_size: usize,
        has_dangling_comments: bool,
    },

    ArrayOfTable {
        header_keys: tombi_ast::AstChildren<tombi_ast::Key>,
        key_value_size: usize,
        has_dangling_comments: bool,
    },
}

#[cfg(test)]
mod test {
    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_own_subtable(
            r#"
            [foo]
            [foo.bar]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_other_table(
            r#"
            [foo]

            [bar.baz]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_own_array_of_sub_tables(
            r#"
            [foo]
            [[foo.bar]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_other_array_of_tables(
            r#"
            [foo]

            [[bar.baz]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_other_array_of_tables_with_comments(
            r#"
            [foo]  # header table comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            # table header leading comment1
            # table header leading comment2
            [[bar.baz]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_own_subtable(
            r#"
            [[foo]]
            [foo.bar]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_own_subtable_with_comments(
            r#"
            [[foo]]  # header trailing comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            # table header leading comment1
            # table header leading comment2
            [foo.bar]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_other_subtable(
            r#"
            [[foo]]

            [bar.baz]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_other_subtable_with_comments(
            r#"
            [[foo]]  # header trailing comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            [bar.baz]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_same_array_of_tables(
            r#"
            [[foo]]

            [[foo]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_same_array_of_tables_with_comment(
            r#"
            [[foo]]  # header trailing comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            # table header leading comment1
            # table header leading comment2
            [[foo]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn only_dangling_comment1(
            r#"
            # root dangling comment
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn only_dangling_comment2(
            r#"
            # root dangling comment 1-1
            # root dangling comment 1-2

            # root dangling comment 2-1
            # root dangling comment 2-1
            # root dangling comment 2-3

            # root dangling comment 3-1
            "#
        ) -> Ok(source)
    }
}
