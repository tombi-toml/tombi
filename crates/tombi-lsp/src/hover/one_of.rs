use std::borrow::Cow;

use tombi_diagnostic::Diagnostic;
use tombi_future::Boxable;
use tombi_schema_store::{Accessor, CurrentSchema, SchemaUri};

use crate::HoverContent;

use super::{
    GetHoverContent, HoverValueContent,
    constraints::ValueConstraints,
    display_value::{DisplayValue, GetEnum},
};

pub fn get_one_of_hover_content<'a: 'b, 'b, T>(
    value: &'a T,
    position: tombi_text::Position,
    keys: &'a [tombi_document_tree::Key],
    accessors: &'a [tombi_schema_store::Accessor],
    one_of_schema: &'a tombi_schema_store::OneOfSchema,
    schema_uri: &'a SchemaUri,
    definitions: &'a tombi_schema_store::SchemaDefinitions,
    schema_context: &'a tombi_schema_store::SchemaContext,
) -> tombi_future::BoxFuture<'b, Option<HoverContent>>
where
    T: GetHoverContent
        + tombi_document_tree::ValueImpl
        + tombi_validator::Validate
        + Sync
        + Send
        + std::fmt::Debug,
{
    log::trace!("value = {:?}", value);
    log::trace!("keys = {:?}", keys);
    log::trace!("accessors = {:?}", accessors);
    log::trace!("one_of_schema = {:?}", one_of_schema);
    log::trace!("schema_uri = {:?}", schema_uri);

    async move {
        let mut one_hover_value_contents = ahash::AHashSet::new();
        let mut valid_hover_value_contents = ahash::AHashSet::new();
        let mut value_type_set = indexmap::IndexSet::new();
        let mut enum_values = Vec::new();
        let default = one_of_schema
            .default
            .as_ref()
            .and_then(|default| DisplayValue::try_from(default).ok());

        let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
            &one_of_schema.schemas,
            Cow::Borrowed(schema_uri),
            Cow::Borrowed(definitions),
            schema_context.store,
        )
        .await
        else {
            return None;
        };

        let Ok(_cycle_guard) = one_of_schema.schemas.try_write() else {
            return None;
        };

        for resolved_schema in &resolved_schemas {
            if let Some(values) = resolved_schema
                .value_schema
                .as_ref()
                .get_enum(schema_uri, definitions, schema_context)
                .await
            {
                enum_values.extend(values);
            }

            value_type_set.insert(resolved_schema.value_schema.value_type().await);
        }

        let value_type = if value_type_set.len() == 1 {
            value_type_set.into_iter().next().unwrap()
        } else {
            tombi_schema_store::ValueType::OneOf(value_type_set.into_iter().collect())
        };

        for resolved_schema in &resolved_schemas {
            match value
                .get_hover_content(
                    position,
                    keys,
                    accessors,
                    Some(resolved_schema),
                    schema_context,
                )
                .await
            {
                Some(HoverContent::Value(mut hover_value_content)) => {
                    if hover_value_content.title.is_none()
                        && hover_value_content.description.is_none()
                    {
                        if let Some(title) = &one_of_schema.title {
                            hover_value_content.title = Some(title.clone());
                        }
                        if let Some(description) = &one_of_schema.description {
                            hover_value_content.description = Some(description.clone());
                        }
                    }

                    if keys.is_empty() && accessors == hover_value_content.accessors.as_ref() {
                        hover_value_content.value_type = value_type.clone();
                    }

                    if resolved_schema.value_schema.value_type().await
                        == tombi_schema_store::ValueType::Array
                        && hover_value_content.value_type != tombi_schema_store::ValueType::Array
                    {
                        return Some(HoverContent::Value(hover_value_content));
                    }

                    match value
                        .validate(accessors, Some(resolved_schema), schema_context)
                        .await
                    {
                        Ok(()) => {
                            valid_hover_value_contents.insert(hover_value_content.clone());
                        }
                        Err(tombi_validator::Error { diagnostics, .. })
                            if diagnostics.iter().all(Diagnostic::is_warning) =>
                        {
                            valid_hover_value_contents.insert(hover_value_content.clone());
                        }
                        _ => {}
                    }

                    one_hover_value_contents.insert(hover_value_content);
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
            }
        }

        drop(_cycle_guard);

        let mut hover_value_content = if one_hover_value_contents.len() == 1 {
            one_hover_value_contents
                .into_iter()
                .next()
                .map(|mut hover_content| {
                    if hover_content.title.is_none() && hover_content.description.is_none() {
                        if let Some(title) = &one_of_schema.title {
                            hover_content.title = Some(title.clone());
                        }
                        if let Some(description) = &one_of_schema.description {
                            hover_content.description = Some(description.clone());
                        }
                    }

                    hover_content
                })
        } else if valid_hover_value_contents.len() == 1 {
            valid_hover_value_contents
                .into_iter()
                .next()
                .map(|mut hover_content| {
                    if hover_content.title.is_none() && hover_content.description.is_none() {
                        if let Some(title) = &one_of_schema.title {
                            hover_content.title = Some(title.clone());
                        }
                        if let Some(description) = &one_of_schema.description {
                            hover_content.description = Some(description.clone());
                        }
                    }

                    hover_content
                })
        } else {
            Some(HoverValueContent {
                title: None,
                description: None,
                accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
                value_type: value.value_type().into(),
                constraints: None,
                schema_uri: Some(schema_uri.to_owned()),
                range: None,
            })
        };

        if let Some(hover_value_content) = hover_value_content.as_mut() {
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

            if !enum_values.is_empty() {
                if let Some(constraints) = hover_value_content.constraints.as_mut() {
                    constraints.r#enum = Some(enum_values);
                } else {
                    hover_value_content.constraints = Some(ValueConstraints {
                        r#enum: Some(enum_values),
                        ..Default::default()
                    });
                }
            }
        }

        hover_value_content.map(HoverContent::Value)
    }
    .boxed()
}

impl GetHoverContent for tombi_schema_store::OneOfSchema {
    fn get_hover_content<'a: 'b, 'b>(
        &'a self,
        _position: tombi_text::Position,
        _keys: &'a [tombi_document_tree::Key],
        accessors: &'a [Accessor],
        current_schema: Option<&'a CurrentSchema<'a>>,
        schema_context: &'a tombi_schema_store::SchemaContext,
    ) -> tombi_future::BoxFuture<'b, Option<HoverContent>> {
        async move {
            let Some(current_schema) = current_schema else {
                unreachable!("schema must be provided");
            };

            let mut title_description_set = ahash::AHashSet::new();
            let mut value_type_set = indexmap::IndexSet::new();
            let mut enum_values = Vec::new();
            let default = self
                .default
                .as_ref()
                .and_then(|default| DisplayValue::try_from(default).ok());

            let Some(resolved_schemas) = tombi_schema_store::resolve_and_collect_schemas(
                &self.schemas,
                current_schema.schema_uri.clone(),
                current_schema.definitions.clone(),
                schema_context.store,
            )
            .await
            else {
                return None;
            };

            let Ok(_cycle_guard) = self.schemas.try_write() else {
                return None;
            };

            for resolved_schema in &resolved_schemas {
                if resolved_schema.value_schema.title().is_some()
                    || resolved_schema.value_schema.description().is_some()
                {
                    title_description_set.insert((
                        resolved_schema
                            .value_schema
                            .title()
                            .map(ToString::to_string),
                        resolved_schema
                            .value_schema
                            .description()
                            .map(ToString::to_string),
                    ));
                }
                value_type_set.insert(resolved_schema.value_schema.value_type().await);

                if let Some(values) = resolved_schema
                    .value_schema
                    .as_ref()
                    .get_enum(
                        &resolved_schema.schema_uri,
                        &resolved_schema.definitions,
                        schema_context,
                    )
                    .await
                {
                    enum_values.extend(values);
                }
            }

            drop(_cycle_guard);

            let (mut title, mut description) = if title_description_set.len() == 1 {
                title_description_set.into_iter().next().unwrap()
            } else {
                (None, None)
            };

            if title.is_none() && description.is_none() {
                if let Some(t) = &self.title {
                    title = Some(t.clone());
                }
                if let Some(d) = &self.description {
                    description = Some(d.clone());
                }
            }

            let value_type: tombi_schema_store::ValueType = if value_type_set.len() == 1 {
                value_type_set.into_iter().next().unwrap()
            } else {
                tombi_schema_store::ValueType::OneOf(value_type_set.into_iter().collect())
            };

            let mut hover_value_content = HoverValueContent {
                title,
                description,
                accessors: tombi_schema_store::Accessors::from(accessors.to_vec()),
                value_type,
                constraints: None,
                schema_uri: Some(current_schema.schema_uri.as_ref().to_owned()),
                range: None,
            };

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

            if !enum_values.is_empty() {
                if let Some(constraints) = hover_value_content.constraints.as_mut() {
                    constraints.r#enum = Some(enum_values);
                } else {
                    hover_value_content.constraints = Some(ValueConstraints {
                        r#enum: Some(enum_values),
                        ..Default::default()
                    });
                }
            }

            Some(HoverContent::Value(hover_value_content))
        }
        .boxed()
    }
}
