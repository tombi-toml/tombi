use std::borrow::Cow;

use tombi_ast_syntax::AstNode;
use tombi_document_tree_syntax::TryIntoDocumentTree;
use tombi_schema_store::{CurrentSchema, SchemaContext};

mod change;
mod edit;
mod rule;

use change::Change;
use edit::Edit;

pub(crate) async fn edit<'a>(
    root: tombi_ast_syntax::Root,
    source_path: Option<&'a std::path::Path>,
    schema_context: &'a SchemaContext<'a>,
) -> tombi_ast_syntax::Root {
    let Ok(document_tree) = root
        .clone()
        .try_into_document_tree(schema_context.toml_version)
    else {
        return root;
    };
    let current_schema = schema_context.root_schema.and_then(|document_schema| {
        document_schema
            .schema_view
            .as_ref()
            .map(|schema_view| CurrentSchema {
                schema_view: schema_view.clone(),
                semantic_schema: document_schema.semantic_schema.clone(),
                schema_uri: Cow::Borrowed(&document_schema.schema_uri),
                definitions: Cow::Borrowed(&document_schema.definitions),
                strict: document_schema.strict,
            })
    });

    let document_value = tombi_document_tree_syntax::Value::from(document_tree);
    let changes = root
        .edit(
            &document_value,
            &[],
            source_path,
            current_schema.as_ref(),
            schema_context,
        )
        .await;
    if changes.is_empty() {
        return root;
    }

    let source = match change::apply(root.syntax(), changes) {
        Ok(source) => source,
        Err(error) => {
            log::error!("failed to apply formatter source rewrite: {error:?}");
            return root;
        }
    };
    let (edited_root, errors) = tombi_parser::parse(&source).into_root_and_errors();
    if errors.is_empty() {
        edited_root
    } else {
        log::error!("formatter source rewrite produced invalid TOML: {errors:#?}");
        root
    }
}
