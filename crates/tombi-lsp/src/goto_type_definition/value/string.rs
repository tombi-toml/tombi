use itertools::Itertools;

use tombi_comment_directive::value::{StringCommonFormatRules, StringCommonLintRules};
use tombi_future::Boxable;
use tombi_schema_store::SchemaView;
use tombi_x_keyword::StringFormat;

use crate::{
    comment_directive::get_key_table_value_comment_directive_content_and_schema_uri,
    goto_type_definition::{
        GetTypeDefinition, TypeDefinition, adjacent_type_definition,
        all_of::get_all_of_type_definition, any_of::get_any_of_type_definition,
        comment::get_tombi_value_comment_directive_type_definition,
        one_of::get_one_of_type_definition, prefer_type_definitions,
    },
};

impl GetTypeDefinition for tombi_document_tree_syntax::String {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        log::trace!("self = {:?}", self);
        log::trace!("keys = {:?}", keys);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_key_table_value_comment_directive_content_and_schema_uri::<
                    StringCommonFormatRules,
                    StringCommonLintRules,
                >(self.comment_directives(), position, accessors)
                && let hover_content = get_tombi_value_comment_directive_type_definition(
                    comment_directive_context,
                    schema_uri,
                )
                .await
                && !hover_content.is_empty()
            {
                return hover_content;
            }

            if let Some(current_schema) = current_schema {
                match current_schema.schema_view.as_ref() {
                    SchemaView::String(string_schema) => {
                        let base_type_definition = string_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;

                        prefer_type_definitions(
                            adjacent_type_definition(
                                self,
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                string_schema.one_of.as_deref(),
                                string_schema.any_of.as_deref(),
                                string_schema.all_of.as_deref(),
                            )
                            .await,
                            base_type_definition,
                        )
                    }
                    SchemaView::OffsetDateTime(offset_date_time_schema)
                        if schema_context.has_string_format(StringFormat::DateTime) =>
                    {
                        let base_type_definition = offset_date_time_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;

                        prefer_type_definitions(
                            adjacent_type_definition(
                                self,
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                offset_date_time_schema.one_of.as_deref(),
                                offset_date_time_schema.any_of.as_deref(),
                                offset_date_time_schema.all_of.as_deref(),
                            )
                            .await,
                            base_type_definition,
                        )
                    }
                    SchemaView::LocalDateTime(local_date_time_schema)
                        if schema_context.has_string_format(StringFormat::DateTimeLocal) =>
                    {
                        let base_type_definition = local_date_time_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;

                        prefer_type_definitions(
                            adjacent_type_definition(
                                self,
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                local_date_time_schema.one_of.as_deref(),
                                local_date_time_schema.any_of.as_deref(),
                                local_date_time_schema.all_of.as_deref(),
                            )
                            .await,
                            base_type_definition,
                        )
                    }
                    SchemaView::LocalDate(local_date_schema)
                        if schema_context.has_string_format(StringFormat::Date) =>
                    {
                        let base_type_definition = local_date_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;

                        prefer_type_definitions(
                            adjacent_type_definition(
                                self,
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                local_date_schema.one_of.as_deref(),
                                local_date_schema.any_of.as_deref(),
                                local_date_schema.all_of.as_deref(),
                            )
                            .await,
                            base_type_definition,
                        )
                    }
                    SchemaView::LocalTime(local_time_schema)
                        if schema_context.has_string_format(StringFormat::TimeLocal) =>
                    {
                        let base_type_definition = local_time_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;

                        prefer_type_definitions(
                            adjacent_type_definition(
                                self,
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                                local_time_schema.one_of.as_deref(),
                                local_time_schema.any_of.as_deref(),
                                local_time_schema.all_of.as_deref(),
                            )
                            .await,
                            base_type_definition,
                        )
                    }
                    SchemaView::OneOf(one_of_schema) => {
                        get_one_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            current_schema.schema_uri.as_ref(),
                            current_schema.definitions.as_ref(),
                            current_schema.strict,
                            schema_context,
                        )
                        .await
                    }
                    SchemaView::AnyOf(any_of_schema) => {
                        get_any_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            current_schema.schema_uri.as_ref(),
                            current_schema.definitions.as_ref(),
                            current_schema.strict,
                            schema_context,
                        )
                        .await
                    }
                    SchemaView::AllOf(all_of_schema) => {
                        get_all_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            current_schema.schema_uri.as_ref(),
                            current_schema.definitions.as_ref(),
                            current_schema.strict,
                            schema_context,
                        )
                        .await
                    }
                    _ if current_schema
                        .semantic_schema
                        .as_deref()
                        .is_some_and(|schema| !schema.has_type_assertion()) =>
                    {
                        let (one_of, any_of, all_of, _) =
                            current_schema.schema_view.adjacent_applicators();

                        adjacent_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            Some(current_schema),
                            schema_context,
                            one_of,
                            any_of,
                            all_of,
                        )
                        .await
                    }
                    _ => Vec::new(),
                }
            } else {
                Vec::new()
            }
        }
        .boxed()
    }
}

impl GetTypeDefinition for tombi_schema_store::StringSchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [tombi_schema_store::Accessor],
        current_schema: Option<&'a tombi_schema_store::CurrentSchema<'a>>,
        _schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        async move {
            current_schema.map_or_else(Vec::new, |schema| {
                let mut schema_uri = schema.schema_uri.as_ref().clone();
                schema_uri.set_fragment(Some(&format!("L{}", self.range.start.line + 1)));

                vec![TypeDefinition {
                    schema_uri,
                    schema_accessors: accessors.iter().map(Into::into).collect_vec(),
                    range: schema.schema_view.range(),
                }]
            })
        }
        .boxed()
    }
}
