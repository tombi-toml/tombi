use tombi_comment_directive::{
    document::TombiDocumentDirectiveContent, TombiCommentDirectiveImpl,
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;

use crate::comment_directive::into_directive_diagnostic;

pub async fn get_tombi_document_comment_directive(
    root: &tombi_ast::Root,
) -> Option<TombiDocumentDirectiveContent> {
    get_tombi_document_comment_directive_and_diagnostics(root)
        .await
        .0
}

pub async fn get_tombi_document_comment_directive_and_diagnostics(
    root: &tombi_ast::Root,
) -> (
    Option<TombiDocumentDirectiveContent>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    use serde::Deserialize;

    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    let tombi_directives = root.tombi_document_comment_directives();
    if !tombi_directives.is_empty() {
        let schema_store = tombi_comment_directive_store::schema_store().await;
        let document_schema = comment_directive_document_schema(
            schema_store,
            TombiDocumentDirectiveContent::comment_directive_schema_url(),
        )
        .await;
        let source_schema = tombi_schema_store::SourceSchema {
            root_schema: Some(document_schema),
            sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
        };
        let schema_context = tombi_schema_store::SchemaContext {
            toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
            root_schema: source_schema.root_schema.as_ref(),
            sub_schema_uri_map: None,
            store: schema_store,
            strict: None,
        };

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
                if let Err(diagnostics) =
                    crate::validate(document_tree.clone(), Some(&source_schema), &schema_context)
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
            TombiDocumentDirectiveContent::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            total_diagnostics,
        )
    } else {
        (None, total_diagnostics)
    }
}
