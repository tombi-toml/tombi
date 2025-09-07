use serde::Deserialize;
use tombi_comment_directive::{
    value::{
        ArrayOfTableRules, CommonRules, InlineTableCommonRules, InlineTableRules,
        KeyArrayOfTableCommonRules, KeyCommonExtensibleRules, KeyTableCommonRules, LintOptions,
        RootTableCommonRules, RootTableRules, TableRules, TombiValueDirectiveContent,
        WithCommonRules, WithKeyRules,
    },
    TombiCommentDirectiveImpl, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::{IntoDocumentTreeAndErrors, TableKind};
use tombi_schema_store::SchemaUri;

use crate::comment_directive::into_directive_diagnostic;

pub async fn get_tombi_comment_directive_and_diagnostics<Rules>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> (
    Option<TombiValueDirectiveContent<Rules>>,
    Vec<tombi_diagnostic::Diagnostic>,
)
where
    Rules: serde::de::DeserializeOwned + serde::Serialize,
    TombiValueDirectiveContent<Rules>: TombiCommentDirectiveImpl,
{
    let schema_uri = TombiValueDirectiveContent::<Rules>::comment_directive_schema_url();

    let (document_tree_table, diagnostics) =
        get_comment_directive_document_tree_and_diagnostics(comment_directives, schema_uri).await;

    if let Some(total_document_tree_table) = document_tree_table {
        (
            TombiValueDirectiveContent::<Rules>::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            diagnostics,
        )
    } else {
        (None, diagnostics)
    }
}

pub async fn get_tombi_table_comment_directive_and_diagnostics(
    table: &tombi_document_tree::Table,
    accessors: &[tombi_schema_store::Accessor],
) -> (
    Option<TableRules>,
    Option<CommonRules>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    async fn inner_get_tombi_table_comment_directive_and_diagnostics(
        table: &tombi_document_tree::Table,
        accessors: &[tombi_schema_store::Accessor],
        comment_directives: &[tombi_ast::TombiValueCommentDirective],
    ) -> (
        Option<TableRules>,
        Option<CommonRules>,
        Vec<tombi_diagnostic::Diagnostic>,
    ) {
        match table.kind() {
            TableKind::InlineTable { .. } => {
                let (rules, diagnostics) =
                    get_tombi_rules_and_diagnostics::<InlineTableCommonRules>(&comment_directives)
                        .await;
                if let Some(WithCommonRules {
                    common,
                    value: InlineTableRules(table),
                }) = rules
                {
                    (Some(table), Some(common), diagnostics)
                } else {
                    (None, None, diagnostics)
                }
            }
            TableKind::Table
            | TableKind::ParentTable
            | TableKind::ParentKey
            | TableKind::KeyValue => {
                if accessors
                    .iter()
                    .any(|accessor| matches!(accessor, tombi_schema_store::Accessor::Index(_)))
                {
                    let (rules, diagnostics) = get_tombi_rules_and_diagnostics::<
                        KeyArrayOfTableCommonRules,
                    >(&comment_directives)
                    .await;
                    if let Some(WithKeyRules {
                        value:
                            WithCommonRules {
                                common,
                                value: ArrayOfTableRules { table, .. },
                            },
                        ..
                    }) = rules
                    {
                        (Some(table), Some(common), diagnostics)
                    } else {
                        (None, None, diagnostics)
                    }
                } else {
                    let (rules, diagnostics) =
                        get_tombi_rules_and_diagnostics::<KeyTableCommonRules>(&comment_directives)
                            .await;
                    if let Some(WithKeyRules {
                        value:
                            WithCommonRules {
                                common,
                                value: table,
                            },
                        ..
                    }) = rules
                    {
                        (Some(table), Some(common), diagnostics)
                    } else {
                        (None, None, diagnostics)
                    }
                }
            }
            TableKind::Root => {
                let (rules, diagnostics) =
                    get_tombi_rules_and_diagnostics::<RootTableCommonRules>(&comment_directives)
                        .await;

                if let Some(WithCommonRules {
                    common,
                    value: RootTableRules { table, .. },
                }) = rules
                {
                    (Some(table), Some(common), diagnostics)
                } else {
                    (None, None, diagnostics)
                }
            }
        }
    }

    let mut total_diagnostics = vec![];
    let (mut table_rules, mut common_rules) =
        if let Some(comment_directives) = table.comment_directives() {
            let (table_rules, common_rules, diagnostics) =
                inner_get_tombi_table_comment_directive_and_diagnostics(
                    table,
                    accessors,
                    comment_directives,
                )
                .await;

            total_diagnostics.extend(diagnostics);
            (table_rules, common_rules)
        } else {
            (None, None)
        };

    if let Some(comment_directives) = table.inner_comment_directives() {
        let (inner_table_rules, inner_common_rules, diagnostics) =
            inner_get_tombi_table_comment_directive_and_diagnostics(
                table,
                accessors,
                comment_directives,
            )
            .await;

        if table_rules.is_none() {
            table_rules = inner_table_rules;
        }
        if common_rules.is_none() {
            common_rules = inner_common_rules;
        }
        for diagnostic in diagnostics {
            if !total_diagnostics.contains(&diagnostic) {
                total_diagnostics.push(diagnostic);
            }
        }
    };

    (table_rules, common_rules, total_diagnostics)
}

pub async fn get_tombi_key_rules_and_diagnostics(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> (
    Option<KeyCommonExtensibleRules>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    get_tombi_rules_and_diagnostics::<KeyCommonExtensibleRules>(comment_directives).await
}

pub async fn get_tombi_rules_and_diagnostics<Rules>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> (Option<Rules>, Vec<tombi_diagnostic::Diagnostic>)
where
    Rules: serde::de::DeserializeOwned + serde::Serialize,
    TombiValueDirectiveContent<Rules>: TombiCommentDirectiveImpl,
{
    let (comment_directive, diagnostics) =
        get_tombi_comment_directive_and_diagnostics(comment_directives).await;

    if let Some(TombiValueDirectiveContent {
        lint: Some(LintOptions { rules, .. }),
        ..
    }) = comment_directive
    {
        (rules, diagnostics)
    } else {
        (None, diagnostics)
    }
}

pub async fn get_tombi_value_rules_and_diagnostics_with_key_rules<Rules>(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
    accessors: &[tombi_schema_store::Accessor],
) -> (Option<Rules>, Vec<tombi_diagnostic::Diagnostic>)
where
    Rules: serde::de::DeserializeOwned + serde::Serialize,
    TombiValueDirectiveContent<Rules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyRules<Rules>>: TombiCommentDirectiveImpl,
{
    if let Some(tombi_schema_store::Accessor::Index(_)) = accessors.last() {
        get_tombi_rules_and_diagnostics(comment_directives).await
    } else {
        let (comment_directive, diagnostics) =
            get_tombi_comment_directive_and_diagnostics::<WithKeyRules<Rules>>(comment_directives)
                .await;

        if let Some(TombiValueDirectiveContent {
            lint: Some(LintOptions { rules, .. }),
            ..
        }) = comment_directive
        {
            (rules.map(|rules| rules.value), diagnostics)
        } else {
            (None, diagnostics)
        }
    }
}

pub async fn get_comment_directive_document_tree_and_diagnostics(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
    schema_uri: SchemaUri,
) -> (
    Option<tombi_document_tree::Table>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;
    let mut total_diagnostics = Vec::new();
    let schema_store = tombi_comment_directive_store::schema_store().await;

    let source_schema = tombi_schema_store::SourceSchema {
        root_schema: Some(comment_directive_document_schema(schema_store, schema_uri).await),
        sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
    };

    let schema_context = tombi_schema_store::SchemaContext {
        toml_version: TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
        root_schema: source_schema.root_schema.as_ref(),
        sub_schema_uri_map: None,
        store: schema_store,
        strict: None,
    };

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
            if let Err(diagnostics) =
                crate::validate(document_tree.clone(), Some(&source_schema), &schema_context).await
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

    (total_document_tree_table, total_diagnostics)
}
