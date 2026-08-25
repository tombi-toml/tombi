mod comment;
mod completion_source;
mod schema_completion;
mod value;

use std::{borrow::Cow, ops::Deref, sync::Arc};

pub use comment::get_document_comment_directive_completion_contents;
use completion_source::CompletionSource;
use itertools::Itertools;
use tombi_ast_syntax::{AstNode, AstToken};
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{IntoDocumentTreeAndErrors, TryIntoDocumentTree};
use tombi_extension::CompletionContentPriority;
use tombi_extension::{
    CommaHint, CommentContext, CompletionContent, CompletionEdit, CompletionHint, CompletionKind,
};
use tombi_future::Boxable;
use tombi_schema_store::{
    Accessor, AccessorKeyKind, AllOfSchema, AnyOfSchema, CompositeSchema, CurrentSchema,
    KeyContext, OneOfSchema, SchemaDefinitions, SchemaStore, SchemaUri, SchemaView,
    get_schema_name,
};

use crate::schema_tooltip::{SchemaTooltip, SchemaTooltipContent};

pub fn get_comment_context(
    root: &tombi_ast_syntax::Root,
    position: tombi_text::Position,
) -> Option<CommentContext<tombi_ast_syntax::Comment>> {
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
            let comment: tombi_ast_syntax::Comment = leading_comment.into();
            if comment.syntax().range().contains(position)
                && comment.syntax().text()[1..].trim_start().starts_with(":")
            {
                return Some(CommentContext::DocumentDirective(comment));
            }
        }
    }

    if let Some(comment) = root.comment_at_position(position) {
        return _get_comment_context(comment);
    }

    None
}

fn _get_comment_context(
    comment: tombi_ast_syntax::Comment,
) -> Option<CommentContext<tombi_ast_syntax::Comment>> {
    if comment.get_tombi_value_directive().is_some() {
        Some(CommentContext::ValueDirective(comment))
    } else {
        Some(CommentContext::Normal(comment))
    }
}

pub fn extract_keys_and_hint(
    root: &tombi_ast_syntax::Root,
    position: tombi_text::Position,
    toml_version: TomlVersion,
    comment_context: Option<&CommentContext<tombi_ast_syntax::Comment>>,
) -> Option<(Vec<tombi_document_tree_syntax::Key>, Option<CompletionHint>)> {
    let mut keys: Vec<tombi_document_tree_syntax::Key> = vec![];
    let mut completion_hint = None;
    let is_tombi_value_comment_directive =
        matches!(comment_context, Some(CommentContext::ValueDirective(_)));

    for (index, node) in root.nodes_at_position(position).enumerate() {
        let ast_keys = match node {
            tombi_ast_syntax::TomlNode::Keys(keys) => {
                if let Some(last_token) = keys.last_dot() {
                    completion_hint = Some(CompletionHint::DotTrigger {
                        range: last_token.range(),
                        cleanup_range: tombi_text::Range {
                            start: last_token.range().start,
                            end: position,
                        },
                    });
                }
                continue;
            }
            tombi_ast_syntax::TomlNode::KeyValue(kv) => {
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
                        if let Some(last_dot) = kv_keys.last_dot() {
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
            }
            tombi_ast_syntax::TomlNode::Table(table) => {
                let bracket_start_range = table.bracket_start()?.range();
                let bracket_end_range = table.bracket_end().map(|bracket| bracket.range());
                if !is_tombi_value_comment_directive
                    && (position < bracket_start_range.start
                        || bracket_end_range.is_some_and(|end| {
                            end.end <= position && position.line == end.end.line
                        }))
                {
                    return None;
                } else {
                    if table.contains_header(position) {
                        completion_hint = Some(CompletionHint::InTableHeader);
                    }
                    table.header()
                }
            }
            tombi_ast_syntax::TomlNode::ArrayOfTable(array_of_table) => {
                let double_bracket_start_range = array_of_table.double_bracket_start()?.range();
                let double_bracket_end_range = array_of_table
                    .double_bracket_end()
                    .map(|bracket| bracket.range());
                if !is_tombi_value_comment_directive
                    && (position < double_bracket_start_range.start
                        || double_bracket_end_range.is_some_and(|end| {
                            end.end <= position && position.line == end.end.line
                        }))
                {
                    return None;
                } else {
                    if array_of_table.contains_header(position) {
                        completion_hint = Some(CompletionHint::InTableHeader);
                    }
                    array_of_table.header()
                }
            }
            _ => {
                if index == 0 {
                    let commas = root.adjacent_commas(position);
                    let leading_comma = commas.before.map(|range| CommaHint { range });
                    let trailing_comma = commas.after.map(|range| CommaHint { range });
                    if leading_comma.is_some() || trailing_comma.is_some() {
                        completion_hint = Some(CompletionHint::Comma {
                            leading_comma,
                            trailing_comma,
                        });
                    }
                }

                continue;
            }
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
    document_tree: &tombi_document_tree_syntax::DocumentTree,
    position: tombi_text::Position,
    keys: &[tombi_document_tree_syntax::Key],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let completion_items = match CompletionSource::new(
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
            if let Some((_, value)) =
                tombi_document_tree_syntax::dig_accessors(document_tree, &accessors)
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
    };
    dedup_completion_contents(completion_items)
}

pub trait FindCompletionContents {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Vec<CompletionContent>>;
}

fn dedup_completion_contents(completion_items: Vec<CompletionContent>) -> Vec<CompletionContent> {
    let mut deduped_items = tombi_hashmap::IndexMap::with_capacity(completion_items.len());

    for item in completion_items {
        match deduped_items.entry(completion_content_key(&item)) {
            tombi_hashmap::map::Entry::Occupied(mut entry) => {
                merge_completion_content(entry.get_mut(), item);
            }
            tombi_hashmap::map::Entry::Vacant(entry) => {
                entry.insert(item);
            }
        }
    }

    deduped_items.into_values().collect()
}

pub(super) fn dedup_composite_completion_contents(
    completion_items: Vec<(CompletionContent, Option<SchemaTooltip>)>,
) -> Vec<CompletionContent> {
    let mut deduped_items = tombi_hashmap::IndexMap::with_capacity(completion_items.len());

    for (mut item, tooltip) in completion_items {
        match deduped_items.entry(completion_content_key(&item)) {
            tombi_hashmap::map::Entry::Occupied(mut entry) => {
                let (existing, tooltips): &mut (CompletionContent, Vec<SchemaTooltip>) =
                    entry.get_mut();
                merge_completion_content(existing, item);
                if let Some(tooltip) = tooltip {
                    tooltips.push(tooltip);
                }
            }
            tombi_hashmap::map::Entry::Vacant(entry) => {
                let tooltips = tooltip.into_iter().collect();
                item.documentation = None;
                entry.insert((item, tooltips));
            }
        }
    }

    deduped_items
        .into_values()
        .map(|(mut item, tooltips)| {
            if let Some(tooltip) = SchemaTooltip::composite(tooltips) {
                item.documentation = Some(tooltip.to_string());
            }
            item
        })
        .collect()
}

pub(super) fn take_completion_schema_tooltip(
    item: &mut CompletionContent,
    current_schema: &CurrentSchema<'_>,
) -> Option<SchemaTooltip> {
    if item.schema_uri.as_ref() == Some(current_schema.schema_uri.as_ref()) {
        item.schema_uri = Some(
            tombi_extension::get_schema_link_uri(
                current_schema.schema_uri.as_ref(),
                current_schema.schema_view.range().start,
            )
            .into(),
        );
    }
    let mut markdown = item.documentation.take().unwrap_or_default();
    if let Some(schema_uri) = item.schema_uri.take()
        && let Some(schema_name) = get_schema_name(&schema_uri)
    {
        if !markdown.is_empty() && !markdown.ends_with("\n\n") {
            if markdown.ends_with('\n') {
                markdown.push('\n');
            } else {
                markdown.push_str("\n\n");
            }
        }
        markdown.push_str(&format!("Schema: [{schema_name}]({schema_uri})\n"));
    }

    (!markdown.is_empty()).then_some(SchemaTooltip::Markdown(markdown))
}

fn completion_content_key(item: &CompletionContent) -> (String, Option<CompletionKind>) {
    // Literal candidates with the same label are merged regardless of their literal kind.
    let non_literal_kind = (!item.kind.is_literal()).then_some(item.kind);
    (item.label.clone(), non_literal_kind)
}

fn merge_completion_content(existing: &mut CompletionContent, item: CompletionContent) {
    if item.priority < existing.priority {
        *existing = item;
    }
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
    keys: &[tombi_document_tree_syntax::Key],
    accessors: &[Accessor],
    current_schema: Option<&CurrentSchema<'_>>,
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    completion_hint: Option<CompletionHint>,
    base_completion_items: Vec<CompletionContent>,
    one_of_schema: Option<&OneOfSchema>,
    any_of_schema: Option<&AnyOfSchema>,
    all_of_schema: Option<&AllOfSchema>,
) -> Vec<CompletionContent> {
    if one_of_schema.is_none() && any_of_schema.is_none() && all_of_schema.is_none() {
        return base_completion_items;
    }

    let Some(current_schema) = current_schema else {
        return base_completion_items;
    };

    let instance_type = match current_schema.schema_view.as_ref() {
        SchemaView::Boolean(_) => tombi_schema_store::SchemaType::Boolean,
        SchemaView::Integer(_) => tombi_schema_store::SchemaType::Integer,
        SchemaView::Float(_) => tombi_schema_store::SchemaType::Number,
        SchemaView::String(_)
        | SchemaView::OffsetDateTime(_)
        | SchemaView::LocalDateTime(_)
        | SchemaView::LocalDate(_)
        | SchemaView::LocalTime(_) => tombi_schema_store::SchemaType::String,
        SchemaView::Array(_) => tombi_schema_store::SchemaType::Array,
        SchemaView::Table(_) => tombi_schema_store::SchemaType::Object,
        SchemaView::Null
        | SchemaView::Anything(_)
        | SchemaView::Nothing(_)
        | SchemaView::OneOf(_)
        | SchemaView::AnyOf(_)
        | SchemaView::AllOf(_) => return base_completion_items,
    };
    let instance_completion = schema_completion::InstanceSchemaCompletion(instance_type);

    let mut adjacent_completion_items = Vec::new();

    if let Some(one_of_schema) = one_of_schema {
        adjacent_completion_items.extend(
            value::find_one_of_completion_items(
                &instance_completion,
                position,
                keys,
                accessors,
                one_of_schema,
                &CurrentSchema {
                    schema_view: Arc::new(SchemaView::OneOf(one_of_schema.clone())),
                    semantic_schema: None,
                    schema_uri: current_schema.schema_uri.clone(),
                    definitions: current_schema.definitions.clone(),
                    strict: current_schema.strict,
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
                &instance_completion,
                position,
                keys,
                accessors,
                any_of_schema,
                &CurrentSchema {
                    schema_view: Arc::new(SchemaView::AnyOf(any_of_schema.clone())),
                    semantic_schema: None,
                    schema_uri: current_schema.schema_uri.clone(),
                    definitions: current_schema.definitions.clone(),
                    strict: current_schema.strict,
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
                &instance_completion,
                position,
                keys,
                accessors,
                all_of_schema,
                &CurrentSchema {
                    schema_view: Arc::new(SchemaView::AllOf(all_of_schema.clone())),
                    semantic_schema: None,
                    schema_uri: current_schema.schema_uri.clone(),
                    definitions: current_schema.definitions.clone(),
                    strict: current_schema.strict,
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
    dedup_composite_completion_contents(
        completion_items
            .into_iter()
            .map(|mut item| {
                let tooltip = take_completion_schema_tooltip(&mut item, current_schema);
                (item, tooltip)
            })
            .collect(),
    )
}

pub trait CompletionCandidate {
    fn title<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    fn description<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    async fn detail(
        &self,
        schema_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.title(
            schema_uri,
            definitions,
            strict,
            schema_store,
            completion_hint,
        )
        .await
    }

    async fn documentation(
        &self,
        schema_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.description(
            schema_uri,
            definitions,
            strict,
            schema_store,
            completion_hint,
        )
        .await
    }
}

fn composite_title<'a: 'b, 'b, T: CompositeSchema + Sync + Send>(
    composite_schema: &'a T,
    schema_uri: &'a SchemaUri,
    definitions: &'a SchemaDefinitions,
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_store: &'a SchemaStore,
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Option<String>> {
    async move {
        let mut candidates = tombi_hashmap::IndexSet::new();
        let schema_visits = tombi_schema_store::SchemaVisits::default();

        if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            composite_schema.schemas(),
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_store,
            &schema_visits,
            &[],
        )
        .await
        {
            for current_schema in &resolved_schemas {
                if matches!(current_schema.schema_view.as_ref(), SchemaView::Null) {
                    continue;
                }

                if let Some(candidate) = CompletionCandidate::title(
                    current_schema.schema_view.as_ref(),
                    &current_schema.schema_uri,
                    &current_schema.definitions,
                    current_schema.strict,
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
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_store: &'a SchemaStore,
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Option<String>> {
    async move {
        let mut contents = Vec::new();
        let schema_visits = tombi_schema_store::SchemaVisits::default();

        if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            composite_schema.schemas(),
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_store,
            &schema_visits,
            &[],
        )
        .await
        {
            for current_schema in &resolved_schemas {
                if matches!(current_schema.schema_view.as_ref(), SchemaView::Null) {
                    continue;
                }

                let title = CompletionCandidate::title(
                    current_schema.schema_view.as_ref(),
                    &current_schema.schema_uri,
                    &current_schema.definitions,
                    current_schema.strict,
                    schema_store,
                    completion_hint,
                )
                .await;
                let description = CompletionCandidate::description(
                    current_schema.schema_view.as_ref(),
                    &current_schema.schema_uri,
                    &current_schema.definitions,
                    current_schema.strict,
                    schema_store,
                    completion_hint,
                )
                .await;
                contents.push(SchemaTooltip::Content(SchemaTooltipContent {
                    title,
                    description,
                    value_type: current_schema.schema_view.value_type().await.to_string(),
                    constraints: None,
                    schema: None,
                }));
            }
        }

        if let Some(tooltip) = SchemaTooltip::composite(contents) {
            return Some(tooltip.to_string());
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
                strict: Option<tombi_schema_type::BoolDefaultTrue>,
                schema_store: &'a SchemaStore,
                completion_hint: Option<CompletionHint>,
            ) -> tombi_future::BoxFuture<'b, Option<String>> {
                composite_title(
                    self,
                    schema_uri,
                    definitions,
                    strict,
                    schema_store,
                    completion_hint,
                )
            }

            fn description<'a: 'b, 'b>(
                &'a self,
                schema_uri: &'a SchemaUri,
                definitions: &'a SchemaDefinitions,
                strict: Option<tombi_schema_type::BoolDefaultTrue>,
                schema_store: &'a SchemaStore,
                completion_hint: Option<CompletionHint>,
            ) -> tombi_future::BoxFuture<'b, Option<String>> {
                composite_description(
                    self,
                    schema_uri,
                    definitions,
                    strict,
                    schema_store,
                    completion_hint,
                )
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

fn tombi_json_value_to_completion_enum_item(
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
    Some(CompletionContent::new_enum_value(
        label,
        detail,
        documentation,
        edit,
        schema_uri,
        None,
    ))
}

pub async fn get_completion_keys_with_context(
    root: &tombi_ast_syntax::Root,
    position: tombi_text::Position,
    toml_version: tombi_config::TomlVersion,
) -> Option<(Vec<tombi_document_tree_syntax::Key>, Vec<KeyContext>)> {
    let mut keys_vec = vec![];
    let mut key_contexts = vec![];

    for node in root.nodes_at_position(position) {
        if let tombi_ast_syntax::TomlNode::KeyValue(kv) = node {
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
        } else if let tombi_ast_syntax::TomlNode::Table(table) = node {
            if let Some(header) = table.header() {
                for key in header.keys_rev() {
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
        } else if let tombi_ast_syntax::TomlNode::ArrayOfTable(array_of_table) = node
            && let Some(header) = array_of_table.header()
        {
            for key in header.keys_rev() {
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
