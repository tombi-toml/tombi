use std::path::PathBuf;

use crate::{CatalogUrl, SchemaUrl};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to lock document: {schema_url}")]
    DocumentLockError { schema_url: SchemaUrl },

    #[error("failed to lock reference: {ref_string}")]
    ReferenceLockError { ref_string: String },

    #[error("failed to lock schema")]
    SchemaLockError,

    #[error("definition ref not found: {definition_ref}")]
    DefinitionNotFound { definition_ref: String },

    #[error("failed to convert to catalog url: {catalog_path}")]
    CatalogPathConvertUrlFailed { catalog_path: String },

    #[error("failed to fetch catalog: {catalog_url}, reason: {reason}")]
    CatalogUrlFetchFailed {
        catalog_url: CatalogUrl,
        reason: String,
    },

    #[error("invalid catalog file url: {catalog_url}")]
    InvalidCatalogFileUrl { catalog_url: CatalogUrl },

    #[error("failed to read catalog: {catalog_path}")]
    CatalogFileReadFailed { catalog_path: PathBuf },

    #[error("unsupported schema url: {schema_url}")]
    UnsupportedSchemaUrl { schema_url: SchemaUrl },

    #[error("invalid schema url: {schema_url}")]
    InvalidSchemaUrl { schema_url: String },

    #[error("invalid schema url or file path: {schema_url_or_file_path}")]
    InvalidSchemaUrlOrFilePath { schema_url_or_file_path: String },

    #[error("schema file not found: {schema_path}")]
    SchemaFileNotFound { schema_path: PathBuf },

    #[error("schema resource not found: {schema_url}")]
    SchemaResourceNotFound { schema_url: SchemaUrl },

    #[error("failed to read schema: \"{schema_path}\"")]
    SchemaFileReadFailed { schema_path: PathBuf },

    #[error("failed to parse schema: {schema_url}, reason: {reason}")]
    SchemaFileParseFailed {
        schema_url: SchemaUrl,
        reason: String,
    },

    #[error("failed to fetch schema: {schema_url}, reason: {reason}")]
    SchemaFetchFailed {
        schema_url: SchemaUrl,
        reason: String,
    },

    #[error("unsupported source url: {source_url}")]
    UnsupportedSourceUrl { source_url: url::Url },

    #[error("invalid source url: {source_url}")]
    SourceUrlParseFailed { source_url: url::Url },

    #[error("invalid file path: {url}")]
    InvalidFilePath { url: url::Url },

    #[error("invalid json format: {url}, reason: {reason}")]
    InvalidJsonFormat { url: url::Url, reason: String },

    #[error("invalid json schema reference: {reference}")]
    InvalidJsonSchemaReference { reference: String },

    #[error("unsupported reference: {reference}")]
    UnsupportedReference { reference: String },

    #[error("unsupported url schema: {schema}")]
    UnsupportedUrlSchema { schema: String },

    #[error("schema must be an object: {schema_url}")]
    SchemaMustBeObject { schema_url: SchemaUrl },
}
