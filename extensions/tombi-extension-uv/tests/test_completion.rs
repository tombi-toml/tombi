#[cfg(test)]
mod tests {
    use tombi_config::TomlVersion;
    use tombi_document_tree::TryIntoDocumentTree;
    use tombi_extension::CompletionHint;
    use tombi_parser::parse;
    use tombi_schema_store::Accessor;
    use tower_lsp::lsp_types::TextDocumentIdentifier;
    use tower_lsp::lsp_types::Url;

    #[tokio::test]
    async fn test_completion_project_dependencies() {
        let content = r#"[project]
dependencies = ["requests=="]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 26); // After "requests=="
        let accessors = vec![
            Accessor::Key("project".to_string()),
            Accessor::Key("dependencies".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        assert!(!items.is_empty());
        // Should contain version completions
        assert!(items.iter().any(|item| item.label.contains("==")));
    }

    #[tokio::test]
    async fn test_completion_optional_dependencies() {
        let content = r#"[project]
[project.optional-dependencies]
dev = ["pytest>="]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(2, 16); // After "pytest>="
        let accessors = vec![
            Accessor::Key("project".to_string()),
            Accessor::Key("optional-dependencies".to_string()),
            Accessor::Key("dev".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        assert!(!items.is_empty());
    }

    #[tokio::test]
    async fn test_completion_tool_uv_dependencies() {
        let content = r#"[tool.uv]
dependencies = ["numpy=="]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 24); // After "numpy=="
        let accessors = vec![
            Accessor::Key("tool".to_string()),
            Accessor::Key("uv".to_string()),
            Accessor::Key("dependencies".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        assert!(!items.is_empty());
    }

    #[tokio::test]
    async fn test_completion_dependency_groups() {
        let content = r#"[dependency-groups]
test = ["pytest~="]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 17); // After "pytest~="
        let accessors = vec![
            Accessor::Key("dependency-groups".to_string()),
            Accessor::Key("test".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        assert!(!items.is_empty());
    }

    #[tokio::test]
    async fn test_no_completion_for_package_name() {
        let content = r#"[project]
dependencies = ["req"]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 20); // Within "req"
        let accessors = vec![
            Accessor::Key("project".to_string()),
            Accessor::Key("dependencies".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        // Should not provide version completion when cursor is within package name
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_completion_after_package_name() {
        let content = r#"[project]
dependencies = ["requests"]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 25); // After "requests"
        let accessors = vec![
            Accessor::Key("project".to_string()),
            Accessor::Key("dependencies".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        assert!(!items.is_empty());

        // Should contain operators and features bracket
        assert!(items.iter().any(|item| item.label == "["));
        assert!(items.iter().any(|item| item.label == "=="));
        assert!(items.iter().any(|item| item.label == ">="));
        assert!(items.iter().any(|item| item.label == "~="));
    }

    #[tokio::test]
    async fn test_completion_in_features() {
        let content = r#"[project]
dependencies = ["requests["]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 26); // After "["
        let accessors = vec![
            Accessor::Key("project".to_string()),
            Accessor::Key("dependencies".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        // Should at least suggest closing bracket
        assert!(items.iter().any(|item| item.label == "]"));
        // May also include actual features if PyPI is accessible
        println!(
            "Feature completion items: {:?}",
            items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_version_completion_with_existing_features() {
        let content = r#"[project]
dependencies = ["requests[socks]=="]
"#;
        let root = parse(content, TomlVersion::V1).into_syntax_node();
        let document_tree = tombi_ast::Root::cast(root)
            .unwrap()
            .try_into_document_tree(TomlVersion::V1)
            .unwrap();

        let text_document = TextDocumentIdentifier {
            uri: Url::parse("file:///test/pyproject.toml").unwrap(),
        };

        let position = tombi_text::Position::new(1, 34); // After "=="
        let accessors = vec![
            Accessor::Key("project".to_string()),
            Accessor::Key("dependencies".to_string()),
            Accessor::Index(0),
        ];

        let result = crate::completion(
            &text_document,
            &document_tree,
            position,
            &accessors,
            TomlVersion::V1,
            None,
        )
        .await
        .unwrap();

        assert!(result.is_some());
        let items = result.unwrap();
        assert!(!items.is_empty());
        // Should contain version numbers
        assert!(items
            .iter()
            .all(|item| !item.label.contains("==") && !item.label.contains("[")));
    }
}
