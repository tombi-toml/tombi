use std::fmt::Write;

use itertools::Itertools;

use super::Format;

impl Format for tombi_ast::Root {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        f.reset();

        let key_values = self.key_values().collect_vec();
        let table_or_array_of_tables = self.table_or_array_of_tables().collect_vec();
        let key_values_dangling_comments = self.key_values_dangling_comments();

        if !key_values.is_empty() {
            self.key_values_begin_dangling_comments().format(f)?;

            for (i, key_value) in key_values.iter().enumerate() {
                if i != 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                key_value.format(f)?;
            }

            self.key_values_end_dangling_comments().format(f)?;
        } else {
            key_values_dangling_comments.format(f)?;
        }

        if !(table_or_array_of_tables.is_empty()
            || key_values.is_empty() && key_values_dangling_comments.is_empty())
        {
            write!(f, "{}", f.line_ending())?;
            write!(f, "{}", f.line_ending())?;
        }

        let mut header = Header::Root;
        for (i, table_or_array_of_tables) in table_or_array_of_tables.iter().enumerate() {
            if i != 0 {
                write!(f, "{}", f.line_ending())?;
            }
            match table_or_array_of_tables {
                tombi_ast::TableOrArrayOfTable::Table(table) => {
                    let header_keys = table.header().unwrap().keys();
                    let key_value_size = table.key_values().count();
                    let has_dangling_comments = !table.key_values_dangling_comments().is_empty();

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
                        !array_of_table.key_values_dangling_comments().is_empty();

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

impl Format for tombi_ast::RootItem {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            tombi_ast::RootItem::Table(it) => it.format(f),
            tombi_ast::RootItem::ArrayOfTable(it) => it.format(f),
            tombi_ast::RootItem::KeyValue(it) => it.format(f),
        }
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
    use crate::test_format;

    test_format! {
        #[test]
        fn empty_table_space_on_own_subtable(
            r#"
            [foo]
            [foo.bar]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_table_space_on_other_table(
            r#"
            [foo]

            [bar.baz]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_table_space_on_own_array_of_subtables(
            r#"
            [foo]
            [[foo.bar]]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_table_space_on_other_array_of_tables(
            r#"
            [foo]

            [[bar.baz]]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_table_space_on_other_array_of_tables_with_comments(
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
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_array_of_tables_space_on_own_subtable(
            r#"
            [[foo]]
            [foo.bar]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_array_of_tables_space_on_own_subtable_with_comments(
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
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_array_of_tables_space_on_other_subtable(
            r#"
            [[foo]]

            [bar.baz]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_array_of_tables_space_on_other_subtable_with_comments(
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
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_array_of_tables_space_on_same_array_of_tables(
            r#"
            [[foo]]

            [[foo]]
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn empty_array_of_tables_space_on_same_array_of_tables_with_comment(
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
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn only_dangling_comment1(
            r#"
            # root dangling comment
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn only_dangling_comment2(
            r#"
            # root dangling comment 1-1
            # root dangling comment 1-2

            # root dangling comment 2-1
            # root dangling comment 2-1
            # root dangling comment 2-3

            # root dangling comment 3-1
            "#
        ) -> Ok(source);
    }
}
