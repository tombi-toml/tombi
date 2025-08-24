use serde::Deserialize;
use tombi_comment_directive::{
    schema_store, source_schema, ValueTombiCommentDirective, ValueTombiCommentDirectiveImpl,
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::DocumentSchema;

use crate::comment_directive::into_directive_diagnostic;

pub async fn get_tombi_value_comment_directive<
    T: serde::de::DeserializeOwned + serde::Serialize + ValueTombiCommentDirectiveImpl,
>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> Option<ValueTombiCommentDirective<T>> {
    get_tombi_value_comment_directive_and_diagnostics(comment_directives)
        .await
        .0
}

pub async fn get_tombi_value_comment_directive_and_diagnostics<
    T: serde::de::DeserializeOwned + serde::Serialize + ValueTombiCommentDirectiveImpl,
>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> (
    Option<ValueTombiCommentDirective<T>>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    let schema_store = schema_store().await;

    for tombi_ast::TombiValueCommentDirective {
        content,
        content_range,
        ..
    } in comment_directives
    {
        let (root, errors) = tombi_parser::parse(&content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
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
                    .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
            );
            continue;
        }

        let (document_tree, errors) = root
            .into_document_tree_and_errors(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
            .into();

        if !errors.is_empty() {
            let mut diagnostics = Vec::new();
            for error in errors {
                error.set_diagnostics(&mut diagnostics);
            }
            total_diagnostics.extend(
                diagnostics
                    .into_iter()
                    .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
            );
        } else {
            let schema_uri = T::value_comment_directive_schema_url();
            let tombi_json::ValueNode::Object(object) = schema_store
                .fetch_schema_value(&schema_uri)
                .await
                // Value Comment Directive Schema is embedded in the crate
                .unwrap()
                .unwrap()
            else {
                panic!(
                    "Failed to fetch value comment directive schema from URL '{schema_uri}'. \
                    The fetched value was not an object."
                );
            };
            let document_schema = DocumentSchema::new(object, schema_uri.clone());

            let schema_context = tombi_schema_store::SchemaContext {
                toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
                root_schema: None,
                sub_schema_uri_map: None,
                store: schema_store,
                strict: None,
            };

            if let Err(diagnostics) = crate::validate(
                document_tree.clone(),
                source_schema(document_schema).await,
                &schema_context,
            )
            .await
            {
                total_diagnostics.extend(
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
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
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
                );
            }
        } else {
            total_document_tree_table = Some(document_tree.into());
        }
    }

    if let Some(total_document_tree_table) = total_document_tree_table {
        (
            ValueTombiCommentDirective::<T>::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            total_diagnostics,
        )
    } else {
        (None, total_diagnostics)
    }
}
