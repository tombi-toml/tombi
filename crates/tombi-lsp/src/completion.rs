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

pub fn get_comment_context<'t>(
    root: &tombi_ast_syntax::Root<'t>,
    offset: tombi_text::Offset,
) -> Option<CommentContext<tombi_ast_syntax::Comment<'t>>> {
    if let Some(comment_group) = root.dangling_comment_groups().next() {
        for comment in comment_group.comments() {
            if comment.syntax().span().contains_inclusive(offset)
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
            let comment: tombi_ast_syntax::Comment<'_> = leading_comment.into();
            if comment.syntax().span().contains_inclusive(offset)
                && comment.syntax().text()[1..].trim_start().starts_with(":")
            {
                return Some(CommentContext::DocumentDirective(comment));
            }
        }
    }

    if let Some(comment) = root.comment_at_offset(offset) {
        return _get_comment_context(comment);
    }

    None
}

fn _get_comment_context(
    comment: tombi_ast_syntax::Comment<'_>,
) -> Option<CommentContext<tombi_ast_syntax::Comment<'_>>> {
    if comment.get_tombi_value_directive().is_some() {
        Some(CommentContext::ValueDirective(comment))
    } else {
        Some(CommentContext::Normal(comment))
    }
}

pub fn extract_keys_and_hint<'t>(
    root: &tombi_ast_syntax::Root<'t>,
    decoded: &'t tombi_ast_syntax::DecodedTextResolver,
    offset: tombi_text::Offset,
    toml_version: TomlVersion,
    comment_context: Option<&CommentContext<tombi_ast_syntax::Comment<'_>>>,
) -> Option<(
    Vec<tombi_document_tree_syntax::Key<'t>>,
    Option<CompletionHint>,
)> {
    let mut keys: Vec<tombi_document_tree_syntax::Key<'t>> = vec![];
    let mut completion_hint = None;
    let is_tombi_value_comment_directive =
        matches!(comment_context, Some(CommentContext::ValueDirective(_)));

    let cursor = crate::CursorPosition::new(offset, root.syntax().line_index());

    for (index, node) in root.nodes_at_offset(offset).enumerate() {
        let ast_keys = match node {
            tombi_ast_syntax::TomlNode::Keys(keys) => {
                if let Some(last_token) = keys.last_dot() {
                    completion_hint = Some(CompletionHint::DotTrigger {
                        span: last_token.span(),
                        cleanup_span: tombi_text::Span::new(last_token.span().start, offset),
                    });
                }
                continue;
            }
            tombi_ast_syntax::TomlNode::KeyValue(kv) => {
                let Some(kv_keys) = kv.keys() else { continue };
                if comment_context.is_none() && kv_keys.span().start > offset {
                    continue;
                }
                match (kv.eq(), kv.value()) {
                    (Some(_), Some(_)) => {}
                    (Some(eq), None) => {
                        completion_hint = Some(CompletionHint::EqualTrigger {
                            span: eq.span(),
                            cleanup_span: tombi_text::Span::new(kv_keys.span().end, offset),
                        });
                    }
                    (None, None) => {
                        if let Some(last_dot) = kv_keys.last_dot() {
                            completion_hint = Some(CompletionHint::DotTrigger {
                                span: last_dot.span(),
                                cleanup_span: tombi_text::Span::new(last_dot.span().start, offset),
                            });
                        }
                    }
                    _ => {}
                }
                Some(kv_keys)
            }
            tombi_ast_syntax::TomlNode::Table(table) => {
                let bracket_start_span = table.bracket_start()?.span();
                let bracket_end_span = table.bracket_end().map(|bracket| bracket.span());
                if !is_tombi_value_comment_directive
                    && (offset < bracket_start_span.start
                        || bracket_end_span
                            .is_some_and(|end| end.end <= offset && cursor.is_on_line(end.end)))
                {
                    return None;
                } else {
                    if table.contains_header(offset) {
                        completion_hint = Some(CompletionHint::InTableHeader);
                    }
                    table.header()
                }
            }
            tombi_ast_syntax::TomlNode::ArrayOfTable(array_of_table) => {
                let double_bracket_start_span = array_of_table.double_bracket_start()?.span();
                let double_bracket_end_span = array_of_table
                    .double_bracket_end()
                    .map(|bracket| bracket.span());
                if !is_tombi_value_comment_directive
                    && (offset < double_bracket_start_span.start
                        || double_bracket_end_span
                            .is_some_and(|end| end.end <= offset && cursor.is_on_line(end.end)))
                {
                    return None;
                } else {
                    if array_of_table.contains_header(offset) {
                        completion_hint = Some(CompletionHint::InTableHeader);
                    }
                    array_of_table.header()
                }
            }
            _ => {
                if index == 0 {
                    let commas = root.adjacent_commas(offset);
                    let leading_comma = commas.before.map(|span| CommaHint { span });
                    let trailing_comma = commas.after.map(|span| CommaHint { span });
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
        let mut new_keys = if ast_keys.span().contains_inclusive(offset) {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys
                .keys()
                .take_while(|key| key.token().unwrap().span().start <= offset)
            {
                let document_tree_key = key
                    .into_document_tree_and_errors(toml_version, decoded)
                    .tree;
                if let Some(document_tree_key) = document_tree_key {
                    new_keys.push(document_tree_key);
                }
            }
            new_keys
        } else {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys.keys() {
                match key.try_into_document_tree(toml_version, decoded) {
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
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cursor: crate::CursorPosition<'_>,
    keys: &[tombi_document_tree_syntax::Key<'_>],
    schema_context: &tombi_schema_store::SchemaContext<'_>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let offset = cursor.offset();
    let completion_items =
        match CompletionSource::new(document_tree, offset, keys, schema_context, completion_hint)
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
                        cursor,
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
                            cursor,
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
                        cursor,
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
        cursor: crate::CursorPosition<'a>,
        keys: &'a [tombi_document_tree_syntax::Key<'_>],
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
    if item.schema_base_uri.as_ref() == Some(current_schema.schema_base_uri.as_ref()) {
        item.schema_base_uri = Some(crate::hover::schema_view_link_uri(current_schema));
    }
    let mut markdown = item.documentation.take().unwrap_or_default();
    if let Some(schema_base_uri) = item.schema_base_uri.take()
        && let Some(schema_name) = get_schema_name(&schema_base_uri)
    {
        if !markdown.is_empty() && !markdown.ends_with("\n\n") {
            if markdown.ends_with('\n') {
                markdown.push('\n');
            } else {
                markdown.push_str("\n\n");
            }
        }
        markdown.push_str(&format!("Schema: [{schema_name}]({schema_base_uri})\n"));
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
    cursor: crate::CursorPosition<'_>,
    keys: &[tombi_document_tree_syntax::Key<'_>],
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
                cursor,
                keys,
                accessors,
                one_of_schema,
                &CurrentSchema {
                    schema_view: Arc::new(SchemaView::OneOf(one_of_schema.clone())),
                    semantic_schema: None,
                    schema_uri: current_schema.schema_uri.clone(),
                    schema_base_uri: current_schema.schema_base_uri.clone(),
                    schema_document_uri: current_schema.schema_document_uri.clone(),
                    line_index: current_schema.line_index.clone(),
                    definitions: current_schema.definitions.clone(),
                    strict: current_schema.strict,
                    dynamic_scope: current_schema.dynamic_scope.clone(),
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
                cursor,
                keys,
                accessors,
                any_of_schema,
                &CurrentSchema {
                    schema_view: Arc::new(SchemaView::AnyOf(any_of_schema.clone())),
                    semantic_schema: None,
                    schema_uri: current_schema.schema_uri.clone(),
                    schema_base_uri: current_schema.schema_base_uri.clone(),
                    schema_document_uri: current_schema.schema_document_uri.clone(),
                    line_index: current_schema.line_index.clone(),
                    definitions: current_schema.definitions.clone(),
                    strict: current_schema.strict,
                    dynamic_scope: current_schema.dynamic_scope.clone(),
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
                cursor,
                keys,
                accessors,
                all_of_schema,
                &CurrentSchema {
                    schema_view: Arc::new(SchemaView::AllOf(all_of_schema.clone())),
                    semantic_schema: None,
                    schema_uri: current_schema.schema_uri.clone(),
                    schema_base_uri: current_schema.schema_base_uri.clone(),
                    schema_document_uri: current_schema.schema_document_uri.clone(),
                    line_index: current_schema.line_index.clone(),
                    definitions: current_schema.definitions.clone(),
                    strict: current_schema.strict,
                    dynamic_scope: current_schema.dynamic_scope.clone(),
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
        schema_base_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        parent_dynamic_scope: &'a [SchemaUri],
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    fn description<'a: 'b, 'b>(
        &'a self,
        schema_base_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &'a SchemaStore,
        parent_dynamic_scope: &'a [SchemaUri],
        completion_hint: Option<CompletionHint>,
    ) -> tombi_future::BoxFuture<'b, Option<String>>;

    async fn detail(
        &self,
        schema_base_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &SchemaStore,
        parent_dynamic_scope: &[SchemaUri],
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.title(
            schema_base_uri,
            definitions,
            strict,
            schema_store,
            parent_dynamic_scope,
            completion_hint,
        )
        .await
    }

    async fn documentation(
        &self,
        schema_base_uri: &SchemaUri,
        definitions: &SchemaDefinitions,
        strict: Option<tombi_schema_type::BoolDefaultTrue>,
        schema_store: &SchemaStore,
        parent_dynamic_scope: &[SchemaUri],
        completion_hint: Option<CompletionHint>,
    ) -> Option<String> {
        self.description(
            schema_base_uri,
            definitions,
            strict,
            schema_store,
            parent_dynamic_scope,
            completion_hint,
        )
        .await
    }
}

fn composite_title<'a: 'b, 'b, T: CompositeSchema + Sync + Send>(
    composite_schema: &'a T,
    schema_base_uri: &'a SchemaUri,
    definitions: &'a SchemaDefinitions,
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_store: &'a SchemaStore,
    parent_dynamic_scope: &'a [SchemaUri],
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Option<String>> {
    async move {
        let mut candidates = tombi_hashmap::IndexSet::new();
        let schema_visits = tombi_schema_store::SchemaVisits::default();

        if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas_in_scope(
            composite_schema.schemas(),
            Cow::Borrowed(schema_base_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_store,
            &schema_visits,
            &[],
            Some(parent_dynamic_scope),
        )
        .await
        {
            for current_schema in &resolved_schemas {
                if matches!(current_schema.schema_view.as_ref(), SchemaView::Null) {
                    continue;
                }

                if let Some(candidate) = CompletionCandidate::title(
                    current_schema.schema_view.as_ref(),
                    &current_schema.schema_base_uri,
                    &current_schema.definitions,
                    current_schema.strict,
                    schema_store,
                    &current_schema.dynamic_scope,
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
    schema_base_uri: &'a SchemaUri,
    definitions: &'a SchemaDefinitions,
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_store: &'a SchemaStore,
    parent_dynamic_scope: &'a [SchemaUri],
    completion_hint: Option<CompletionHint>,
) -> tombi_future::BoxFuture<'b, Option<String>> {
    async move {
        let mut contents = Vec::new();
        let schema_visits = tombi_schema_store::SchemaVisits::default();

        if let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas_in_scope(
            composite_schema.schemas(),
            Cow::Borrowed(schema_base_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_store,
            &schema_visits,
            &[],
            Some(parent_dynamic_scope),
        )
        .await
        {
            for current_schema in &resolved_schemas {
                if matches!(current_schema.schema_view.as_ref(), SchemaView::Null) {
                    continue;
                }

                let title = CompletionCandidate::title(
                    current_schema.schema_view.as_ref(),
                    &current_schema.schema_base_uri,
                    &current_schema.definitions,
                    current_schema.strict,
                    schema_store,
                    &current_schema.dynamic_scope,
                    completion_hint,
                )
                .await;
                let description = CompletionCandidate::description(
                    current_schema.schema_view.as_ref(),
                    &current_schema.schema_base_uri,
                    &current_schema.definitions,
                    current_schema.strict,
                    schema_store,
                    &current_schema.dynamic_scope,
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
                schema_base_uri: &'a SchemaUri,
                definitions: &'a SchemaDefinitions,
                strict: Option<tombi_schema_type::BoolDefaultTrue>,
                schema_store: &'a SchemaStore,
                parent_dynamic_scope: &'a [SchemaUri],
                completion_hint: Option<CompletionHint>,
            ) -> tombi_future::BoxFuture<'b, Option<String>> {
                composite_title(
                    self,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
                    completion_hint,
                )
            }

            fn description<'a: 'b, 'b>(
                &'a self,
                schema_base_uri: &'a SchemaUri,
                definitions: &'a SchemaDefinitions,
                strict: Option<tombi_schema_type::BoolDefaultTrue>,
                schema_store: &'a SchemaStore,
                parent_dynamic_scope: &'a [SchemaUri],
                completion_hint: Option<CompletionHint>,
            ) -> tombi_future::BoxFuture<'b, Option<String>> {
                composite_description(
                    self,
                    schema_base_uri,
                    definitions,
                    strict,
                    schema_store,
                    parent_dynamic_scope,
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
    offset: tombi_text::Offset,
    detail: Option<String>,
    documentation: Option<String>,
    schema_base_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Option<CompletionContent> {
    if !matches!(
        value,
        tombi_json::Value::String(_) | tombi_json::Value::Number(_) | tombi_json::Value::Bool(_)
    ) {
        return None;
    }

    let label = value.to_string();
    let edit = CompletionEdit::new_literal(&label, offset, completion_hint);

    Some(CompletionContent::new_default_value(
        label,
        detail,
        documentation,
        edit,
        schema_base_uri,
        None,
    ))
}

fn tombi_json_value_to_completion_example_item(
    value: &tombi_json::Value,
    offset: tombi_text::Offset,
    detail: Option<String>,
    documentation: Option<String>,
    schema_base_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Option<CompletionContent> {
    if !matches!(
        value,
        tombi_json::Value::String(_) | tombi_json::Value::Number(_) | tombi_json::Value::Bool(_)
    ) {
        return None;
    }

    let label = value.to_string();
    let edit = CompletionEdit::new_literal(&label, offset, completion_hint);

    Some(CompletionContent::new_example_value(
        label,
        detail,
        documentation,
        edit,
        schema_base_uri,
        None,
    ))
}

fn tombi_json_value_to_completion_enum_item(
    value: &tombi_json::Value,
    offset: tombi_text::Offset,
    detail: Option<String>,
    documentation: Option<String>,
    schema_base_uri: Option<&SchemaUri>,
    completion_hint: Option<CompletionHint>,
) -> Option<CompletionContent> {
    if !matches!(
        value,
        tombi_json::Value::String(_) | tombi_json::Value::Number(_) | tombi_json::Value::Bool(_)
    ) {
        return None;
    }

    let label = value.to_string();
    let edit = CompletionEdit::new_literal(&label, offset, completion_hint);
    Some(CompletionContent::new_enum_value(
        label,
        detail,
        documentation,
        edit,
        schema_base_uri,
        None,
    ))
}

pub async fn get_completion_keys_with_context<'t>(
    root: &tombi_ast_syntax::Root<'t>,
    decoded: &'t tombi_ast_syntax::DecodedTextResolver,
    offset: tombi_text::Offset,
    toml_version: tombi_config::TomlVersion,
) -> Option<(Vec<tombi_document_tree_syntax::Key<'t>>, Vec<KeyContext>)> {
    let mut keys_vec = vec![];
    let mut key_contexts = vec![];

    for node in root.nodes_at_offset(offset) {
        if let tombi_ast_syntax::TomlNode::KeyValue(kv) = node {
            let keys = kv.keys()?;
            let keys = if keys.span().contains_inclusive(offset) {
                keys.keys()
                    .take_while(|key| key.token().unwrap().span().start <= offset)
                    .collect_vec()
            } else {
                keys.keys().collect_vec()
            };
            for (i, key) in keys.into_iter().rev().enumerate() {
                match key.try_into_document_tree(toml_version, decoded) {
                    Ok(Some(key_dt)) => {
                        let kind = if i == 0 {
                            AccessorKeyKind::KeyValue
                        } else {
                            AccessorKeyKind::Dotted
                        };
                        keys_vec.push(key_dt.clone());
                        key_contexts.push(KeyContext {
                            kind,
                            span: key_dt.span(),
                        });
                    }
                    _ => return None,
                }
            }
        } else if let tombi_ast_syntax::TomlNode::Table(table) = node {
            if let Some(header) = table.header() {
                for key in header.keys_rev() {
                    match key.try_into_document_tree(toml_version, decoded) {
                        Ok(Some(key_dt)) => {
                            keys_vec.push(key_dt.clone());
                            key_contexts.push(KeyContext {
                                kind: AccessorKeyKind::Header,
                                span: key_dt.span(),
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
                match key.try_into_document_tree(toml_version, decoded) {
                    Ok(Some(key_dt)) => {
                        keys_vec.push(key_dt.clone());
                        key_contexts.push(KeyContext {
                            kind: AccessorKeyKind::Header,
                            span: key_dt.span(),
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
