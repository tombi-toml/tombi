use itertools::Itertools;
use serde::Deserialize;
use tombi_comment_directive::{
    value::{
        ArrayCommonFormatRules, ArrayCommonLintRules, ArrayOfTableCommonFormatRules,
        ArrayOfTableCommonLintRules, ArrayOfTableLintRules, InlineTableCommonFormatRules,
        InlineTableCommonLintRules, InlineTableLintRules, KeyArrayOfTableCommonFormatRules,
        KeyArrayOfTableCommonLintRules, KeyCommonExtensibleLintRules, KeyFormatRules,
        KeyTableCommonFormatRules, KeyTableCommonLintRules, LintOptions,
        ParentTableCommonLintRules, RootTableCommonLintRules, RootTableLintRules,
        TableCommonFormatRules, TableCommonLintRules, TombiValueDirectiveContent,
        WithCommonExtensibleLintRules, WithCommonLintRules, WithKeyFormatRules, WithKeyLintRules,
        WithKeyTableLintRules,
    },
    TombiCommentDirectiveImpl, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::{ArrayKind, IntoDocumentTreeAndErrors, TableKind};
use tombi_schema_store::{Accessor, SchemaUri};

use crate::comment_directive::into_directive_diagnostic;

pub async fn get_tombi_value_comment_directive_and_diagnostics<FormatRules, LintRules>(
    comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
) -> (
    Option<TombiValueDirectiveContent<FormatRules, LintRules>>,
    Vec<tombi_diagnostic::Diagnostic>,
)
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: serde::de::DeserializeOwned,
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
{
    let schema_uri =
        TombiValueDirectiveContent::<FormatRules, LintRules>::comment_directive_schema_url();

    let (document_tree_table, diagnostics) =
        get_comment_directive_document_tree_and_diagnostics(comment_directives, schema_uri).await;

    if let Some(total_document_tree_table) = document_tree_table {
        (
            TombiValueDirectiveContent::<FormatRules, LintRules>::deserialize(
                &total_document_tree_table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok(),
            diagnostics,
        )
    } else {
        (None, diagnostics)
    }
}

pub async fn get_tombi_array_comment_directive_and_diagnostics(
    array: &tombi_document_tree::Array,
    accessors: &[tombi_schema_store::Accessor],
) -> (
    Option<ArrayCommonLintRules>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    async fn _get_tombi_array_comment_directive_and_diagnostics(
        array: &tombi_document_tree::Array,
        accessors: &[tombi_schema_store::Accessor],
        comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
        is_inner_comment_directives: bool,
    ) -> (
        Option<ArrayCommonLintRules>,
        Vec<tombi_diagnostic::Diagnostic>,
    ) {
        match array.kind() {
            ArrayKind::Array => {
                let (rules, diagnostics) = if is_inner_comment_directives {
                    get_tombi_value_rules_and_diagnostics::<
                        ArrayCommonFormatRules,
                        ArrayCommonLintRules,
                    >(comment_directives)
                    .await
                } else {
                    get_tombi_key_table_value_rules_and_diagnostics::<
                        ArrayCommonFormatRules,
                        ArrayCommonLintRules,
                    >(comment_directives, accessors)
                    .await
                };
                if let Some(array_common_rules) = rules {
                    (Some(array_common_rules), diagnostics)
                } else {
                    (None, diagnostics)
                }
            }
            ArrayKind::ArrayOfTable | ArrayKind::ParentArrayOfTable => {
                let (rules, diagnostics) = get_tombi_key_value_rules_and_diagnostics::<
                    ArrayOfTableCommonFormatRules,
                    ArrayOfTableCommonLintRules,
                >(comment_directives, accessors)
                .await;
                if let Some(WithCommonLintRules {
                    common,
                    value: ArrayOfTableLintRules { array, .. },
                }) = rules
                {
                    (
                        Some(WithCommonLintRules {
                            common,
                            value: array,
                        }),
                        diagnostics,
                    )
                } else {
                    (None, diagnostics)
                }
            }
        }
    }

    let mut total_diagnostics = vec![];
    let mut array_common_rules = if let Some(comment_directives) = array.comment_directives() {
        let (array_common_rules, diagnostics) =
            if let Some(inner_comment_directives) = array.inner_comment_directives() {
                _get_tombi_array_comment_directive_and_diagnostics(
                    array,
                    accessors,
                    comment_directives
                        .iter()
                        .chain(inner_comment_directives)
                        .collect_vec(),
                    false,
                )
                .await
            } else {
                _get_tombi_array_comment_directive_and_diagnostics(
                    array,
                    accessors,
                    comment_directives.iter().collect_vec(),
                    false,
                )
                .await
            };

        total_diagnostics.extend(diagnostics);

        array_common_rules
    } else {
        None
    };

    if let Some(inner_comment_directives) = array.inner_comment_directives() {
        let (inner_array_common_rules, diagnostics) =
            _get_tombi_array_comment_directive_and_diagnostics(
                array,
                accessors,
                inner_comment_directives.iter(),
                true,
            )
            .await;

        if array_common_rules.is_none() {
            array_common_rules = inner_array_common_rules;
        }
        for diagnostic in diagnostics {
            if !total_diagnostics.contains(&diagnostic) {
                total_diagnostics.push(diagnostic);
            }
        }
    };

    (array_common_rules, total_diagnostics)
}

pub async fn get_tombi_table_comment_directive_and_diagnostics(
    table: &tombi_document_tree::Table,
    accessors: &[tombi_schema_store::Accessor],
) -> (
    Option<TableCommonLintRules>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    async fn _get_tombi_table_comment_directive_and_diagnostics(
        table: &tombi_document_tree::Table,
        accessors: &[tombi_schema_store::Accessor],
        comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
        is_inner_comment_directives: bool,
    ) -> (
        Option<TableCommonLintRules>,
        Vec<tombi_diagnostic::Diagnostic>,
    ) {
        match table.kind() {
            TableKind::InlineTable { .. } => {
                let (rules, diagnostics) = get_tombi_key_value_rules_and_diagnostics::<
                    InlineTableCommonFormatRules,
                    InlineTableCommonLintRules,
                >(comment_directives, accessors)
                .await;
                if let Some(WithCommonLintRules {
                    common,
                    value: InlineTableLintRules(table),
                }) = rules
                {
                    (
                        Some(WithCommonLintRules {
                            common,
                            value: table,
                        }),
                        diagnostics,
                    )
                } else {
                    (None, diagnostics)
                }
            }
            TableKind::Table | TableKind::ParentTable => {
                if is_inner_comment_directives {
                    get_tombi_value_rules_and_diagnostics::<
                        TableCommonFormatRules,
                        TableCommonLintRules,
                    >(comment_directives)
                    .await
                } else if matches!(accessors.last(), Some(Accessor::Index(_))) {
                    let (rules, diagnostics) = get_tombi_value_rules_and_diagnostics::<
                        KeyArrayOfTableCommonFormatRules,
                        KeyArrayOfTableCommonLintRules,
                    >(comment_directives)
                    .await;
                    if let Some(WithKeyLintRules {
                        value:
                            WithCommonLintRules {
                                common,
                                value: ArrayOfTableLintRules { table, .. },
                            },
                        ..
                    }) = rules
                    {
                        (
                            Some(WithCommonLintRules {
                                common,
                                value: table,
                            }),
                            diagnostics,
                        )
                    } else {
                        (None, diagnostics)
                    }
                } else {
                    let (rules, diagnostics) = get_tombi_value_rules_and_diagnostics::<
                        KeyTableCommonFormatRules,
                        KeyTableCommonLintRules,
                    >(comment_directives)
                    .await;
                    if let Some(WithKeyLintRules { value, .. }) = rules {
                        (Some(value), diagnostics)
                    } else {
                        (None, diagnostics)
                    }
                }
            }
            TableKind::KeyValue | TableKind::ParentKey => {
                let (rules, diagnostics) = get_tombi_value_rules_and_diagnostics::<
                    TableCommonFormatRules,
                    ParentTableCommonLintRules,
                >(comment_directives)
                .await;

                if let Some(WithCommonExtensibleLintRules {
                    common,
                    value: table,
                }) = rules
                {
                    (
                        Some(WithCommonLintRules {
                            common,
                            value: table,
                        }),
                        diagnostics,
                    )
                } else {
                    (None, diagnostics)
                }
            }
            TableKind::Root => {
                let (rules, diagnostics) = get_tombi_value_rules_and_diagnostics::<
                    TableCommonFormatRules,
                    RootTableCommonLintRules,
                >(comment_directives)
                .await;

                if let Some(WithCommonLintRules {
                    common,
                    value: RootTableLintRules { table, .. },
                }) = rules
                {
                    (
                        Some(WithCommonLintRules {
                            common,
                            value: table,
                        }),
                        diagnostics,
                    )
                } else {
                    (None, diagnostics)
                }
            }
        }
    }

    let mut total_diagnostics = vec![];
    let mut table_common_rules = if let Some(comment_directives) = table.comment_directives() {
        let (table_common_rules, diagnostics) =
            if let Some(inner_comment_directives) = table.inner_comment_directives() {
                _get_tombi_table_comment_directive_and_diagnostics(
                    table,
                    accessors,
                    comment_directives
                        .iter()
                        .chain(inner_comment_directives)
                        .collect_vec(),
                    false,
                )
                .await
            } else {
                _get_tombi_table_comment_directive_and_diagnostics(
                    table,
                    accessors,
                    comment_directives.iter().collect_vec(),
                    false,
                )
                .await
            };

        total_diagnostics.extend(diagnostics);
        table_common_rules
    } else {
        None
    };

    if let Some(inner_comment_directives) = table.inner_comment_directives() {
        let (inner_table_common_rules, diagnostics) =
            _get_tombi_table_comment_directive_and_diagnostics(
                table,
                accessors,
                inner_comment_directives.iter(),
                true,
            )
            .await;

        if table_common_rules.is_none() {
            table_common_rules = inner_table_common_rules;
        }

        for diagnostic in diagnostics {
            if !total_diagnostics.contains(&diagnostic) {
                total_diagnostics.push(diagnostic);
            }
        }
    };

    (table_common_rules, total_diagnostics)
}

pub async fn get_tombi_key_rules_and_diagnostics(
    comment_directives: &[tombi_ast::TombiValueCommentDirective],
) -> (
    Option<KeyCommonExtensibleLintRules>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    get_tombi_value_rules_and_diagnostics::<KeyFormatRules, KeyCommonExtensibleLintRules>(
        comment_directives,
    )
    .await
}

pub async fn get_tombi_value_rules_and_diagnostics<FormatRules, LintRules>(
    comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
) -> (Option<LintRules>, Vec<tombi_diagnostic::Diagnostic>)
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: serde::de::DeserializeOwned,
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
{
    let (comment_directive, diagnostics) =
        get_tombi_value_comment_directive_and_diagnostics(comment_directives).await;

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

pub async fn get_tombi_key_value_rules_and_diagnostics<FormatRules, LintRules>(
    comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
    accessors: &[tombi_schema_store::Accessor],
) -> (Option<LintRules>, Vec<tombi_diagnostic::Diagnostic>)
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: serde::de::DeserializeOwned,
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyFormatRules<FormatRules>, WithKeyLintRules<LintRules>>:
        TombiCommentDirectiveImpl,
{
    if let Some(tombi_schema_store::Accessor::Index(_)) = accessors.last() {
        get_tombi_value_rules_and_diagnostics(comment_directives).await
    } else {
        let (comment_directive, diagnostics) = get_tombi_value_comment_directive_and_diagnostics::<
            WithKeyFormatRules<FormatRules>,
            WithKeyLintRules<LintRules>,
        >(comment_directives)
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

pub async fn get_tombi_key_table_value_rules_and_diagnostics<FormatRules, LintRules>(
    comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
    accessors: &[tombi_schema_store::Accessor],
) -> (Option<LintRules>, Vec<tombi_diagnostic::Diagnostic>)
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: serde::de::DeserializeOwned,
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
    TombiValueDirectiveContent<WithKeyFormatRules<FormatRules>, WithKeyTableLintRules<LintRules>>:
        TombiCommentDirectiveImpl,
{
    if let Some(tombi_schema_store::Accessor::Index(_)) = accessors.last() {
        get_tombi_value_rules_and_diagnostics(comment_directives).await
    } else {
        let (comment_directive, diagnostics) = get_tombi_value_comment_directive_and_diagnostics::<
            WithKeyFormatRules<FormatRules>,
            WithKeyTableLintRules<LintRules>,
        >(comment_directives)
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
    comment_directives: impl IntoIterator<Item = &tombi_ast::TombiValueCommentDirective>,
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
        let (root, errors) = tombi_parser::parse(content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
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
        } else if let Err(diagnostics) =
            crate::validate(document_tree.clone(), Some(&source_schema), &schema_context).await
        {
            total_diagnostics.extend(
                diagnostics
                    .into_iter()
                    .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
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
                        .map(|diagnostic| into_directive_diagnostic(&diagnostic, *content_range)),
                );
            }
        } else {
            total_document_tree_table = Some(document_tree.into());
        }
    }

    (total_document_tree_table, total_diagnostics)
}
