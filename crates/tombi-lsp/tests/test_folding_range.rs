mod folding_range_tests {
    use super::*;

    test_folding_range!(
        #[tokio::test]
        async fn simple_key_value_comment(
            r#"
            # Line 0
            # Line 1

            # Line 3
            # Line 4

            # Line 6
            # Line 7
            key = "value"

            # Line 10
            # Line 11
            "#,
        ) -> [
            0..4,
            6..7,
            10..11,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn simple_table_comment(
            r#"
            # Line 0
            # Line 1
            [table] # Line 2
            key = "value" # Line 3
            "#,
        ) -> [
            0..1,
            2..3,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn multiple_tables(
            r#"
            # Line 0
            [table1] # Line 1
            key1 = "value1"
            key2 = "value2" # Line 3

            # Line 5
            [table2] # Line 6
            key3 = "value3" # Line 7
            "#,
        ) -> [
            0..0,
            1..3,
            5..5,
            6..7,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn array_of_tables(
            r#"
            [[array]] # Line 0
            # Line 1
            key1 = "value1" # Line 2

            [[array]] # Line 4
            # Line 5
            key2 = "value2" # Line 6
            "#,
        ) -> [
            0..2,
            1..1,
            4..6,
            5..5,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn table_with_comments(
            r#"
            # Line 0
            # Line 1
            [table] # Line 2
            # Line 3
            # Line 4
            key1 = "value1"
            # Line 6
            # Line 7
            "#,
        ) -> [
            0..1,
            2..7,
            3..4,
            6..7,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn array_with_comments(
            r#"
            array = [ # Line 0

              # Line 2
              # Line 3
              1,
              # Line 5
              # Line 6
              2,
              # Line 8
              # Line 9

              # Line 11
            ] # Line 12
            "#,
        ) -> [
            0..12,
            2..3,
            5..6,
            8..11,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn nested_structure_with_comments(
            r#"
            # Line 0
            # Line 1
            [outer] # Line 2
            # Line 3
            # Line 4
            inner = [ # Line 5

                # Line 7
                # Line 8
                1,
                # Line 10
                # Line 11
            ] # Line 12
            # Line 13
            # Line 14

            # Line 16
            # Line 17
            "#,
        ) -> [
            0..1,
            2..17,
            3..4,
            5..12,
            7..8,
            10..11,
            13..17,
        ];
    );

    test_folding_range!(
        #[tokio::test]
        async fn array_of_tables_with_comments(
            r#"
            # Line 0
            # Line 1
            [[items]] # Line 2
            # Line 3
            # Line 4
            key1 = 1
            # Line 6
            # Line 7

            [[items]] # Line 9
            # Line 10
            # Line 11
            key2 = 2
            # Line 13
            # Line 14
            # Line 15
            # Line 16
            "#,
        ) -> [
            0..1,
            2..7,
            3..4,
            6..7,
            9..16,
            10..11,
            13..16,
        ];
    );

    #[macro_export]
    macro_rules! test_folding_range {
        (#[tokio::test] async fn $name:ident($source:expr $(,)?) -> [$($expected:expr),* $(,)?];) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use tombi_test_lib::tombi_schema_path;
                use tombi_lsp::handler::{handle_did_open, handle_folding_range};
                use tombi_lsp::Backend;
                use tower_lsp_server::{
                    ls_types::lsp::{
                        DidOpenTextDocumentParams,
                        FoldingRangeParams,
                        PartialResultParams,
                        TextDocumentIdentifier,
                        TextDocumentItem,
                        WorkDoneProgressParams,
                    },
                    LspService,
                };
                use itertools::Itertools;

                let (service, _) = LspService::new(|client| {
                    Backend::new(client, &tombi_lsp::backend::Options::default())
                });
                let backend = service.inner();

                let toml_file_uri = tombi_uri::Uri::from_file_path(tombi_schema_path())
                    .expect("failed to convert file path to URI");
                let toml_text = textwrap::dedent($source).trim().to_string();

                handle_did_open(
                    backend,
                    DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: toml_file_uri.clone().into(),
                            language_id: "toml".to_string(),
                            version: 0,
                            text: toml_text.clone(),
                        },
                    },
                )
                .await;

                let params = FoldingRangeParams {
                    text_document: TextDocumentIdentifier { uri: toml_file_uri.into() },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                };

                let Ok(Some(result)) = handle_folding_range(backend, params).await else {
                    panic!("failed to get folding range");
                };

                let expected: Vec<std::ops::Range<u32>> = vec![$($expected),*];
                let actual: Vec<std::ops::Range<u32>> = result
                    .into_iter()
                    .sorted_by_key(|r| (r.start_line, r.start_character))
                    .map(|r| r.start_line..r.end_line)
                    .collect();

                pretty_assertions::assert_eq!(actual, expected);

                Ok(())
            }
        };
    }
}
