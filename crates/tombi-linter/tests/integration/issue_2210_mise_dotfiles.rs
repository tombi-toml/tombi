use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> std::path::PathBuf {
    project_root_path().join("schemas/issue-2210-mise-dotfiles.schema.json")
}

test_lint! {
    #[test]
    fn test_issue_2210_empty_dotfile_entry_matches_only_object_branch(
        r#"
        [dotfiles]
        "~/hello" = {}
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_issue_2210_string_dotfile_entry_matches_only_string_branch(
        r#"
        [dotfiles]
        "~/hello" = "dotfiles/hello"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_issue_2210_permissions_entry_matches_permissions_branch(
        r#"
        [dotfiles]
        "~/hello" = { source = "hello", permissions = "0600" }
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}
