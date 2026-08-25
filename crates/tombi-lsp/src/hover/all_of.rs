use std::borrow::Cow;

use itertools::Itertools;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext, SchemaUri};

use super::CompositeKind;
use crate::{HoverContent, hover::display_value::GetEnum};

use super::{
    GetHoverContent, HoverValueContent, constraints::ValueConstraints, display_value::DisplayValue,
};

pub fn get_all_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree_syntax::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    all_of_schema: &'a tombi_schema_store::AllOfSchema,
    schema_uri: &'a SchemaUri,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_context: &'a SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent + tombi_document_tree_syntax::ValueImpl + Sync + Send + std::fmt::Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("keys = {:?}", keys);
    log::trace!("accessors = {:?}", accessors);
    log::trace!("all_of_schema = {:?}", all_of_schema);
    log::trace!("schema_uri = {:?}", schema_uri);

    async move {
        let mut hover_value_contents = Vec::new();

        let resolved_schemas = tombi_schema_store::resolve_and_collect_schemas(
            &all_of_schema.schemas,
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await?;

        for resolved_schema in &resolved_schemas {
            let projected_schema = crate::schema_resolver::project_schema_for_concrete_value(
                value,
                resolved_schema,
                schema_context,
            );
            let resolved_schema = projected_schema.as_ref().unwrap_or(resolved_schema);
            if let Some(hover_content) = value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(resolved_schema),
                    schema_context,
                )
                .await
            {
                match hover_content {
                    HoverContent::Value(mut hover_value_content) => {
                        if hover_value_content
                            .constraints
                            .as_ref()
                            .is_none_or(|constraints| constraints.r#enum.is_none())
                            && hover_value_content.accessors.as_ref() == accessors
                            && let Some(enum_values) = resolved_schema
                                .schema_view
                                .as_ref()
                                .get_enum(
                                    &resolved_schema.schema_uri,
                                    &resolved_schema.definitions,
                                    resolved_schema.strict,
                                    schema_context,
                                )
                                .await
                        {
                            hover_value_content
                                .constraints
                                .get_or_insert_default()
                                .r#enum = Some(enum_values);
                        }
                        hover_value_contents.push(hover_value_content);
                    }
                    HoverContent::Directive(hover_content) => {
                        return Some(HoverContent::Directive(hover_content));
                    }
                    HoverContent::DirectiveContent(hover_content) => {
                        return Some(HoverContent::DirectiveContent(hover_content));
                    }
                }
            }
        }

        let mut hover_value_content = super::first_most_specific_hover_value_content(
            hover_value_contents,
            CompositeKind::All,
        )
        .unwrap_or_else(|| HoverValueContent {
            title: None,
            description: None,
            accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
            value_type: value.value_type().into(),
            constraints: None,
            schema_uri: Some(super::schema_link_uri(schema_uri, all_of_schema.range)),
            range: None,
            schema_tooltip: None,
        });
        hover_value_content
            .schema_uri
            .get_or_insert_with(|| super::schema_link_uri(schema_uri, all_of_schema.range));

        if hover_value_content.title.is_none() && hover_value_content.description.is_none() {
            hover_value_content.title = all_of_schema.title.clone();
            hover_value_content.description = all_of_schema.description.clone();
        }

        if hover_value_content.accessors.as_ref().len() == accessors.len()
            && let Some(default) = all_of_schema
                .default
                .as_ref()
                .and_then(|default| DisplayValue::try_from(default).ok())
        {
            if let Some(constraints) = hover_value_content.constraints.as_mut() {
                if constraints.default.is_none() {
                    constraints.default = Some(default);
                }
            } else {
                hover_value_content.constraints = Some(ValueConstraints {
                    default: Some(default),
                    ..Default::default()
                });
            }
        }

        if let Some(all_of_examples) = all_of_schema.examples.as_ref() {
            let all_of_examples = all_of_examples
                .iter()
                .filter_map(|example| DisplayValue::try_from(example).ok())
                .collect_vec();

            if !all_of_examples.is_empty() {
                if let Some(constraints) = hover_value_content.constraints.as_mut() {
                    constraints.examples = super::merge_optional_vec(
                        constraints.examples.take(),
                        Some(all_of_examples),
                    );
                } else {
                    hover_value_content.constraints = Some(ValueConstraints {
                        examples: Some(all_of_examples),
                        ..Default::default()
                    });
                }
            }
        }

        Some(HoverContent::Value(hover_value_content))
    }
    .boxed()
}

impl GetHoverContent for tombi_schema_store::AllOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree_syntax::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            let resolved_schemas = tombi_schema_store::resolve_and_collect_schemas(
                &self.schemas,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                current_schema.strict,
                schema_context.store,
                &schema_context.schema_visits,
                accessors,
            )
            .await?;

            let value_type = self.value_type().await;
            let mut contents = Vec::new();
            for schema in &resolved_schemas {
                if let Some(HoverContent::Value(content)) = schema
                    .schema_view
                    .get_hover_content(_position, _keys, accessors, Some(schema), schema_context)
                    .await
                {
                    contents.push(content);
                }
            }

            let mut hover_value_content =
                super::first_most_specific_hover_value_content(contents, CompositeKind::All)
                    .unwrap_or_else(|| HoverValueContent {
                        title: self.title.clone(),
                        description: self.description.clone(),
                        accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
                        value_type,
                        constraints: None,
                        schema_uri: super::current_schema_link_uri(Some(current_schema)),
                        range: None,
                        schema_tooltip: None,
                    });

            if let Some(default) = self
                .default
                .as_ref()
                .and_then(|default| DisplayValue::try_from(default).ok())
            {
                if let Some(constraints) = hover_value_content.constraints.as_mut() {
                    if constraints.default.is_none() {
                        constraints.default = Some(default);
                    }
                } else {
                    hover_value_content.constraints = Some(ValueConstraints {
                        default: Some(default),
                        ..Default::default()
                    });
                }
            }

            if let Some(all_of_examples) = self.examples.as_ref() {
                let all_of_examples = all_of_examples
                    .iter()
                    .filter_map(|example| DisplayValue::try_from(example).ok())
                    .collect_vec();

                if !all_of_examples.is_empty() {
                    if let Some(constraints) = hover_value_content.constraints.as_mut() {
                        constraints.examples = super::merge_optional_vec(
                            constraints.examples.take(),
                            Some(all_of_examples),
                        );
                    } else {
                        hover_value_content.constraints = Some(ValueConstraints {
                            examples: Some(all_of_examples),
                            ..Default::default()
                        });
                    }
                }
            }

            Some(HoverContent::Value(hover_value_content))
        }
        .boxed()
    }
}
