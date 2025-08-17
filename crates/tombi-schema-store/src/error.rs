use std::path::PathBuf;

use crate::{CatalogUri, SchemaUri};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to lock document: {schema_uri}")]
    DocumentLockError { schema_uri: SchemaUri },

    #[error("failed to lock reference: {ref_string}")]
    ReferenceLockError { ref_string: String },

    #[error("failed to lock schema")]
    SchemaLockError,

    #[error("definition ref not found: {definition_ref}")]
    DefinitionNotFound { definition_ref: String },

    #[error("failed to convert to catalog uri: {catalog_path}")]
    CatalogPathConvertUriFailed { catalog_path: String },

    #[error("failed to parse catalog: {tagalog_uri}, reason: {reason}")]
    CatalogFileParseFailed {
        tagalog_uri: CatalogUri,
        reason: String,
    },

    #[error("failed to fetch catalog: {catalog_uri}, reason: {reason}")]
    CatalogUriFetchFailed {
        catalog_uri: CatalogUri,
        reason: String,
    },

    #[error("catalog file not found: {catalog_path}")]
    CatalogFileNotFound { catalog_path: PathBuf },

    #[error("invalid catalog file uri: {catalog_uri}")]
    InvalidCatalogFileUri { catalog_uri: CatalogUri },

    #[error("failed to read catalog: {catalog_path}")]
    CatalogFileReadFailed { catalog_path: PathBuf },

    #[error("invalid schema uri: {schema_uri}")]
    InvalidSchemaUri { schema_uri: String },

    #[error("invalid schema uri or file path: {schema_uri_or_file_path}")]
    InvalidSchemaUriOrFilePath { schema_uri_or_file_path: String },

    #[error("schema file not found: {schema_path}")]
    SchemaFileNotFound { schema_path: PathBuf },

    #[error("schema resource not found: {schema_uri}")]
    SchemaResourceNotFound { schema_uri: SchemaUri },

    #[error("failed to read schema: \"{schema_path}\"")]
    SchemaFileReadFailed { schema_path: PathBuf },

    #[error("failed to parse schema: {schema_uri}, reason: {reason}")]
    SchemaFileParseFailed {
        schema_uri: SchemaUri,
        reason: String,
    },

    #[error("failed to fetch schema: {schema_uri}, reason: {reason}")]
    SchemaFetchFailed {
        schema_uri: SchemaUri,
        reason: String,
    },

    #[error("unsupported source uri: {source_uri}")]
    UnsupportedSourceUri { source_uri: tombi_uri::Uri },

    #[error("invalid source uri: {source_uri}")]
    SourceUriParseFailed { source_uri: tombi_uri::Uri },

    #[error("invalid file path: {uri}")]
    InvalidFilePath { uri: tombi_uri::Uri },

    #[error("invalid json format: {uri}, reason: {reason}")]
    InvalidJsonFormat { uri: tombi_uri::Uri, reason: String },

    #[error("invalid json pointer: {pointer}, schema_uri: {schema_uri}")]
    InvalidJsonPointer {
        pointer: String,
        schema_uri: SchemaUri,
    },

    #[error("invalid json schema reference: {reference}, schema_uri: {schema_uri}")]
    InvalidJsonSchemaReference {
        reference: String,
        schema_uri: SchemaUri,
    },

    #[error("unsupported reference: {reference}, schema_uri: {schema_uri}")]
    UnsupportedReference {
        reference: String,
        schema_uri: SchemaUri,
    },

    #[error("unsupported uri scheme: {scheme}, uri: {uri}", scheme = uri.scheme())]
    UnsupportedUriScheme { uri: tombi_uri::Uri },

    #[error("schema must be an object: {schema_uri}")]
    SchemaMustBeObject { schema_uri: SchemaUri },

    #[error(transparent)]
    CacheError(#[from] tombi_cache::Error),
}

impl Error {
    #[inline]
    pub fn code(&self) -> &'static str {
        match self {
            Self::DocumentLockError { .. } => "document-lock-error",
            Self::ReferenceLockError { .. } => "reference-lock-error",
            Self::SchemaLockError => "schema-lock-error",
            Self::DefinitionNotFound { .. } => "definition-not-found",
            Self::CatalogPathConvertUriFailed { .. } => "catalog-path-convert-url-failed",
            Self::CatalogFileParseFailed { .. } => "catalog-file-parse-failed",
            Self::CatalogUriFetchFailed { .. } => "catalog-url-fetch-failed",
            Self::CatalogFileNotFound { .. } => "catalog-file-not-found",
            Self::InvalidCatalogFileUri { .. } => "invalid-catalog-file-url",
            Self::CatalogFileReadFailed { .. } => "catalog-file-read-failed",
            Self::InvalidSchemaUri { .. } => "invalid-schema-uri",
            Self::InvalidSchemaUriOrFilePath { .. } => "invalid-schema-uri-or-file-path",
            Self::SchemaFileNotFound { .. } => "schema-file-not-found",
            Self::SchemaResourceNotFound { .. } => "schema-resource-not-found",
            Self::SchemaFileReadFailed { .. } => "schema-file-read-failed",
            Self::SchemaFileParseFailed { .. } => "schema-file-parse-failed",
            Self::SchemaFetchFailed { .. } => "schema-fetch-failed",
            Self::UnsupportedSourceUri { .. } => "unsupported-source-url",
            Self::SourceUriParseFailed { .. } => "source-url-parse-failed",
            Self::InvalidFilePath { .. } => "invalid-file-path",
            Self::InvalidJsonFormat { .. } => "invalid-json-format",
            Self::InvalidJsonPointer { .. } => "invalid-json-pointer",
            Self::InvalidJsonSchemaReference { .. } => "invalid-json-schema-reference",
            Self::UnsupportedReference { .. } => "unsupported-reference",
            Self::UnsupportedUriScheme { .. } => "unsupported-url-scheme",
            Self::SchemaMustBeObject { .. } => "schema-must-be-object",
            Self::CacheError(error) => error.code(),
        }
    }
}
