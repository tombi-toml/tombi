use itertools::Itertools;

use tombi_future::Boxable;
use tombi_schema_store::{Accessor, ArraySchema, CurrentSchema, SchemaView};

use crate::{
    comment_directive::get_array_comment_directive_content_with_schema_uri,
    goto_type_definition::{
        GetTypeDefinition, TypeDefinition, adjacent_type_definition,
        all_of::get_all_of_type_definition, any_of::get_any_of_type_definition,
        comment::get_tombi_value_comment_directive_type_definition,
        one_of::get_one_of_type_definition,
    },
    schema_resolver::resolve_array_item_schema,
};

impl GetTypeDefinition for tombi_document_tree_syntax::Array {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        position: tombi_text::Position,
        keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Vec<TypeDefinition>> {
        log::trace!("self = {:?}", self);
        log::trace!("keys = {:?}", keys);
        log::trace!("accessors = {:?}", accessors);
        log::trace!("current_schema = {:?}", current_schema);

        async move {
            if let Some((comment_directive_context, schema_uri)) =
                get_array_comment_directive_content_with_schema_uri(self, position, accessors)
                && let hover_content = get_tombi_value_comment_directive_type_definition(
                    comment_directive_context,
                    schema_uri,
                )
                .await
                && !hover_content.is_empty()
            {
                return hover_content;
            }

            if let Some(Ok(current_schema)) = schema_context
                .get_subschema(accessors, current_schema)
                .await
            {
                return self
                    .get_type_definition(
                        position,
                        keys,
                        accessors,
                        Some(&current_schema),
                        schema_context,
                    )
                    .await;
            }

            if let Some(current_schema) = current_schema {
                match current_schema.schema_view.as_ref() {
                    SchemaView::Array(array_schema) => {
                        if keys.is_empty()
                            && matches!(
                                self.kind(),
                                tombi_document_tree_syntax::ArrayKind::ArrayOfTable
                                    | tombi_document_tree_syntax::ArrayKind::ParentArrayOfTable
                            )
                            && self.values().iter().any(|value| {
                                tombi_document_tree_syntax::ValueImpl::range(value)
                                    .start
                                    .line
                                    == position.line
                            })
                        {
                            return array_schema
                                .get_type_definition(
                                    position,
                                    keys,
                                    accessors,
                                    Some(current_schema),
                                    schema_context,
                                )
                                .await;
                        }

                        for (index, value) in self.values().iter().enumerate() {
                            if value.contains(position) {
                                let accessor = Accessor::Index(index);

                                if let Some(current_schema) = resolve_array_item_schema(
                                    index,
                                    array_schema,
                                    current_schema,
                                    schema_context,
                                )
                                .await
                                {
                                    return value
                                        .get_type_definition(
                                            position,
                                            keys,
                                            &accessors
                                                .iter()
                                                .cloned()
                                                .chain(std::iter::once(accessor.clone()))
                                                .collect_vec(),
                                            Some(&current_schema),
                                            schema_context,
                                        )
                                        .await;
                                }

                                let type_definitions = adjacent_type_definition(
                                    self,
                                    position,
                                    keys,
                                    accessors,
                                    Some(current_schema),
                                    schema_context,
                                    array_schema.one_of.as_deref(),
                                    array_schema.any_of.as_deref(),
                                    array_schema.all_of.as_deref(),
                                )
                                .await;
                                if !type_definitions.is_empty() {
                                    return type_definitions;
                                }

                                return value
                                    .get_type_definition(
                                        position,
                                        keys,
                                        &accessors
                                            .iter()
                                            .cloned()
                                            .chain(std::iter::once(accessor))
                                            .collect_vec(),
                                        None,
                                        schema_context,
                                    )
                                    .await;
                            }
                        }
                        return array_schema
                            .get_type_definition(
                                position,
                                keys,
                                accessors,
                                Some(current_schema),
                                schema_context,
                            )
                            .await;
                    }
                    SchemaView::OneOf(one_of_schema) => {
                        return get_one_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            one_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            current_schema.strict,
                            schema_context,
                        )
                        .await;
                    }
                    SchemaView::AnyOf(any_of_schema) => {
                        return get_any_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            any_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            current_schema.strict,
                            schema_context,
                        )
                        .await;
                    }
                    SchemaView::AllOf(all_of_schema) => {
                        return get_all_of_type_definition(
                            self,
                            position,
                            keys,
                            accessors,
                            all_of_schema,
                            &current_schema.schema_uri,
                            &current_schema.definitions,
                            current_schema.strict,
                            schema_context,
                        )
                        .await;
                    }
                    _ => {}
                }
            }

            for (index, value) in self.values().iter().enumerate() {
                if value.contains(position) {
                    let accessor = Accessor::Index(index);
                    return value
                        .get_type_definition(
                            position,
                            keys,
                            &accessors
                                .iter()
                                .cloned()
                                .chain(std::iter::once(accessor))
                                .collect_vec(),
                            None,
                            schema_context,
                        )
                        .await;
                }
            }

            Vec::new()
        }
        .boxed()
    }
}

impl GetTypeDefinition for ArraySchema {
    fn get_type_definition<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
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
