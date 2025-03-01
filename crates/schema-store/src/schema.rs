mod all_of_schema;
mod any_of_schema;
mod array_schema;
mod boolean_schema;
mod document_schema;
mod float_schema;
mod integer_schema;
mod local_date_schema;
mod local_date_time_schema;
mod local_time_schema;
mod offset_date_time_schema;
mod one_of_schema;
mod referable_schema;
mod schema_context;
mod schema_url;
mod source_schema;
mod string_schema;
mod table_schema;
mod value_schema;
mod x_tombi;

use std::sync::Arc;

pub use all_of_schema::AllOfSchema;
pub use any_of_schema::AnyOfSchema;
pub use array_schema::{ArraySchema, ArrayValuesOrderBy};
pub use boolean_schema::BooleanSchema;
pub use document_schema::DocumentSchema;
pub use float_schema::FloatSchema;
use futures::future::BoxFuture;
pub use integer_schema::IntegerSchema;
pub use local_date_schema::LocalDateSchema;
pub use local_date_time_schema::LocalDateTimeSchema;
pub use local_time_schema::LocalTimeSchema;
pub use offset_date_time_schema::OffsetDateTimeSchema;
pub use one_of_schema::OneOfSchema;
pub use referable_schema::{is_online_url, CurrentSchema, Referable};
pub use schema_context::SchemaContext;
pub use schema_url::SchemaUrl;
pub use source_schema::{SourceSchema, SubSchemaUrlMap};
pub use string_schema::StringSchema;
pub use table_schema::{TableKeysOrderBy, TableSchema};
pub use value_schema::*;
pub use x_tombi::*;

use crate::SchemaAccessor;
use crate::{Accessor, SchemaStore};

pub type SchemaProperties =
    Arc<tokio::sync::RwLock<ahash::AHashMap<Accessor, Referable<ValueSchema>>>>;
pub type SchemaPatternProperties =
    Arc<tokio::sync::RwLock<ahash::AHashMap<String, Referable<ValueSchema>>>>;
pub type SchemaItemTokio = Arc<tokio::sync::RwLock<Referable<ValueSchema>>>;
pub type SchemaDefinitions =
    Arc<tokio::sync::RwLock<ahash::AHashMap<String, Referable<ValueSchema>>>>;
pub type ReferableValueSchemas = Arc<tokio::sync::RwLock<Vec<Referable<ValueSchema>>>>;

#[derive(Debug, Clone)]
pub struct Schema {
    pub toml_version: Option<config::TomlVersion>,
    pub url: crate::SchemaUrl,
    pub include: Vec<String>,
    pub root_keys: Option<Vec<SchemaAccessor>>,
}

pub trait FindSchemaCandidates {
    fn find_schema_candidates<'a: 'b, 'b>(
        &'a self,
        accessors: &'a [Accessor],
        schema_url: &'a SchemaUrl,
        definitions: &'a SchemaDefinitions,
        schema_store: &'a SchemaStore,
    ) -> BoxFuture<'b, (Vec<ValueSchema>, Vec<crate::Error>)>;
}
