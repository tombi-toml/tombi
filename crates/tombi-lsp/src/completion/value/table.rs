use std::borrow::Cow;

use futures::future::join_all;
use itertools::Itertools;
use tombi_future::{BoxFuture, Boxable};
use tombi_schema_store::{
    Accessor, CurrentSchema, CurrentValueSchema, FindSchemaCandidates, Referable, SchemaAccessor,
    SchemaStore, TableSchema, ValueSchema, is_online_url,
};

use crate::{
    comment_directive::get_table_comment_directive_content_with_schema_uri,
    completion::{
        CompletionCandidate, CompletionContent, CompletionHint, FindCompletionContents,
        comment::get_tombi_comment_directive_content_completion_contents,
        value::{
            all_of::find_all_of_completion_items, any_of::find_any_of_completion_items,
            one_of::find_one_of_completion_items, type_hint_value,
        },
    },
};

impl FindCompletionContents for tombi_document_tree::Table {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        log::trace!("self = {:?}", self);
        log::trace!("keys = {:?}", keys);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);
        log::trace!("completion_hint = {:?}", completion_hint);

        async move {
            if keys.is_empty() {
                if let Some((comment_directive_context, schema_uri)) =
                    get_table_comment_directive_content_with_schema_uri(self, position, accessors)
                    && let Some(completions) =
                        get_tombi_comment_directive_content_completion_contents(
                            comment_directive_context,
                            schema_uri,
                        )
                        .await
                {
                    return completions;
                }

                if !matches!(
                    self.kind(),
                    tombi_document_tree::TableKind::InlineTable { .. }
                ) && completion_hint != Some(CompletionHint::InTableHeader)
                {
                    // Skip if the cursor is the end space of key value like:
                    //
                    // ```toml
                    // key = "value" █
                    // ```
                    for value in self.values() {
                        let end = value.range().end;
                        if end.line == position.line && end.column < position.column {
                            return vec![];
                        }
                    }
                }
            }

            // `range.end` points to the cursor position right after `}`.
            // At that point, completion should not behave as "inside inline table".
            if matches!(
                self.kind(),
                tombi_document_tree::TableKind::InlineTable { .. }
            ) && position >= self.range().end
            {
                return Vec::with_capacity(0);
            }

            if let Some(Ok(document_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
                && let Some(value_schema) = &document_schema.value_schema
            {
                return self
                    .find_completion_contents(
                        position,
                        keys,
                        accessors,
                        Some(&CurrentSchema {
                            value_schema: CurrentValueSchema::Shared(value_schema.clone()),
                            schema_uri: Cow::Borrowed(&document_schema.schema_uri),
                            definitions: Cow::Borrowed(&document_schema.definitions),
                        }),
                        schema_context,
                        completion_hint,
                    )
                    .await;
            }

            if let Some(current_schema) = current_schema {
                match current_schema.value_schema.as_ref() {
                    ValueSchema::Table(table_schema) => {
                        let mut completion_contents = Vec::new();

                        if let Some(key) = keys.first() {
                            let accessor_str = &key.value;
                            if let Some(value) = self.get(key) {
                                let accessor: Accessor = Accessor::Key(accessor_str.to_string());
                                let schema_accessor = SchemaAccessor::from(&accessor);
                                let need_magic_trigger = match completion_hint {
                                    Some(
                                        CompletionHint::DotTrigger { range, .. }
                                        | CompletionHint::EqualTrigger { range, .. },
                                    ) => range.end <= key.range().start,
                                    Some(
                                        CompletionHint::InArray { .. }
                                        | CompletionHint::InTableHeader
                                        | CompletionHint::Comma { .. },
                                    ) => false,
                                    None => true,
                                };

                                if table_schema
                                    .properties
                                    .read()
                                    .await
                                    .contains_key(&schema_accessor)
                                {
                                    if matches!(
                                        value,
                                        tombi_document_tree::Value::Incomplete { .. }
                                    ) && need_magic_trigger
                                    {
                                        return CompletionContent::new_magic_triggers(
                                            accessor_str,
                                            position,
                                            Some(current_schema.schema_uri.as_ref()),
                                        );
                                    }

                                    if let Ok(Some(current_schema)) = table_schema
                                        .resolve_property_schema(
                                            &schema_accessor,
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        log::trace!(
                                            "property_schema = {:?}",
                                            current_schema.value_schema
                                        );

                                        let mut contents = value
                                            .find_completion_contents(
                                                position,
                                                &keys[1..],
                                                &accessors
                                                    .iter()
                                                    .cloned()
                                                    .chain(std::iter::once(accessor))
                                                    .collect_vec(),
                                                Some(&current_schema),
                                                schema_context,
                                                completion_hint,
                                            )
                                            .await;

                                        if !contents.is_empty()
                                            && current_schema.value_schema.deprecated().await
                                                == Some(true)
                                        {
                                            for content in &mut contents {
                                                if !content.in_comment {
                                                    content.deprecated = Some(true);
                                                }
                                            }
                                        }

                                        return contents;
                                    }
                                } else if keys.len() == 1 {
                                    let property_keys = table_schema
                                        .properties
                                        .read()
                                        .await
                                        .keys()
                                        .cloned()
                                        .collect_vec();
                                    for property_key in property_keys {
                                        let key_name = property_key.to_string();
                                        if !key_name.starts_with(accessor_str) {
                                            continue;
                                        }

                                        if let Some(value) = self.get(&key_name)
                                            && check_used_table_value(
                                                value,
                                                accessors.is_empty(),
                                                completion_hint,
                                            )
                                        {
                                            continue;
                                        }

                                        if let Ok(Some(current_schema)) = table_schema
                                            .resolve_property_schema(
                                                &property_key,
                                                current_schema.schema_uri.clone(),
                                                current_schema.definitions.clone(),
                                                schema_context.store,
                                            )
                                            .await
                                        {
                                            log::trace!(
                                                "property_schema = {:?}",
                                                &current_schema.value_schema
                                            );

                                            let Some(mut contents) =
                                                collect_table_key_completion_contents(
                                                    self,
                                                    &key_name,
                                                    position,
                                                    accessors,
                                                    table_schema,
                                                    &current_schema,
                                                    schema_context,
                                                    completion_hint,
                                                )
                                                .await
                                            else {
                                                continue;
                                            };

                                            if !contents.is_empty()
                                                && current_schema.value_schema.deprecated().await
                                                    == Some(true)
                                            {
                                                for content in &mut contents {
                                                    if !content.in_comment {
                                                        content.deprecated = Some(true);
                                                    }
                                                }
                                            }

                                            completion_contents.extend(contents);
                                        }
                                    }
                                }

                                if !completion_contents.is_empty() {
                                    return completion_contents;
                                }

                                if let Some(pattern_properties) = &table_schema.pattern_properties {
                                    let pattern_keys = pattern_properties
                                        .read()
                                        .await
                                        .keys()
                                        .cloned()
                                        .collect_vec();
                                    for property_key in pattern_keys {
                                        let Ok(pattern) = tombi_regex::Regex::new(&property_key)
                                        else {
                                            log::warn!(
                                                "Invalid regex pattern property: {}",
                                                property_key
                                            );
                                            continue;
                                        };
                                        if pattern.is_match(accessor_str) {
                                            if let Ok(Some(current_schema)) = table_schema
                                                .resolve_pattern_property_schema(
                                                    &property_key,
                                                    current_schema.schema_uri.clone(),
                                                    current_schema.definitions.clone(),
                                                    schema_context.store,
                                                )
                                                .await
                                            {
                                                let mut contents =
                                                    get_property_value_completion_contents(
                                                        value,
                                                        position,
                                                        key,
                                                        keys,
                                                        accessors,
                                                        Some(&current_schema),
                                                        schema_context,
                                                        completion_hint,
                                                    )
                                                    .await;

                                                if !contents.is_empty()
                                                    && current_schema
                                                        .value_schema
                                                        .deprecated()
                                                        .await
                                                        == Some(true)
                                                {
                                                    for content in &mut contents {
                                                        if !content.in_comment {
                                                            content.deprecated = Some(true);
                                                        }
                                                    }
                                                }

                                                return contents;
                                            }
                                        }
                                    }
                                }

                                if let Some((_, referable_additional_property_schema)) =
                                    &table_schema.additional_property_schema
                                {
                                    log::trace!(
                                        "additional_property_schema = {:?}",
                                        referable_additional_property_schema
                                    );

                                    if let Ok(Some(current_schema)) =
                                        tombi_schema_store::resolve_schema_item(
                                            referable_additional_property_schema,
                                            current_schema.schema_uri.clone(),
                                            current_schema.definitions.clone(),
                                            schema_context.store,
                                        )
                                        .await
                                    {
                                        let mut contents = get_property_value_completion_contents(
                                            value,
                                            position,
                                            key,
                                            keys,
                                            accessors,
                                            Some(&current_schema),
                                            schema_context,
                                            completion_hint,
                                        )
                                        .await;

                                        if !contents.is_empty()
                                            && current_schema.value_schema.deprecated().await
                                                == Some(true)
                                        {
                                            for content in &mut contents {
                                                if !content.in_comment {
                                                    content.deprecated = Some(true);
                                                }
                                            }
                                        }

                                        return contents;
                                    }
                                }

                                if table_schema
                                    .allows_any_additional_properties(schema_context.strict())
                                {
                                    return get_property_value_completion_contents(
                                        value,
                                        position,
                                        key,
                                        keys,
                                        accessors,
                                        None,
                                        schema_context,
                                        completion_hint,
                                    )
                                    .await;
                                }
                            }
                        } else {
                            let schema_accessors = table_schema
                                .properties
                                .read()
                                .await
                                .keys()
                                .cloned()
                                .collect_vec();
                            for schema_accessor in schema_accessors {
                                let key_name = schema_accessor.to_string();

                                if let Some(value) = self.get(&key_name)
                                    && check_used_table_value(
                                        value,
                                        accessors.is_empty(),
                                        completion_hint,
                                    )
                                {
                                    continue;
                                }

                                let online_ref_metadata = {
                                    let properties = table_schema.properties.read().await;
                                    properties
                                        .get(&schema_accessor)
                                        .and_then(|property_schema| {
                                            if let Referable::Ref {
                                                reference,
                                                title,
                                                description,
                                                deprecated,
                                                ..
                                            } = &property_schema.property_schema
                                                && is_online_url(reference)
                                            {
                                                Some((
                                                    title.clone(),
                                                    description.clone(),
                                                    *deprecated,
                                                ))
                                            } else {
                                                None
                                            }
                                        })
                                };

                                if let Some((title, description, deprecated)) = online_ref_metadata
                                {
                                    completion_contents.push(CompletionContent::new_key(
                                        &key_name,
                                        position,
                                        title,
                                        description,
                                        table_schema.required.as_ref(),
                                        Some(current_schema.schema_uri.as_ref()),
                                        deprecated,
                                        completion_hint,
                                    ));
                                    continue;
                                }

                                if let Ok(Some(current_schema)) = table_schema
                                    .resolve_property_schema(
                                        &schema_accessor,
                                        current_schema.schema_uri.clone(),
                                        current_schema.definitions.clone(),
                                        schema_context.store,
                                    )
                                    .await
                                {
                                    let Some(contents) = collect_table_key_completion_contents(
                                        self,
                                        &key_name,
                                        position,
                                        accessors,
                                        table_schema,
                                        &current_schema,
                                        schema_context,
                                        completion_hint,
                                    )
                                    .await
                                    else {
                                        continue;
                                    };
                                    completion_contents.extend(contents)
                                }
                            }

                            if let Some(sub_schema_uri_map) = schema_context.sub_schema_uri_map {
                                for (root_accessors, sub_schema_uri) in sub_schema_uri_map {
                                    if let Some(SchemaAccessor::Key(last_key)) =
                                        root_accessors.last()
                                    {
                                        let head_accessors =
                                            &root_accessors[..root_accessors.len() - 1];
                                        if head_accessors == accessors
                                            && let Ok(Some(document_schema)) = schema_context
                                                .store
                                                .try_get_document_schema(sub_schema_uri)
                                                .await
                                            && let Some(value_schema) =
                                                &document_schema.value_schema
                                        {
                                            completion_contents.push(CompletionContent::new_key(
                                                last_key,
                                                position,
                                                value_schema
                                                    .detail(
                                                        &current_schema.schema_uri,
                                                        &current_schema.definitions,
                                                        schema_context.store,
                                                        completion_hint,
                                                    )
                                                    .await,
                                                value_schema
                                                    .documentation(
                                                        &current_schema.schema_uri,
                                                        &current_schema.definitions,
                                                        schema_context.store,
                                                        completion_hint,
                                                    )
                                                    .await,
                                                None,
                                                Some(current_schema.schema_uri.as_ref()),
                                                value_schema.deprecated().await,
                                                completion_hint,
                                            ));
                                        }
                                    }
                                }
                            }

                            if let Some(pattern_properties) = &table_schema.pattern_properties {
                                let patterns = pattern_properties
                                    .read()
                                    .await
                                    .keys()
                                    .map(ToString::to_string)
                                    .collect_vec();
                                completion_contents.push(CompletionContent::new_pattern_key(
                                    table_schema.additional_key_label.as_deref(),
                                    patterns.as_ref(),
                                    position,
                                    Some(current_schema.schema_uri.as_ref()),
                                    completion_hint,
                                ))
                            } else if let Some((_, additional_property_schema)) =
                                &table_schema.additional_property_schema
                                && let Ok(Some(CurrentSchema {
                                    value_schema,
                                    schema_uri,
                                    ..
                                })) = tombi_schema_store::resolve_schema_item(
                                    additional_property_schema,
                                    current_schema.schema_uri.clone(),
                                    current_schema.definitions.clone(),
                                    schema_context.store,
                                )
                                .await
                            {
                                completion_contents.push(CompletionContent::new_additional_key(
                                    table_schema.additional_key_label.as_deref(),
                                    position,
                                    Some(schema_uri.as_ref()),
                                    value_schema.deprecated().await,
                                    completion_hint,
                                ));
                            }
                        }
                        completion_contents
                    }
                    ValueSchema::OneOf(one_of_schema) => {
                        find_one_of_completion_items(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    ValueSchema::AnyOf(any_of_schema) => {
                        find_any_of_completion_items(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    ValueSchema::AllOf(all_of_schema) => {
                        find_all_of_completion_items(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                    }
                    _ => Vec::with_capacity(0),
                }
            } else if let Some(key) = keys.first() {
                if let Some(value) = self.get(key) {
                    get_property_value_completion_contents(
                        value,
                        position,
                        key,
                        keys,
                        accessors,
                        None,
                        schema_context,
                        completion_hint,
                    )
                    .await
                } else {
                    Vec::with_capacity(0)
                }
            } else {
                vec![CompletionContent::new_type_hint_empty_key(
                    position,
                    None,
                    completion_hint,
                )]
            }
        }
        .boxed()
    }
}

impl FindCompletionContents for TableSchema {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        log::trace!("self = {:?}", self);
        log::trace!("position = {:?}", position);
        log::trace!("keys = {:?}", keys);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);
        log::trace!("completion_hint = {:?}", completion_hint);

        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            let mut completion_items = Vec::new();

            let property_keys = self.properties.read().await.keys().cloned().collect_vec();
            for key in property_keys {
                let label = key.to_string();
                let current_schema = match self
                    .resolve_property_schema(
                        &key,
                        current_schema.schema_uri.clone(),
                        current_schema.definitions.clone(),
                        schema_context.store,
                    )
                    .await
                {
                    Ok(Some(current_schema)) => current_schema,
                    Ok(None) => continue,
                    Err(err) => {
                        log::warn!("{err}");
                        continue;
                    }
                };

                let (schema_candidates, errors) = current_schema
                    .value_schema
                    .find_schema_candidates(
                        accessors,
                        &current_schema.schema_uri,
                        &current_schema.definitions,
                        schema_context.store,
                    )
                    .await;

                for error in errors {
                    log::warn!("{}", error);
                }

                for schema_candidate in schema_candidates {
                    if let Some(CompletionHint::InTableHeader) = completion_hint
                        && count_table_or_array_schema(&current_schema, schema_context.store).await
                            == 0
                    {
                        continue;
                    }

                    completion_items.push(CompletionContent::new_key(
                        &label,
                        position,
                        schema_candidate
                            .detail(
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                schema_context.store,
                                completion_hint,
                            )
                            .await,
                        schema_candidate
                            .documentation(
                                &current_schema.schema_uri,
                                &current_schema.definitions,
                                schema_context.store,
                                completion_hint,
                            )
                            .await,
                        self.required.as_ref(),
                        Some(current_schema.schema_uri.as_ref()),
                        current_schema.value_schema.deprecated().await,
                        completion_hint,
                    ));
                }
            }

            completion_items.push(CompletionContent::new_type_hint_inline_table(
                position,
                Some(current_schema.schema_uri.as_ref()),
                completion_hint,
            ));

            completion_items
        }
        .boxed()
    }
}

async fn count_table_or_array_schema(
    current_schema: &CurrentSchema<'_>,
    schema_store: &SchemaStore,
) -> usize {
    join_all(
        current_schema
            .value_schema
            .match_flattened_schemas(
                &|schema| matches!(schema, ValueSchema::Table(_) | ValueSchema::Array(_)),
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_store,
            )
            .await
            .into_iter()
            .map(|schema| async {
                match schema {
                    ValueSchema::Array(array_schema) => {
                        if let Some(item) = array_schema.items
                            && let Ok(Some(CurrentSchema {
                                schema_uri,
                                value_schema,
                                definitions,
                            })) = tombi_schema_store::resolve_schema_item(
                                &item,
                                Cow::Borrowed(&current_schema.schema_uri),
                                Cow::Borrowed(&current_schema.definitions),
                                schema_store,
                            )
                            .await
                        {
                            return value_schema
                                .is_match(
                                    &|schema| matches!(schema, ValueSchema::Table(_)),
                                    &schema_uri,
                                    &definitions,
                                    schema_store,
                                )
                                .await;
                        }
                        true
                    }
                    ValueSchema::Table(_) => true,
                    _ => unreachable!("only table and array are allowed"),
                }
            }),
    )
    .await
    .into_iter()
    .filter(|&is_table_or_array_schema| is_table_or_array_schema)
    .count()
}

fn get_property_value_completion_contents<'a: 'b, 'b>(
    value: &'a tombi_document_tree::Value,
    position: tombi_text::Position,
    key: &'a tombi_document_tree::Key,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [Accessor],
    current_schema: Option<&'a CurrentSchema<'a>>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> BoxFuture<'b, Vec<CompletionContent>> {
    log::trace!("key = {:?}", key);
    log::trace!("value = {:?}", value);
    log::trace!("keys = {:?}", keys);
    log::trace!("accessors = {:?}", accessors);
    log::trace!("current_schema = {:?}", current_schema);
    log::trace!("completion_hint = {:?}", completion_hint);

    async move {
        if keys.len() == 1 {
            match completion_hint {
                Some(
                    CompletionHint::DotTrigger { range } | CompletionHint::EqualTrigger { range },
                ) => {
                    let key = keys.first().unwrap();
                    if current_schema.is_none() {
                        if range.end <= key.range().start {
                            return vec![CompletionContent::new_type_hint_key(
                                &key.value,
                                key.range(),
                                None,
                                completion_hint,
                            )];
                        }
                        return type_hint_value(Some(key), position, None, completion_hint);
                    }
                }
                Some(CompletionHint::InTableHeader) => {
                    if let Some(current_schema) = current_schema
                        && count_table_or_array_schema(current_schema, schema_context.store).await
                            == 0
                    {
                        return Vec::with_capacity(0);
                    }
                }
                Some(CompletionHint::InArray { .. } | CompletionHint::Comma { .. }) | None => {
                    if matches!(value, tombi_document_tree::Value::Incomplete { .. }) {
                        return CompletionContent::new_magic_triggers(
                            &key.value,
                            position,
                            current_schema.map(|schema| schema.schema_uri.as_ref()),
                        );
                    }
                }
            }
        }

        value
            .find_completion_contents(
                position,
                &keys[1..],
                &accessors
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Accessor::Key(key.value.clone())))
                    .collect_vec(),
                current_schema,
                schema_context,
                completion_hint,
            )
            .await
    }
    .boxed()
}

fn check_used_table_value(
    value: &tombi_document_tree::Value,
    is_root: bool,
    completion_hint: Option<CompletionHint>,
) -> bool {
    match value {
        tombi_document_tree::Value::Boolean(_)
        | tombi_document_tree::Value::Integer(_)
        | tombi_document_tree::Value::Float(_)
        | tombi_document_tree::Value::String(_)
        | tombi_document_tree::Value::OffsetDateTime(_)
        | tombi_document_tree::Value::LocalDateTime(_)
        | tombi_document_tree::Value::LocalDate(_)
        | tombi_document_tree::Value::LocalTime(_) => return true,
        tombi_document_tree::Value::Array(array) => {
            if array.kind() == tombi_document_tree::ArrayKind::Array {
                return true;
            }
        }
        tombi_document_tree::Value::Table(table) => {
            if matches!(
                table.kind(),
                tombi_document_tree::TableKind::InlineTable { .. }
            ) || (is_root
                && completion_hint.is_none()
                && table.kind() == tombi_document_tree::TableKind::Table)
            {
                return true;
            }
        }
        tombi_document_tree::Value::Incomplete { .. } => {}
    }
    false
}

fn collect_table_key_completion_contents<'a: 'b, 'b>(
    table: &'a tombi_document_tree::Table,
    key_name: &'a String,
    position: tombi_text::Position,
    accessors: &'a [Accessor],
    table_schema: &'a TableSchema,
    current_schema: &'a CurrentSchema<'a>,
    schema_context: &'a tombi_schema_store::SchemaContext<'a>,
    completion_hint: Option<CompletionHint>,
) -> BoxFuture<'b, Option<Vec<CompletionContent>>> {
    async move {
        let mut completion_contents = Vec::new();

        let (schema_candidates, errors) = current_schema
            .value_schema
            .find_schema_candidates(
                accessors,
                &current_schema.schema_uri,
                &current_schema.definitions,
                schema_context.store,
            )
            .await;

        for error in errors {
            log::warn!("{}", error);
        }

        for schema_candidate in schema_candidates {
            match &schema_candidate {
                ValueSchema::Boolean(_)
                | ValueSchema::Integer(_)
                | ValueSchema::Float(_)
                | ValueSchema::String(_)
                | ValueSchema::OffsetDateTime(_)
                | ValueSchema::LocalDateTime(_)
                | ValueSchema::LocalDate(_)
                | ValueSchema::LocalTime(_) => {
                    if matches!(completion_hint, Some(CompletionHint::InTableHeader))
                        || table.contains_key(key_name)
                    {
                        return None;
                    }
                }
                ValueSchema::Array(_) => {
                    if matches!(completion_hint, Some(CompletionHint::InTableHeader))
                        && count_table_or_array_schema(current_schema, schema_context.store).await
                            == 0
                    {
                        return None;
                    }
                }
                ValueSchema::Table(table_schema) => {
                    if matches!(completion_hint, Some(CompletionHint::InTableHeader))
                        && count_table_or_array_schema(current_schema, schema_context.store).await
                            == 0
                    {
                        return None;
                    }
                    if let Some(tombi_document_tree::Value::Table(table)) = table.get(key_name) {
                        let properties = table_schema.properties.read().await;
                        if !table_schema.allows_any_additional_properties(schema_context.strict())
                            && properties.keys().all(|key| {
                                let property_name = &key.to_string();
                                table.get(property_name).is_some()
                            })
                        {
                            return None;
                        }
                    }
                }
                ValueSchema::Null
                | ValueSchema::OneOf(_)
                | ValueSchema::AnyOf(_)
                | ValueSchema::AllOf(_) => {
                    unreachable!(
                        "Null, OneOf, AnyOf, and AllOf are not allowed in flattened schema"
                    );
                }
            }

            completion_contents.push(CompletionContent::new_key(
                key_name,
                position,
                schema_candidate
                    .detail(
                        &current_schema.schema_uri,
                        &current_schema.definitions,
                        schema_context.store,
                        completion_hint,
                    )
                    .await,
                schema_candidate
                    .documentation(
                        &current_schema.schema_uri,
                        &current_schema.definitions,
                        schema_context.store,
                        completion_hint,
                    )
                    .await,
                table_schema.required.as_ref(),
                Some(&current_schema.schema_uri),
                current_schema.value_schema.deprecated().await,
                completion_hint,
            ));
        }

        Some(completion_contents)
    }
    .boxed()
}
