mod all_of;
mod any_of;
mod array;
mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod one_of;
mod string;
mod table;

pub use all_of::find_all_of_completion_items;
pub use any_of::find_any_of_completion_items;
use array::type_hint_array;
use boolean::type_hint_boolean;
use tombi_config::TomlVersion;
use float::type_hint_float;
use futures::{future::BoxFuture, FutureExt};
use integer::type_hint_integer;
use local_date::type_hint_local_date;
use local_date_time::type_hint_local_date_time;
use local_time::type_hint_local_time;
use offset_date_time::type_hint_offset_date_time;
pub use one_of::find_one_of_completion_items;
use schema_store::{
    Accessor, ArraySchema, BooleanSchema, CurrentSchema, FloatSchema, IntegerSchema,
    LocalDateSchema, LocalDateTimeSchema, LocalTimeSchema, OffsetDateTimeSchema, SchemaDefinitions,
    SchemaStore, SchemaUrl, StringSchema, TableSchema, ValueSchema,
};
use string::type_hint_string;

use super::{
    schema_completion::SchemaCompletion, CompletionCandidate, CompletionContent, CompletionHint,
    FindCompletionContents,
};

impl FindCompletionContents for document_tree::Value {
    fn find_completion_contents<'a: 'b, 'b>(
        &'a self,
        position: text::Position,
        keys: &'a [document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a schema_store::SchemaContext<'a>,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Vec<CompletionContent>> {
        tracing::trace!("self = {:?}", self);
        tracing::trace!("accessors = {:?}", accessors);
        tracing::trace!("keys = {:?}", keys);
        tracing::trace!("current_schema = {:?}", current_schema);
        tracing::trace!("completion_hint = {:?}", completion_hint);

        async move {
            match self {
                Self::Boolean(_)
                | Self::Integer(_)
                | Self::Float(_)
                | Self::String(_)
                | Self::OffsetDateTime(_)
                | Self::LocalDateTime(_)
                | Self::LocalDate(_)
                | Self::LocalTime(_) => Vec::with_capacity(0),
                Self::Array(array) => {
                    array
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Table(table) => {
                    table
                        .find_completion_contents(
                            position,
                            keys,
                            accessors,
                            current_schema,
                            schema_context,
                            completion_hint,
                        )
                        .await
                }
                Self::Incomplete { .. } => match current_schema {
                    Some(current_schema) => {
                        SchemaCompletion
                            .find_completion_contents(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                completion_hint,
                            )
                            .await
                    }
                    None => {
                        let last_key = keys.last();

                        match (&last_key, completion_hint) {
                            (Some(last_key), Some(CompletionHint::EqualTrigger { range }))
                                if range.end() < position =>
                            {
                                vec![CompletionContent::new_type_hint_key(
                                    last_key,
                                    schema_context.toml_version,
                                    None,
                                    completion_hint,
                                )]
                            }
                            _ => type_hint_value(
                                last_key,
                                position,
                                schema_context.toml_version,
                                None,
                                completion_hint,
                            ),
                        }
                    }
                },
            }
        }
        .boxed()
    }
}

pub fn type_hint_value(
    key: Option<&document_tree::Key>,
    position: text::Position,
    toml_version: TomlVersion,
    schema_url: Option<&SchemaUrl>,
    completion_hint: Option<CompletionHint>,
) -> Vec<CompletionContent> {
    let mut completion_contents = itertools::concat([
        type_hint_boolean(position, schema_url, completion_hint),
        type_hint_integer(position, schema_url, completion_hint),
        type_hint_float(position, schema_url, completion_hint),
        type_hint_string(position, schema_url, completion_hint),
        type_hint_local_date_time(position, schema_url, completion_hint),
        type_hint_local_date(position, schema_url, completion_hint),
        type_hint_local_time(position, schema_url, completion_hint),
        type_hint_offset_date_time(position, schema_url, completion_hint),
        type_hint_array(position, schema_url, completion_hint),
        vec![CompletionContent::new_type_hint_inline_table(
            position,
            schema_url,
            completion_hint,
        )],
    ]);

    if let Some(key) = key {
        let need_key_hint = match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. } | CompletionHint::EqualTrigger { range },
            ) => range.end() == position || range.end() <= key.range().start(),
            Some(CompletionHint::InTableHeader | CompletionHint::InArray) | None => true,
        };
        if need_key_hint {
            completion_contents.push(CompletionContent::new_type_hint_key(
                key,
                toml_version,
                schema_url,
                completion_hint,
            ));
        }
    } else {
        completion_contents.push(CompletionContent::new_type_hint_empty_key(
            position,
            schema_url,
            completion_hint,
        ))
    }

    completion_contents
}

impl CompletionCandidate for ValueSchema {
    fn title<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Option<String>> {
        async move {
            match self {
                Self::Boolean(BooleanSchema { title, .. })
                | Self::Integer(IntegerSchema { title, .. })
                | Self::Float(FloatSchema { title, .. })
                | Self::String(StringSchema { title, .. })
                | Self::OffsetDateTime(OffsetDateTimeSchema { title, .. })
                | Self::LocalDateTime(LocalDateTimeSchema { title, .. })
                | Self::LocalDate(LocalDateSchema { title, .. })
                | Self::LocalTime(LocalTimeSchema { title, .. })
                | Self::Array(ArraySchema { title, .. })
                | Self::Table(TableSchema { title, .. }) => {
                    title.as_deref().map(ToString::to_string)
                }
                Self::OneOf(one_of) => {
                    one_of
                        .title(schema_url, definitions, schema_store, completion_hint)
                        .await
                }
                Self::AnyOf(any_of) => {
                    any_of
                        .title(schema_url, definitions, schema_store, completion_hint)
                        .await
                }
                Self::AllOf(all_of) => {
                    all_of
                        .title(schema_url, definitions, schema_store, completion_hint)
                        .await
                }
                Self::Null => None,
            }
        }
        .boxed()
    }

    fn description<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
        completion_hint: Option<CompletionHint>,
    ) -> BoxFuture<'b, Option<String>> {
        async move {
            match self {
                Self::Boolean(BooleanSchema { description, .. })
                | Self::Integer(IntegerSchema { description, .. })
                | Self::Float(FloatSchema { description, .. })
                | Self::String(StringSchema { description, .. })
                | Self::OffsetDateTime(OffsetDateTimeSchema { description, .. })
                | Self::LocalDateTime(LocalDateTimeSchema { description, .. })
                | Self::LocalDate(LocalDateSchema { description, .. })
                | Self::LocalTime(LocalTimeSchema { description, .. })
                | Self::Array(ArraySchema { description, .. })
                | Self::Table(TableSchema { description, .. }) => {
                    description.as_deref().map(ToString::to_string)
                }
                Self::OneOf(one_of) => {
                    one_of
                        .description(schema_url, definitions, schema_store, completion_hint)
                        .await
                }
                Self::AnyOf(any_of) => {
                    any_of
                        .description(schema_url, definitions, schema_store, completion_hint)
                        .await
                }
                Self::AllOf(all_of) => {
                    all_of
                        .description(schema_url, definitions, schema_store, completion_hint)
                        .await
                }
                Self::Null => None,
            }
        }
        .boxed()
    }
}
