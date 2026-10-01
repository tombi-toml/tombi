use tombi_ast_syntax::{GetHeaderAccessors, HeaderAccessorsTracker};
use tombi_config::TomlVersion;

/// `HeaderAccessorsTracker` accumulates the `[[...]]` headers instead of
/// rescanning the previous siblings, and must give the same accessors as
/// `get_header_accessors` for every table.
macro_rules! test_header_accessors {
    (#[test] fn $name:ident($source:expr $(,)?)) => {
        #[test]
        fn $name() {
            let toml_version = TomlVersion::default();
            let (root, errors) = tombi_parser::parse($source).into_root_and_errors();
            assert!(errors.is_empty(), "{errors:?}");

            let mut tracker = HeaderAccessorsTracker::new();
            let mut tables = 0;
            for table in root.table_or_array_of_tables() {
                pretty_assertions::assert_eq!(
                    tracker.header_accessors(&table, toml_version),
                    table.get_header_accessors(toml_version),
                );
                tables += 1;
            }
            assert!(tables > 0);
        }
    };
}

test_header_accessors! {
    #[test]
    fn tables_only(
        "[a]\n[a.b]\n[c]\n"
    )
}

test_header_accessors! {
    #[test]
    fn array_of_tables_with_sub_tables(
        "[[x]]\na = 1\n[x.sub]\nb = 2\n[[x]]\na = 3\n[x.sub]\nb = 4\n"
    )
}

test_header_accessors! {
    #[test]
    fn nested_array_of_tables(
        "[[x]]\n[[x.y]]\n[[x.y]]\n[x.y.z]\n[[x]]\n[[x.y]]\n[x.y.z]\n"
    )
}

test_header_accessors! {
    #[test]
    fn array_of_tables_interleaved_with_tables(
        "[[x]]\n[t]\n[x.sub]\n[[x]]\n[u]\n[x.sub]\n"
    )
}

test_header_accessors! {
    #[test]
    fn different_array_of_tables_between(
        "[[x]]\n[[y]]\n[x.z]\n[[x]]\n[[y]]\n[y.z]\n[x.z]\n"
    )
}

test_header_accessors! {
    #[test]
    fn dotted_array_of_tables_with_parent_table(
        "[[a.b]]\nx = 1\n[[a.b]]\nx = 2\n[a.b.c]\n[a]\nz = 1\n[[a.b]]\n[a.b.c]\n"
    )
}

test_header_accessors! {
    #[test]
    fn quoted_keys(
        "[[\"x\"]]\n[x.sub]\n[[x]]\n[\"x\".sub]\n"
    )
}
