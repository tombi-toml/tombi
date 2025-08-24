use tombi_comment_directive::{
    document_comment_directive_document_schema, schema_store, TombiDocumentCommentDirective,
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;

use crate::comment_directive::into_directive_diagnostic;

pub async fn get_tombi_document_comment_directive(
    root: &tombi_ast::Root,
) -> Option<TombiDocumentCommentDirective> {
    get_tombi_document_comment_directive_and_diagnostics(root)
        .await
        .0
}

pub async fn get_tombi_document_comment_directive_and_diagnostics(
    root: &tombi_ast::Root,
) -> (
    Option<TombiDocumentCommentDirective>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    use serde::Deserialize;

    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    if let Some(tombi_directives) = root.tombi_document_comment_directives() {
        let schema_store = schema_store().await;
        for tombi_ast::TombiDocumentCommentDirective {
            content,
            content_range,
            ..
        } in tombi_directives
        {
            let (root, errors) =
                tombi_parser::parse(&content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
                    .into_root_and_errors();
            // Check if there are any parsing errors
            if !errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in errors {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_range)),
                );
                continue;
            }

            let (document_tree, errors) = root
                .into_document_tree_and_errors(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
                .into();

            // Check for errors during document tree construction
            if !errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in errors {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_range)),
                );
            } else {
                let document_schema = document_comment_directive_document_schema().await;
                let schema_context = tombi_schema_store::SchemaContext {
                    toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
                    root_schema: None,
                    sub_schema_uri_map: None,
                    store: schema_store,
                    strict: None,
                };

                if let Err(diagnostics) = crate::validate(
                    document_tree.clone(),
                    &tombi_schema_store::SourceSchema {
                        root_schema: Some(document_schema),
                        sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
                    },
                    &schema_context,
                )
                .await
                {
                    total_diagnostics.extend(
                        diagnostics.into_iter().map(|diagnostic| {
                            into_directive_diagnostic(&diagnostic, content_range)
                        }),
                    );
                }
            }
            if let Some(total_document_tree_table) = total_document_tree_table.as_mut() {
                if let Err(errors) = total_document_tree_table.merge(document_tree.into()) {
                    let mut diagnostics = Vec::new();
                    for error in errors {
                        error.set_diagnostics(&mut diagnostics);
                    }
                    total_diagnostics.extend(
                        diagnostics.into_iter().map(|diagnostic| {
                            into_directive_diagnostic(&diagnostic, content_range)
                        }),
                    );
                }
            } else {
                total_document_tree_table = Some(document_tree.into());
            }
        }
    }

    if let Some(total_document_tree_table) = total_document_tree_table {
        (
            TombiDocumentCommentDirective::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            total_diagnostics,
        )
    } else {
        (None, total_diagnostics)
    }
}
