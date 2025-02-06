use schema_store::{Accessor, Accessors, ArraySchema, ValueSchema, ValueType};
use tower_lsp::lsp_types::Url;

use crate::hover::{
    all_of::get_all_of_hover_content, any_of::get_any_of_hover_content,
    constraints::DataConstraints, one_of::get_one_of_hover_content, GetHoverContent, HoverContent,
};

impl GetHoverContent for document_tree::Array {
    fn get_hover_content(
        &self,
        accessors: &Vec<Accessor>,
        value_schema: Option<&ValueSchema>,
        toml_version: config::TomlVersion,
        position: text::Position,
        keys: &[document_tree::Key],
        schema_url: Option<&Url>,
        definitions: &schema_store::SchemaDefinitions,
    ) -> Option<HoverContent> {
        tracing::debug!("self: {:?}", self);
        tracing::trace!("keys: {:?}", keys);
        tracing::trace!("accessors: {:?}", accessors);
        tracing::trace!("value_schema: {:?}", value_schema);

        match value_schema {
            Some(ValueSchema::Array(array_schema)) => {
                for (index, value) in self.values().iter().enumerate() {
                    if value.range().contains(position) {
                        let accessor = Accessor::Index(index);

                        return array_schema
                            .operate_item(
                                |item_schema| {
                                    let mut hover_content = value.get_hover_content(
                                        &accessors
                                            .clone()
                                            .into_iter()
                                            .chain(std::iter::once(accessor.clone()))
                                            .collect(),
                                        Some(item_schema),
                                        toml_version,
                                        position,
                                        keys,
                                        schema_url,
                                        definitions,
                                    )?;

                                    if keys.is_empty()
                                        && self.kind() == document_tree::ArrayKind::ArrayOfTables
                                    {
                                        if let Some(constraints) = &mut hover_content.constraints {
                                            constraints.min_items = array_schema.min_items;
                                            constraints.max_items = array_schema.max_items;
                                            constraints.unique_items = array_schema.unique_items;
                                        }
                                    }

                                    if hover_content.title.is_none()
                                        && hover_content.description.is_none()
                                    {
                                        if let Some(title) = &array_schema.title {
                                            hover_content.title = Some(title.clone());
                                        }
                                        if let Some(description) = &array_schema.description {
                                            hover_content.description = Some(description.clone());
                                        }
                                    }
                                    Some(hover_content)
                                },
                                definitions,
                            )
                            .unwrap_or_else(|| {
                                value.get_hover_content(
                                    &accessors
                                        .clone()
                                        .into_iter()
                                        .chain(std::iter::once(accessor))
                                        .collect(),
                                    None,
                                    toml_version,
                                    position,
                                    keys,
                                    schema_url,
                                    definitions,
                                )
                            });
                    }
                }
                array_schema
                    .get_hover_content(
                        accessors,
                        value_schema,
                        toml_version,
                        position,
                        keys,
                        schema_url,
                        definitions,
                    )
                    .map(|mut hover_content| {
                        hover_content.range = Some(self.range());
                        hover_content
                    })
            }
            Some(ValueSchema::OneOf(one_of_schema)) => get_one_of_hover_content(
                self,
                accessors,
                one_of_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Some(ValueSchema::AnyOf(any_of_schema)) => get_any_of_hover_content(
                self,
                accessors,
                any_of_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Some(ValueSchema::AllOf(all_of_schema)) => get_all_of_hover_content(
                self,
                accessors,
                all_of_schema,
                toml_version,
                position,
                keys,
                schema_url,
                definitions,
            ),
            Some(_) => None,
            None => {
                for (index, value) in self.values().iter().enumerate() {
                    if value.range().contains(position) {
                        let accessor = Accessor::Index(index);
                        return value.get_hover_content(
                            &accessors
                                .clone()
                                .into_iter()
                                .chain(std::iter::once(accessor))
                                .collect(),
                            None,
                            toml_version,
                            position,
                            keys,
                            schema_url,
                            definitions,
                        );
                    }
                }
                Some(HoverContent {
                    title: None,
                    description: None,
                    accessors: Accessors::new(accessors.clone()),
                    value_type: ValueType::Array,
                    constraints: None,
                    schema_url: None,
                    range: Some(self.range()),
                })
            }
        }
    }
}

impl GetHoverContent for ArraySchema {
    fn get_hover_content(
        &self,
        accessors: &Vec<Accessor>,
        _value_schema: Option<&ValueSchema>,
        _toml_version: config::TomlVersion,
        _position: text::Position,
        _keys: &[document_tree::Key],
        schema_url: Option<&Url>,
        _definitions: &schema_store::SchemaDefinitions,
    ) -> Option<HoverContent> {
        Some(HoverContent {
            title: self.title.clone(),
            description: self.description.clone(),
            accessors: Accessors::new(accessors.clone()),
            value_type: ValueType::Array,
            constraints: Some(DataConstraints {
                min_items: self.min_items,
                max_items: self.max_items,
                unique_items: self.unique_items,
                ..Default::default()
            }),
            schema_url: schema_url.cloned(),
            range: None,
        })
    }
}
