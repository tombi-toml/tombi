use tombi_ast_syntax::AstNode as _;
use tombi_comment_directive::{
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, TombiCommentDirectiveImpl,
    document::TombiDocumentDirectiveContent,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree_syntax::IntoDocumentTreeAndErrors;

use crate::comment_directive::into_directive_diagnostic;

pub async fn get_tombi_document_comment_directive(
    root: &tombi_ast_syntax::Root<'_>,
) -> Option<TombiDocumentDirectiveContent> {
    get_tombi_document_comment_directive_and_diagnostics(root)
        .await
        .0
}

pub async fn get_tombi_document_comment_directive_and_diagnostics(
    root: &tombi_ast_syntax::Root<'_>,
) -> (
    Option<TombiDocumentDirectiveContent>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    use serde::Deserialize;

    let mut total_diagnostics = Vec::new();
    let mut tombi_directive_iter = root.tombi_document_comment_directives().peekable();

    if tombi_directive_iter.peek().is_some() {
        let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
        let mut total_document_tree_table: Option<tombi_document_tree_syntax::Table<'_>> = None;
        let schema_store = tombi_comment_directive_store::schema_store().await;
        let document_schema = comment_directive_document_schema(
            schema_store,
            TombiDocumentDirectiveContent::comment_directive_schema_url(),
        )
        .await;

        let source_schema = tombi_schema_store::SourceSchema::new(
            Some(document_schema),
            tombi_hashmap::IndexMap::with_capacity(0),
            Some(toml_version),
            None,
            Default::default(),
            Default::default(),
            Default::default(),
        );

        let schema_context = tombi_schema_store::SchemaContext {
            toml_version,
            root_schema: source_schema.root_schema.as_deref(),
            sub_schema_link_map: None,
            deprecated_lint_level: None,
            schema_format_rules: None,
            schema_lint_rules: None,
            schema_overrides: None,
            schema_visits: Default::default(),
            store: schema_store,
            strict: None,
        };

        // The merged document tree borrows every parse result and decoded pool,
        // so they are all kept until the tree is converted into an owned document.
        let directives = tombi_directive_iter.collect::<Vec<_>>();
        let parsed = directives
            .iter()
            .map(|directive| tombi_parser::parse(&directive.content))
            .collect::<Vec<_>>();
        let decoded = parsed
            .iter()
            .map(|parsed| parsed.root().decode_strings(toml_version))
            .collect::<Vec<_>>();

        for ((directive, parsed), decoded) in directives.iter().zip(&parsed).zip(&decoded) {
            let content_span = directive.content_span;
            // Check if there are any parsing errors
            if !parsed.errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in parsed.errors.iter().cloned() {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_span)),
                );
                continue;
            }

            let (document_tree, errors) = parsed
                .root()
                .into_document_tree_and_errors(toml_version, decoded)
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
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_span)),
                );
            } else if let Err(diagnostics) =
                crate::validate(document_tree.clone(), Some(&source_schema), &schema_context).await
            {
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_span)),
                );
            }
            if let Some(total_document_tree_table) = total_document_tree_table.as_mut() {
                if let Err(errors) = total_document_tree_table.merge(document_tree.into()) {
                    let mut diagnostics = Vec::new();
                    for error in errors {
                        error.set_diagnostics(&mut diagnostics);
                    }
                    total_diagnostics.extend(
                        diagnostics
                            .into_iter()
                            .map(|diagnostic| into_directive_diagnostic(&diagnostic, content_span)),
                    );
                }
            } else {
                total_document_tree_table = Some(document_tree.into());
            }
        }

        if let Some(total_document_tree_table) = total_document_tree_table {
            return (
                TombiDocumentDirectiveContent::deserialize(
                    &total_document_tree_table.into_document(toml_version),
                )
                .ok(),
                total_diagnostics,
            );
        }
    }

    (None, total_diagnostics)
}
