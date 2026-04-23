mod comment;
mod completion_source;
mod schema_completion;
mod value;

use std::{borrow::Cow, ops::Deref, sync::Arc};

pub use comment::get_document_comment_directive_completion_contents;
use completion_source::CompletionSource;
use itertools::Itertools;
use tombi_ast::{AstNode, AstToken, algo::ancestors_at_position};
use tombi_config::TomlVersion;
use tombi_document_tree::{IntoDocumentTreeAndErrors, TryIntoDocumentTree};
use tombi_extension::CompletionContentPriority;
use tombi_extension::{
    CommaHint, CommentContext, CompletionContent, CompletionEdit, CompletionHint,
};
use tombi_future::Boxable;
use tombi_rg_tree::{NodeOrToken, TokenAtOffset};
use tombi_schema_store::{
    Accessor, AccessorKeyKind, AllOfSchema, AnyOfSchema, CompositeSchema, CurrentSchema,
    KeyContext, OneOfSchema, SchemaDefinitions, SchemaStore, SchemaUri, ValueSchema,
};
use tombi_syntax::{Direction, SyntaxElement, SyntaxKind, SyntaxNode};

pub fn get_comment_context(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<CommentContext> {
    if let Some(comment_group) = root.dangling_comment_groups().next() {
        for comment in comment_group.comments() {
            if comment.syntax().range().contains(position)
                && comment.syntax().text()[1..].trim_start().starts_with(":")
            {
                return Some(CommentContext::DocumentDirective(comment.into()));
            }
        }
    }

    if let Some(leading_comments) = root
        .key_values()
        .next()
        .map(|kv| kv.leading_comments().collect_vec())
        .or_else(|| {
            root.table_or_array_of_tables()
                .next()
                .map(|ta| ta.leading_comments().collect_vec())
        })
    {
        for leading_comment in leading_comments {
            let comment: tombi_ast::Comment = leading_comment.into();
            if comment.syntax().range().contains(position)
                && comment.syntax().text()[1..].trim_start().starts_with(":")
            {
                return Some(CommentContext::DocumentDirective(comment));
            }
        }
    }

    match root.syntax().token_at_position(position) {
        TokenAtOffset::Single(token) if token.kind() == SyntaxKind::COMMENT => {
            if let Some(comment) = tombi_ast::Comment::cast(token) {
                return _get_comment_context(comment);
            }
        }
        TokenAtOffset::Between(token1, token2)
            if token1.kind() == SyntaxKind::COMMENT || token2.kind() == SyntaxKind::COMMENT =>
        {
            if let Some(comment) = tombi_ast::Comment::cast(token1) {
                return _get_comment_context(comment);
            }
            if let Some(comment) = tombi_ast::Comment::cast(token2) {
                return _get_comment_context(comment);
            }
        }
        _ => {}
    }

    None
}

fn _get_comment_context(comment: tombi_ast::Comment) -> Option<CommentContext> {
    if comment.get_tombi_value_directive().is_some() {
        Some(CommentContext::ValueDirective(comment))
    } else {
        Some(CommentContext::Normal(comment))
    }
}

pub fn extract_keys_and_hint(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    toml_version: TomlVersion,
    comment_context: Option<&CommentContext>,
) -> Option<(Vec<tombi_document_tree::Key>, Option<CompletionHint>)> {
    let mut keys: Vec<tombi_document_tree::Key> = vec![];
    let mut completion_hint = None;
    let is_tombi_value_comment_directive =
        matches!(comment_context, Some(CommentContext::ValueDirective(_)));

    for (index, node) in ancestors_at_position(root.syntax(), position).enumerate() {
        let ast_keys = if tombi_ast::Keys::cast(node.to_owned()).is_some() {
            if let Some(SyntaxElement::Token(last_token)) = node.last_child_or_token()
                && last_token.kind() == SyntaxKind::DOT
            {
                completion_hint = Some(CompletionHint::DotTrigger {
                    range: last_token.range(),
                    cleanup_range: tombi_text::Range {
                        start: last_token.range().start,
                        end: position,
                    },
                });
            }
            continue;
        } else if let Some(kv) = tombi_ast::KeyValue::cast(node.to_owned()) {
            let Some(kv_keys) = kv.keys() else { continue };
            if comment_context.is_none() && kv_keys.range().start > position {
                continue;
            }
            match (kv.eq(), kv.value()) {
                (Some(_), Some(_)) => {}
                (Some(eq), None) => {
                    completion_hint = Some(CompletionHint::EqualTrigger {
                        range: eq.range(),
                        cleanup_range: tombi_text::Range {
                            start: kv_keys.range().end,
                            end: position,
                        },
                    });
                }
                (None, None) => {
                    if let Some(last_dot) = kv_keys
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
                            cleanup_range: tombi_text::Range {
                                start: last_dot.range().start,
                                end: position,
                            },
                        });
                    }
                }
                _ => {}
            }
            Some(kv_keys)
        } else if let Some(table) = tombi_ast::Table::cast(node.to_owned()) {
            let (bracket_start_range, bracket_end_range) =
                match (table.bracket_start(), table.bracket_end()) {
                    (Some(bracket_start), Some(blacket_end)) => {
                        (bracket_start.range(), blacket_end.range())
                    }
                    _ => return None,
                };
            if !is_tombi_value_comment_directive
                && (position < bracket_start_range.start
                    || (bracket_end_range.end <= position
                        && position.line == bracket_end_range.end.line))
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
            if !is_tombi_value_comment_directive
                && (position < double_bracket_start_range.start
                    && (double_bracket_end_range.end <= position
                        && position.line == double_bracket_end_range.end.line))
            {
                return None;
            } else {
                if array_of_table.contains_header(position) {
                    completion_hint = Some(CompletionHint::InTableHeader);
                }
                array_of_table.header()
            }
        } else {
            if index == 0 {
                let leading_comma = get_leading_comma(&node, position);
                let trailing_comma = get_trailing_comma(&node, position);
                if leading_comma.is_some() || trailing_comma.is_some() {
                    completion_hint = Some(CompletionHint::Comma {
                        leading_comma,
                        trailing_comma,
                    });
                }
            }

            continue;
        };

        let Some(ast_keys) = ast_keys else { continue };
        let mut new_keys = if ast_keys.range().contains(position) {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys
                .keys()
                .take_while(|key| key.token().unwrap().range().start <= position)
            {
                let document_tree_key = key.into_document_tree_and_errors(toml_version).tree;
                if let Some(document_tree_key) = document_tree_key {
                    new_keys.push(document_tree_key);
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

pub async fn find_completion_contents(
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    match CompletionSource::new(
        document_tree,
        position,
        keys,
        schema_context,
        completion_hint,
    )
    .await
    {
        Some(CompletionSource::Root {
            remaining_keys,
            accessors,
            current_schema,
        }) => {
            document_tree
                .deref()
                .find_completion_contents(
                    position,
                    remaining_keys,
                    &accessors,
                    current_schema.as_ref(),
                    schema_context,
                    completion_hint,
                )
                .await
        }
        Some(CompletionSource::Value {
            remaining_keys,
            accessors,
            current_schema,
        }) => {
            if let Some((_, value)) = tombi_document_tree::dig_accessors(document_tree, &accessors)
            {
                value
                    .find_completion_contents(
                        position,
                        remaining_keys,
                        &accessors,
                        current_schema.as_ref(),
                        schema_context,
                        completion_hint,
                    )
                    .await
            } else {
                Vec::new()
            }
        }
        Some(CompletionSource::Schema {
            remaining_keys,
            accessors,
            current_schema,
        }) => {
            schema_completion::SchemaCompletion
                .find_completion_contents(
                    position,
                    remaining_keys,
                    &accessors,
                    Some(&current_schema),
                    schema_context,
                    completion_hint,
                )
                .await
        }
        None => Vec::new(),
    }
    .into_iter()
    .fold(
        tombi_hashmap::IndexMap::new(),
        |mut acc: tombi_hashmap::IndexMap<_, Vec<_>>, content| {
            acc.entry(content.label.clone()).or_default().push(content);
            acc
        },
    )
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

fn dedup_completion_contents(completion_items: Vec<CompletionContent>) -> Vec<CompletionContent> {
    let mut deduped_items: tombi_hashmap::IndexMap<String, CompletionContent> =
        tombi_hashmap::IndexMap::new();

    for item in completion_items {
        match deduped_items.entry(item.label.clone()) {
            tombi_hashmap::map::Entry::Occupied(mut entry) => {
                if item.priority < entry.get().priority {
                    entry.insert(item);
                }
            }
            tombi_hashmap::map::Entry::Vacant(entry) => {
                entry.insert(item);
            }
        }
    }

    deduped_items.into_values().collect()
}

fn is_generic_literal_type_hint(completion_item: &CompletionContent) -> bool {
    matches!(
        completion_item.priority,
        CompletionContentPriority::TypeHint
            | CompletionContentPriority::TypeHintTrue
            | CompletionContentPriority::TypeHintFalse
    ) && completion_item.label != "\"\""
        && completion_item.label != "''"
}

pub(super) async fn merge_adjacent_schema_completion_items(
    position: tombi_text::Position,
    keys: &[tombi_document_tree::Key],
    accessors: &[Accessor],
    current_schema: Option<&CurrentSchema<'_>>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    completion_hint: Option<CompletionHint>,
    base_completion_items: Vec<CompletionContent>,
    one_of_schema: Option<&OneOfSchema>,
    any_of_schema: Option<&AnyOfSchema>,
    all_of_schema: Option<&AllOfSchema>,
) -> Vec<CompletionContent> {
    let Some(current_schema) = current_schema else {
        return base_completion_items;
    };

    let mut adjacent_completion_items = Vec::new();

    if let Some(one_of_schema) = one_of_schema {
        adjacent_completion_items.extend(
            value::find_one_of_completion_items(
                &schema_completion::SchemaCompletion,
                position,
                keys,
                accessors,
                one_of_schema,
                &CurrentSchema {
                    value_schema: Arc::new(ValueSchema::OneOf(one_of_schema.clone())),
                    schema_uri: current_schema.schema_uri.clone(),
                    definitions: current_schema.definitions.clone(),
                },
                schema_context,
                completion_hint,
            )
            .await,
        );
    }
    if let Some(any_of_schema) = any_of_schema {
        adjacent_completion_items.extend(
            value::find_any_of_completion_items(
                &schema_completion::SchemaCompletion,
                position,
                keys,
                accessors,
                any_of_schema,
                &CurrentSchema {
                    value_schema: Arc::new(ValueSchema::AnyOf(any_of_schema.clone())),
                    schema_uri: current_schema.schema_uri.clone(),
                    definitions: current_schema.definitions.clone(),
                },
                schema_context,
                completion_hint,
            )
            .await,
        );
    }
    if let Some(all_of_schema) = all_of_schema {
        adjacent_completion_items.extend(
            value::find_all_of_completion_items(
                &schema_completion::SchemaCompletion,
                position,
                keys,
                accessors,
                all_of_schema,
                &CurrentSchema {
                    value_schema: Arc::new(ValueSchema::AllOf(all_of_schema.clone())),
                    schema_uri: current_schema.schema_uri.clone(),
                    definitions: current_schema.definitions.clone(),
                },
                schema_context,
                completion_hint,
            )
            .await,
        );
    }

    let has_concrete_adjacent_values = adjacent_completion_items.iter().any(|completion_item| {
        !matches!(
            completion_item.priority,
            CompletionContentPriority::TypeHint
                | CompletionContentPriority::TypeHintTrue
                | CompletionContentPriority::TypeHintFalse
        )
    });

    let mut completion_items = adjacent_completion_items;
    completion_items.extend(base_completion_items.into_iter().filter(|completion_item| {
        !has_concrete_adjacent_values || !is_generic_literal_type_hint(completion_item)
    }));
    dedup_completion_contents(completion_items)
}

pub trait CompletionCandidate {
    fn title<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    fn description<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    async fn detail(
        &self,
        schema_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        schema_store: &SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.title(schema_uri, definitions, schema_store, completion_hint)
            .await
    }

    async fn documentation(
        &self,
        schema_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        schema_store: &SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.description(schema_uri, definitions, schema_store, completion_hint)
            .await
    }
}

fn composite_title<'a: 'b, 'b, T: CompositeSchema + Sync + Send>(
    composite_schema: &'a T,
    schema_uri: &'a SchemaUri,
    definitions: &'a SchemaDefinitions,
    schema_store: &'a SchemaStore,
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Option<String>> {
    async move {
        let mut candidates = tombi_hashmap::HashSet::new();
        let schema_visits = tombi_schema_store::SchemaVisits::default();

        if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            composite_schema.schemas(),
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            schema_store,
            &schema_visits,
            &[],
        )
        .await
        {
            for current_schema in &resolved_schemas {
                if matches!(current_schema.value_schema.as_ref(), ValueSchema::Null) {
                    continue;
                }

                if let Some(candidate) = CompletionCandidate::title(
                    current_schema.value_schema.as_ref(),
                    &current_schema.schema_uri,
                    &current_schema.definitions,
                    schema_store,
                    completion_hint,
                )
                .await
                {
                    candidates.insert(candidate.to_string());
                }
            }
        }

        if candidates.len() == 1 {
            return candidates.into_iter().next();
        }

        composite_schema
            .title()
            .as_deref()
            .map(|title| title.into())
    }
    .boxed()
}

fn composite_description<'a: 'b, 'b, T: CompositeSchema + Sync + Send>(
    composite_schema: &'a T,
    schema_uri: &'a SchemaUri,
    definitions: &'a SchemaDefinitions,
    schema_store: &'a SchemaStore,
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Option<String>> {
    async move {
        let mut candidates = tombi_hashmap::HashSet::new();
        let schema_visits = tombi_schema_store::SchemaVisits::default();

        if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            composite_schema.schemas(),
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            schema_store,
            &schema_visits,
            &[],
        )
        .await
        {
            for current_schema in &resolved_schemas {
                if matches!(current_schema.value_schema.as_ref(), ValueSchema::Null) {
                    continue;
                }

                if let Some(candidate) = CompletionCandidate::description(
                    current_schema.value_schema.as_ref(),
                    &current_schema.schema_uri,
                    &current_schema.definitions,
                    schema_store,
                    completion_hint,
                )
                .await
                {
                    candidates.insert(candidate.to_string());
                }
            }
        }

        if candidates.len() == 1 {
            return candidates.into_iter().next();
        }

        composite_schema
            .description()
            .as_deref()
            .map(|description| description.into())
    }
    .boxed()
}

macro_rules! impl_composite_completion_candidate {
    ($ty:path) => {
        impl CompletionCandidate for $ty {
            fn title<'a: 'b, 'b>(
                &'a self,
                schema_uri: &'a SchemaUri,
                definitions: &'a SchemaDefinitions,
                schema_store: &'a SchemaStore,
                completion_hint: Option<CompletionHint>,
            ) -> tombi_future::BoxFuture<'b, Option<String>> {
                composite_title(self, schema_uri, definitions, schema_store, completion_hint)
            }

            fn description<'a: 'b, 'b>(
                &'a self,
                schema_uri: &'a SchemaUri,
                definitions: &'a SchemaDefinitions,
                schema_store: &'a SchemaStore,
                completion_hint: Option<CompletionHint>,
            ) -> tombi_future::BoxFuture<'b, Option<String>> {
                composite_description(self, schema_uri, definitions, schema_store, completion_hint)
            }
        }
    };
}

impl_composite_completion_candidate!(tombi_schema_store::OneOfSchema);
impl_composite_completion_candidate!(tombi_schema_store::AnyOfSchema);
impl_composite_completion_candidate!(tombi_schema_store::AllOfSchema);

fn tombi_json_value_to_completion_default_item(
    value: &tombi_json::Value,
    position: tombi_text::Position,
    detail: Option<String>,
    documentation: Option<String>,
    schema_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Option<CompletionContent> {
    if !matches!(
        value,
        tombi_json::Value::String(_) | tombi_json::Value::Number(_) | tombi_json::Value::Bool(_)
    ) {
        return None;
    }

    let label = value.to_string();
    let edit = CompletionEdit::new_literal(&label, position, completion_hint);

    Some(CompletionContent::new_default_value(
        label,
        detail,
        documentation,
        edit,
        schema_uri,
        None,
    ))
}

fn tombi_json_value_to_completion_example_item(
    value: &tombi_json::Value,
    position: tombi_text::Position,
    detail: Option<String>,
    documentation: Option<String>,
    schema_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Option<CompletionContent> {
    if !matches!(
        value,
        tombi_json::Value::String(_) | tombi_json::Value::Number(_) | tombi_json::Value::Bool(_)
    ) {
        return None;
    }

    let label = value.to_string();
    let edit = CompletionEdit::new_literal(&label, position, completion_hint);

    Some(CompletionContent::new_example_value(
        label,
        detail,
        documentation,
        edit,
        schema_uri,
        None,
    ))
}

fn get_leading_comma(node: &SyntaxNode, position: tombi_text::Position) -> Option<CommaHint> {
    if let Some(child) = node.last_child()
        && child.kind() == SyntaxKind::COMMA
    {
        return Some(CommaHint {
            range: child.range(),
        });
    }
    if let Some(sibling) = node
        .siblings_with_tokens(Direction::Prev)
        .find(|node_or_token| !node_or_token.range().contains(position))
        && sibling.kind() == SyntaxKind::COMMA
    {
        return Some(CommaHint {
            range: sibling.range(),
        });
    }
    None
}

fn get_trailing_comma(node: &SyntaxNode, position: tombi_text::Position) -> Option<CommaHint> {
    if let Some(sibling) = node.siblings_with_tokens(Direction::Next).next() {
        match sibling.kind() {
            SyntaxKind::COMMA => {
                // Case like:
                //
                // ```toml
                // key = ["value" █,]
                // ```
                return Some(CommaHint {
                    range: sibling.range(),
                });
            }
            SyntaxKind::INVALID_TOKEN => {
                // Case like:
                //
                // ```toml
                // key = [█, "value"]
                // ```
                if let NodeOrToken::Node(node) = sibling
                    && let Some(SyntaxElement::Token(token)) = node.first_child_or_token()
                    && token.kind() == SyntaxKind::COMMA
                {
                    return Some(CommaHint {
                        range: token.range(),
                    });
                }
            }
            SyntaxKind::ARRAY => {
                // Case like:
                //
                // ```toml
                // [dependency-groups]
                // dev = [  █   , "pytest"]
                // ```

                if let NodeOrToken::Node(node) = sibling
                    && let Some(next_node_or_token) = node
                        .children_with_tokens()
                        .skip_while(|sibling| !sibling.range().contains(position))
                        .nth(1)
                {
                    match next_node_or_token.kind() {
                        SyntaxKind::COMMA => {
                            return Some(CommaHint {
                                range: next_node_or_token.range(),
                            });
                        }
                        SyntaxKind::INVALID_TOKEN => {
                            if let NodeOrToken::Node(node) = next_node_or_token
                                && let Some(SyntaxElement::Token(token)) =
                                    node.first_child_or_token()
                                && token.kind() == SyntaxKind::COMMA
                            {
                                return Some(CommaHint {
                                    range: token.range(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    None
}

pub async fn get_completion_keys_with_context(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    toml_version: tombi_config::TomlVersion,
) -> Option<(Vec<tombi_document_tree::Key>, Vec<KeyContext>)> {
    let mut keys_vec = vec![];
    let mut key_contexts = vec![];

    for node in ancestors_at_position(root.syntax(), position) {
        if let Some(kv) = tombi_ast::KeyValue::cast(node.to_owned()) {
            let keys = kv.keys()?;
            let keys = if keys.range().contains(position) {
                keys.keys()
                    .take_while(|key| key.token().unwrap().range().start <= position)
                    .collect_vec()
            } else {
                keys.keys().collect_vec()
            };
            for (i, key) in keys.into_iter().rev().enumerate() {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key_dt)) => {
                        let kind = if i == 0 {
                            AccessorKeyKind::KeyValue
                        } else {
                            AccessorKeyKind::Dotted
                        };
                        keys_vec.push(key_dt.clone());
                        key_contexts.push(KeyContext {
                            kind,
                            range: key_dt.range(),
                        });
                    }
                    _ => return None,
                }
            }
        } else if let Some(table) = tombi_ast::Table::cast(node.to_owned()) {
            if let Some(header) = table.header() {
                for key in header.keys().rev() {
                    match key.try_into_document_tree(toml_version) {
                        Ok(Some(key_dt)) => {
                            keys_vec.push(key_dt.clone());
                            key_contexts.push(KeyContext {
                                kind: AccessorKeyKind::Header,
                                range: key_dt.range(),
                            });
                        }
                        _ => return None,
                    }
                }
            }
        } else if let Some(array_of_table) = tombi_ast::ArrayOfTable::cast(node.to_owned())
            && let Some(header) = array_of_table.header()
        {
            for key in header.keys().rev() {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key_dt)) => {
                        keys_vec.push(key_dt.clone());
                        key_contexts.push(KeyContext {
                            kind: AccessorKeyKind::Header,
                            range: key_dt.range(),
                        });
                    }
                    _ => return None,
                }
            }
        }
    }

    if keys_vec.is_empty() {
        return None;
    }
    Some((
        keys_vec.into_iter().rev().collect(),
        key_contexts.into_iter().rev().collect(),
    ))
}
