mod all_of_schema;
mod any_of_schema;
mod array_schema;
mod boolean_schema;
mod document_schema;
mod float_schema;
mod if_then_else_schema;
mod integer_schema;
mod local_date_schema;
mod local_date_time_schema;
mod local_time_schema;
mod not_schema;
mod offset_date_time_schema;
mod one_of_schema;
mod referable_schema;
mod schema_context;
mod schema_cycle_guard;
mod source_schema;
mod string_schema;
mod table_schema;
mod value_schema;

use std::sync::Arc;

use crate::{Accessor, SchemaStore};
pub use all_of_schema::AllOfSchema;
pub use any_of_schema::AnyOfSchema;
pub use array_schema::{ArraySchema, XTombiArrayValuesOrder};
pub use boolean_schema::BooleanSchema;
pub use document_schema::DocumentSchema;
pub use float_schema::FloatSchema;
pub use if_then_else_schema::IfThenElseSchema;
pub use integer_schema::IntegerSchema;
pub use local_date_schema::LocalDateSchema;
pub use local_date_time_schema::LocalDateTimeSchema;
pub use local_time_schema::LocalTimeSchema;
pub use not_schema::NotSchema;
pub use offset_date_time_schema::OffsetDateTimeSchema;
pub use one_of_schema::OneOfSchema;
pub use referable_schema::{
    CurrentSchema, Referable, is_online_url, resolve_and_collect_schemas, resolve_json_pointer,
    resolve_schema_item,
};
pub use schema_context::SchemaContext;
pub use schema_cycle_guard::{SchemaCycleGuard, SchemaVisits};
pub use source_schema::{SourceSchema, SubSchemaUriMap};
pub use string_schema::StringSchema;
pub use table_schema::{Dependency, TableKeysOrderGroup, TableSchema, XTombiTableKeysOrder};
pub use tombi_accessor::{SchemaAccessor, SchemaAccessors};
pub use tombi_uri::{CatalogUri, SchemaUri};
pub use value_schema::*;

pub type SchemaProperties =
    Arc<tokio::sync::RwLock<tombi_hashmap::IndexMap<SchemaAccessor, PropertySchema>>>;
pub type SchemaPatternProperties =
    Arc<tokio::sync::RwLock<tombi_hashmap::HashMap<String, PropertySchema>>>;
pub type SchemaItem = Arc<tokio::sync::RwLock<Referable<ValueSchema>>>;
pub type SchemaMap = tombi_hashmap::HashMap<String, Referable<ValueSchema>>;
pub type SchemaDefinitions = Arc<tokio::sync::RwLock<SchemaMap>>;
pub type SchemaAnchors = Arc<tokio::sync::RwLock<SchemaMap>>;
pub type SchemaDynamicAnchors = Arc<tokio::sync::RwLock<SchemaMap>>;
pub type AnchorCollector = SchemaMap;
pub type DynamicAnchorCollector = SchemaMap;
pub type ReferableValueSchemas = Arc<tokio::sync::RwLock<Vec<Referable<ValueSchema>>>>;

pub trait CompositeSchema {
    fn title(&self) -> Option<String>;
    fn description(&self) -> Option<String>;
    fn schemas(&self) -> &ReferableValueSchemas;
}

impl CompositeSchema for OneOfSchema {
    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &ReferableValueSchemas {
        &self.schemas
    }
}

impl CompositeSchema for AnyOfSchema {
    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &ReferableValueSchemas {
        &self.schemas
    }
}

impl CompositeSchema for AllOfSchema {
    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn description(&self) -> Option<String> {
        self.description.clone()
    }

    fn schemas(&self) -> &ReferableValueSchemas {
        &self.schemas
    }
}

pub(crate) fn referable_from_schema_value(
    value: &tombi_json::ValueNode,
    string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    dialect: Option<crate::JsonSchemaDialect>,
    anchor_collector: Option<&mut AnchorCollector>,
    dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
) -> Option<Referable<ValueSchema>> {
    match value {
        tombi_json::ValueNode::Object(object) => Referable::<ValueSchema>::new(
            object,
            string_formats,
            dialect,
            anchor_collector,
            dynamic_anchor_collector,
        ),
        tombi_json::ValueNode::Bool(bool) => Some(Referable::Resolved {
            schema_uri: None,
            value: Arc::new(bool_value_schema(bool.value, bool.range)),
        }),
        _ => None,
    }
}

pub(crate) fn schema_item_from_schema_value(
    value: &tombi_json::ValueNode,
    string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    dialect: Option<crate::JsonSchemaDialect>,
    anchor_collector: Option<&mut AnchorCollector>,
    dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
) -> Option<SchemaItem> {
    referable_from_schema_value(
        value,
        string_formats,
        dialect,
        anchor_collector,
        dynamic_anchor_collector,
    )
    .map(|schema| Arc::new(tokio::sync::RwLock::new(schema)))
}

pub(crate) fn bool_value_schema(allow: bool, range: tombi_text::Range) -> ValueSchema {
    if allow {
        ValueSchema::Anything(range)
    } else {
        ValueSchema::Nothing(range)
    }
}

pub(crate) fn adjacent_applicators(
    object: &tombi_json::ObjectNode,
    string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    dialect: Option<crate::JsonSchemaDialect>,
    mut anchor_collector: Option<&mut AnchorCollector>,
    mut dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
) -> (
    Option<Box<OneOfSchema>>,
    Option<Box<AnyOfSchema>>,
    Option<Box<AllOfSchema>>,
    Option<Box<NotSchema>>,
) {
    let one_of = object
        .get("oneOf")
        .is_some()
        .then(|| {
            OneOfSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )
        })
        .map(Box::new);
    let any_of = object
        .get("anyOf")
        .is_some()
        .then(|| {
            AnyOfSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )
        })
        .map(Box::new);
    let all_of = object
        .get("allOf")
        .is_some()
        .then(|| {
            AllOfSchema::new(
                object,
                string_formats,
                dialect,
                anchor_collector.as_deref_mut(),
                dynamic_anchor_collector.as_deref_mut(),
            )
        })
        .map(Box::new);
    let not = NotSchema::new(
        object,
        string_formats,
        dialect,
        anchor_collector,
        dynamic_anchor_collector,
    )
    .map(Box::new);

    (one_of, any_of, all_of, not)
}

pub(crate) fn update_named_anchors(
    object: &tombi_json::ObjectNode,
    referable: &Referable<ValueSchema>,
    dialect: Option<crate::JsonSchemaDialect>,
    anchor_collector: Option<&mut AnchorCollector>,
    mut dynamic_anchor_collector: Option<&mut DynamicAnchorCollector>,
) {
    let Some(dialect) = dialect else {
        return;
    };

    if crate::supports_keyword(dialect, "$anchor")
        && let Some(anchor) = object
            .get("$anchor")
            .and_then(|value| value.as_str())
            .filter(|anchor| is_plain_name_fragment(anchor))
        && let Some(anchor_collector) = anchor_collector
    {
        anchor_collector
            .entry(format!("#{anchor}"))
            .or_insert_with(|| referable.clone());
    }
    if crate::supports_keyword(dialect, "$dynamicAnchor")
        && let Some(dynamic_anchor) = object
            .get("$dynamicAnchor")
            .and_then(|value| value.as_str())
            .filter(|dynamic_anchor| is_plain_name_fragment(dynamic_anchor))
        && let Some(dynamic_anchor_collector) = dynamic_anchor_collector.as_deref_mut()
    {
        dynamic_anchor_collector
            .entry(format!("#{dynamic_anchor}"))
            .or_insert_with(|| referable.clone());
    }
    if crate::supports_keyword(dialect, "$recursiveAnchor")
        && object
            .get("$recursiveAnchor")
            .and_then(|value| value.as_bool())
            == Some(true)
        && let Some(dynamic_anchor_collector) = dynamic_anchor_collector
    {
        dynamic_anchor_collector
            .entry("#".to_string())
            .or_insert_with(|| referable.clone());
    }
}

#[inline]
fn is_plain_name_fragment(fragment: &str) -> bool {
    !fragment.is_empty() && !fragment.contains('/')
}

#[derive(Debug, Clone)]
pub struct PropertySchema {
    pub key_range: tombi_text::Range,
    pub property_schema: Referable<ValueSchema>,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub title: Option<String>,
    pub description: Option<String>,
    pub toml_version: Option<tombi_config::TomlVersion>,
    pub schema_uri: tombi_uri::SchemaUri,
    pub catalog_uri: Option<Arc<tombi_uri::CatalogUri>>,
    pub include: Vec<String>,
    pub sub_root_keys: Option<Vec<SchemaAccessor>>,
}

pub trait FindSchemaCandidates {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_uri: &'a SchemaUri,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> tombi_future::BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)>;
}
