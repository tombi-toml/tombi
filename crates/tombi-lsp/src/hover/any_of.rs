use std::borrow::Cow;

use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaContext, SchemaUri};

use super::CompositeKind;
use crate::{HoverContent, hover::display_value::GetEnum};

use super::{
    GetHoverContent, HoverValueContent, constraints::ValueConstraints, display_value::DisplayValue,
};

pub fn get_any_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree_syntax::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    any_of_schema: &'a tombi_schema_store::AnyOfSchema,
    schema_uri: &'a SchemaUri,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    strict: Option<tombi_schema_type::BoolDefaultTrue>,
    schema_context: &'a SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent
        + tombi_document_tree_syntax::ValueImpl
        + tombi_validator::Validate
        + Sync
        + Send
        + std::fmt::Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("keys = {:?}", keys);
    log::trace!("accessors = {:?}", accessors);
    log::trace!("any_of_schema = {:?}", any_of_schema);
    log::trace!("schema_uri = {:?}", schema_uri);

    async move {
        let mut hover_value_contents = vec![];
        let default = any_of_schema
            .default
            .as_ref()
            .and_then(|default| DisplayValue::try_from(default).ok());

        let resolved_schemas = tombi_schema_store::resolve_and_collect_schemas(
            &any_of_schema.schemas,
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            strict,
            schema_context.store,
            &schema_context.schema_visits,
            accessors,
        )
        .await?;
        let value_type = any_of_schema.value_type().await;

        let evaluation = tombi_validator::evaluate_applicator(
            tombi_validator::Applicator::AnyOf,
            value,
            accessors,
            &resolved_schemas,
            schema_context,
        )
        .await;
        let has_applicable_branch = evaluation
            .branches
            .iter()
            .any(|branch| branch.is_applicable());

        for (resolved_schema, branch_applicability) in
            resolved_schemas.iter().zip(&evaluation.branches)
        {
            if has_applicable_branch && !branch_applicability.is_applicable() {
                continue;
            }
            let projected_schema = crate::schema_resolver::project_schema_for_concrete_value(
                value,
                resolved_schema,
                schema_context,
            );
            let navigation_schema = projected_schema.as_ref().unwrap_or(resolved_schema);

            match value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(navigation_schema),
                    schema_context,
                )
                .await
            {
                Some(HoverContent::Value(mut hover_value_content)) => {
                    if hover_value_content.title.is_none()
                        && hover_value_content.description.is_none()
                    {
                        if let Some(title) = &any_of_schema.title {
                            hover_value_content.title = Some(title.clone());
                        }
                        if let Some(description) = &any_of_schema.description {
                            hover_value_content.description = Some(description.clone());
                        }
                    }

                    if accessors.len() == hover_value_content.accessors.as_ref().len() {
                        hover_value_content.value_type = value_type.clone();
                    }

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
                Some(HoverContent::Directive(hover_content)) => {
                    return Some(HoverContent::Directive(hover_content));
                }
                Some(HoverContent::DirectiveContent(hover_content)) => {
                    return Some(HoverContent::DirectiveContent(hover_content));
                }
                None => {
                    continue;
                }
            };
        }

        let mut hover_value_content = super::first_most_specific_hover_value_content(
            hover_value_contents,
            CompositeKind::Any,
        )
        .unwrap_or_else(|| HoverValueContent {
            title: any_of_schema.title.clone(),
            description: any_of_schema.description.clone(),
            accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
            value_type: value.value_type().into(),
            constraints: None,
            schema_uri: Some(super::schema_link_uri(schema_uri, any_of_schema.range)),
            range: None,
            schema_tooltip: None,
        });
        super::inherit_matching_nullable_type(&value_type, &mut hover_value_content.value_type);
        hover_value_content
            .schema_uri
            .get_or_insert_with(|| super::schema_link_uri(schema_uri, any_of_schema.range));

        if let Some(default) = default {
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

        Some(HoverContent::Value(hover_value_content))
    }
    .boxed()
}

impl GetHoverContent for tombi_schema_store::AnyOfSchema {
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

            let default = self
                .default
                .as_ref()
                .and_then(|default| DisplayValue::try_from(default).ok());

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
                super::first_most_specific_hover_value_content(contents, CompositeKind::Any)
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

            if let Some(default) = default {
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

            Some(HoverContent::Value(hover_value_content))
        }
        .boxed()
    }
}
