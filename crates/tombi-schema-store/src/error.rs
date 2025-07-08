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

    #[error("invalid json pointer: {pointer}, schema_url: {schema_url}")]
    InvalidJsonPointer {
        pointer: String,
        schema_url: SchemaUrl,
    },

    #[error("invalid json schema reference: {reference}, schema_url: {schema_url}")]
    InvalidJsonSchemaReference {
        reference: String,
        schema_url: SchemaUrl,
    },

    #[error("unsupported reference: {reference}, schema_url: {schema_url}")]
    UnsupportedReference {
        reference: String,
        schema_url: SchemaUrl,
    },

    #[error("unsupported url scheme: {scheme}, url: {url}", scheme = url.scheme())]
    UnsupportedUrlScheme { url: url::Url },

    #[error("schema must be an object: {schema_url}")]
    SchemaMustBeObject { schema_url: SchemaUrl },

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
            Self::CatalogPathConvertUrlFailed { .. } => "catalog-path-convert-url-failed",
            Self::CatalogUrlFetchFailed { .. } => "catalog-url-fetch-failed",
            Self::InvalidCatalogFileUrl { .. } => "invalid-catalog-file-url",
            Self::CatalogFileReadFailed { .. } => "catalog-file-read-failed",
            Self::InvalidSchemaUrl { .. } => "invalid-schema-url",
            Self::InvalidSchemaUrlOrFilePath { .. } => "invalid-schema-url-or-file-path",
            Self::SchemaFileNotFound { .. } => "schema-file-not-found",
            Self::SchemaResourceNotFound { .. } => "schema-resource-not-found",
            Self::SchemaFileReadFailed { .. } => "schema-file-read-failed",
            Self::SchemaFileParseFailed { .. } => "schema-file-parse-failed",
            Self::SchemaFetchFailed { .. } => "schema-fetch-failed",
            Self::UnsupportedSourceUrl { .. } => "unsupported-source-url",
            Self::SourceUrlParseFailed { .. } => "source-url-parse-failed",
            Self::InvalidFilePath { .. } => "invalid-file-path",
            Self::InvalidJsonFormat { .. } => "invalid-json-format",
            Self::InvalidJsonPointer { .. } => "invalid-json-pointer",
            Self::InvalidJsonSchemaReference { .. } => "invalid-json-schema-reference",
            Self::UnsupportedReference { .. } => "unsupported-reference",
            Self::UnsupportedUrlScheme { .. } => "unsupported-url-scheme",
            Self::SchemaMustBeObject { .. } => "schema-must-be-object",
            Self::CacheError(error) => error.code(),
        }
    }
}
