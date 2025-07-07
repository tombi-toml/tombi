mod comment;
mod schema_completion;
mod value;

use std::borrow::Cow;

use ahash::AHashMap;
pub use comment::get_comment_completion_contents;
use itertools::Itertools;
use tombi_ast::{algo::ancestors_at_position, AstNode};
use tombi_config::TomlVersion;
use tombi_document_tree::TryIntoDocumentTree;
use tombi_extension::{CompletionContent, CompletionEdit, CompletionHint, CompletionKind};
use tombi_future::Boxable;
use tombi_rg_tree::TokenAtOffset;
use tombi_schema_store::{
    Accessor, CurrentSchema, ReferableValueSchemas, SchemaDefinitions, SchemaStore, SchemaUrl,
    ValueSchema,
};
use tombi_syntax::{SyntaxElement, SyntaxKind};

pub fn extract_keys_and_hint(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    toml_version: TomlVersion,
) -> Option<(Vec<tombi_document_tree::Key>, Option<CompletionHint>)> {
    let mut keys: Vec<tombi_document_tree::Key> = vec![];
    let mut completion_hint = None;

    match root.syntax().token_at_position(position) {
        TokenAtOffset::Single(token) if token.kind() == SyntaxKind::COMMENT => {
            return None;
        }
        TokenAtOffset::Between(token1, token2)
            if token1.kind() == SyntaxKind::COMMENT || token2.kind() == SyntaxKind::COMMENT =>
        {
            return None;
        }
        _ => {}
    }

    for node in ancestors_at_position(root.syntax(), position) {
        let ast_keys = if tombi_ast::Keys::cast(node.to_owned()).is_some() {
            if let Some(SyntaxElement::Token(last_token)) = node.last_child_or_token() {
                if last_token.kind() == SyntaxKind::DOT {
                    completion_hint = Some(CompletionHint::DotTrigger {
                        range: last_token.range(),
                    });
                }
            }
            continue;
        } else if let Some(kv) = tombi_ast::KeyValue::cast(node.to_owned()) {
            match (kv.keys(), kv.eq(), kv.value()) {
                (Some(_), Some(_), Some(_)) => {}
                (Some(_), Some(eq), None) => {
                    completion_hint = Some(CompletionHint::EqualTrigger { range: eq.range() });
                }
                (Some(keys), None, None) => {
                    if let Some(last_dot) = keys
                        .syntax()
                        .children_with_tokens()
                        .filter(|node_or_token| match node_or_token {
                            SyntaxElement::Token(token) => token.kind() == SyntaxKind::DOT,
                            _ => false,
                        })
                        .last()
                    {
                        completion_hint = Some(CompletionHint::DotTrigger {
                            range: last_dot.range(),
                        });
                    }
                }
                _ => {}
            }
            kv.keys()
        } else if let Some(table) = tombi_ast::Table::cast(node.to_owned()) {
            let (bracket_start_range, bracket_end_range) =
                match (table.bracket_start(), table.bracket_end()) {
                    (Some(bracket_start), Some(blacket_end)) => {
                        (bracket_start.range(), blacket_end.range())
                    }
                    _ => return None,
                };
            if position < bracket_start_range.start
                || (bracket_end_range.end <= position
                    && position.line == bracket_end_range.end.line)
            {
                return None;
            } else {
                if table.contains_header(position) {
                    completion_hint = Some(CompletionHint::InTableHeader);
                }
                table.header()
            }
        } else if let Some(array_of_table) = tombi_ast::ArrayOfTable::cast(node.to_owned()) {
            let (double_bracket_start_range, double_bracket_end_range) = {
                match (
                    array_of_table.double_bracket_start(),
                    array_of_table.double_bracket_end(),
                ) {
                    (Some(double_bracket_start), Some(double_bracket_end)) => {
                        (double_bracket_start.range(), double_bracket_end.range())
                    }
                    _ => return None,
                }
            };
            if position < double_bracket_start_range.start
                && (double_bracket_end_range.end <= position
                    && position.line == double_bracket_end_range.end.line)
            {
                return None;
            } else {
                if array_of_table.contains_header(position) {
                    completion_hint = Some(CompletionHint::InTableHeader);
                }
                array_of_table.header()
            }
        } else {
            continue;
        };

        let Some(ast_keys) = ast_keys else { continue };
        let mut new_keys = if ast_keys.range().contains(position) {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys
                .keys()
                .take_while(|key| key.token().unwrap().range().start <= position)
            {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key)) => new_keys.push(key),
                    _ => return None,
                }
            }
            new_keys
        } else {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys.keys() {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key)) => new_keys.push(key),
                    _ => return None,
                }
            }
            new_keys
        };
        new_keys.extend(keys);
        keys = new_keys;
    }

    Some((keys, completion_hint))
}

pub async fn find_completion_contents_with_tree(
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let current_schema = schema_context.root_schema.and_then(|document_schema| {
        document_schema
            .value_schema
            .as_ref()
            .map(|value_schema| CurrentSchema {
                value_schema: Cow::Borrowed(value_schema),
                schema_url: Cow::Borrowed(&document_schema.schema_url),
                definitions: Cow::Borrowed(&document_schema.definitions),
            })
    });

    document_tree
        .find_completion_contents(
            position,
            keys,
            &[],
            current_schema.as_ref(),
            schema_context,
            completion_hint,
        )
        .await
        .into_iter()
        .fold(AHashMap::new(), |mut acc: AHashMap<_, Vec<_>>, content| {
            acc.entry(content.label.clone()).or_default().push(content);
            acc
        })
        .into_iter()
        .filter_map(|(_, contents)| {
            contents
                .into_iter()
                .sorted_by(|a, b| a.priority.cmp(&b.priority))
                .next()
        })
        .collect()
}

pub trait FindCompletionContents {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>>;
}

pub trait CompletionCandidate {
    fn title<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    fn description<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    async fn detail(
        &self,
        schema_url: &SchemaUrl,
        definitions: &SchemaDefinitions,
        schema_store: &SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.title(schema_url, definitions, schema_store, completion_hint)
            .await
    }

    async fn documentation(
        &self,
        schema_url: &SchemaUrl,
        definitions: &SchemaDefinitions,
        schema_store: &SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.description(schema_url, definitions, schema_store, completion_hint)
            .await
    }
}

trait CompositeSchemaImpl {
    fn title(&self) -> Option<String>;
    fn description(&self) -> Option<String>;
    fn schemas(&self) -> &ReferableValueSchemas;
}

impl<T: CompositeSchemaImpl + Sync + Send> CompletionCandidate for T {
    fn title<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>> {
        async move {
            let mut candidates = ahash::AHashSet::new();
            {
                for referable_schema in self.schemas().write().await.iter_mut() {
                    if let Ok(Some(CurrentSchema {
                        value_schema,
                        schema_url,
                        definitions,
                    })) = referable_schema
                        .resolve(
                            Cow::Borrowed(schema_url),
                            Cow::Borrowed(definitions),
                            schema_store,
                        )
                        .await
                    {
                        if matches!(value_schema.as_ref(), ValueSchema::Null) {
                            continue;
                        }

                        if let Some(candidate) = CompletionCandidate::title(
                            value_schema.as_ref(),
                            &schema_url,
                            &definitions,
                            schema_store,
                            completion_hint,
                        )
                        .await
                        {
                            candidates.insert(candidate.to_string());
                        }
                    }
                }
            }
            if candidates.len() == 1 {
                return candidates.into_iter().next();
            }

            self.title().as_deref().map(|title| title.into())
        }
        .boxed()
    }

    fn description<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>> {
        async move {
            let mut candidates = ahash::AHashSet::new();
            {
                for referable_schema in self.schemas().write().await.iter_mut() {
                    if let Ok(Some(CurrentSchema {
                        value_schema,
                        schema_url,
                        definitions,
                    })) = referable_schema
                        .resolve(
                            Cow::Borrowed(schema_url),
                            Cow::Borrowed(definitions),
                            schema_store,
                        )
                        .await
                    {
                        if matches!(value_schema.as_ref(), ValueSchema::Null) {
                            continue;
                        }

                        if let Some(candidate) = CompletionCandidate::description(
                            value_schema.as_ref(),
                            &schema_url,
                            &definitions,
                            schema_store,
                            completion_hint,
                        )
                        .await
                        {
                            candidates.insert(candidate.to_string());
                        }
                    }
                }
            }

            if candidates.len() == 1 {
                return candidates.into_iter().next();
            }

            self.description()
                .as_deref()
                .map(|description| description.into())
        }
        .boxed()
    }
}

fn tombi_json_value_to_completion_default_item(
    value: &tombi_json::Value,
    position: tombi_text::Position,
    detail: Option<String>,
    documentation: Option<String>,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Option<CompletionContent> {
    let (kind, value) = match value {
        tombi_json::Value::String(value) => (CompletionKind::String, format!("\"{value}\"")),
        tombi_json::Value::Number(value) => {
            if value.is_i64() {
                (CompletionKind::Integer, value.to_string())
            } else {
                (CompletionKind::Float, value.to_string())
            }
        }
        tombi_json::Value::Bool(value) => (CompletionKind::Boolean, value.to_string()),
        _ => return None,
    };

    Some(CompletionContent::new_default_value(
        kind,
        value.to_string(),
        detail,
        documentation,
        CompletionEdit::new_literal(&value, position, completion_hint),
        schema_url,
        None,
    ))
}
