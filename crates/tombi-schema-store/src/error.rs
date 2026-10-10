use std::path::PathBuf;

use crate::{CatalogUri, SchemaUri};

pub const SCHEMA_RESOLUTION_DIAGNOSTIC_CODE: &str = "schema-resolution";

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

    #[error(
        "schema host is not trusted: {schema_uri}; it resolves to a non-public address, \
         add the host to TOMBI_SCHEMA_TRUSTED_HOSTS"
    )]
    SchemaHostNotTrusted { schema_uri: SchemaUri },

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

    #[error("schema must be an object or boolean: {schema_uri}")]
    SchemaMustBeObjectOrBoolean { schema_uri: SchemaUri },

    #[error(transparent)]
    DuplicateSchemaResourceInDocument(Box<DuplicateSchemaResourceInDocument>),

    #[error(transparent)]
    DuplicateSchemaResourceAcrossDocuments(Box<DuplicateSchemaResourceAcrossDocuments>),

    #[error(transparent)]
    CacheError(#[from] tombi_cache::Error),
}

#[derive(Debug, Clone, thiserror::Error)]
#[error(
    "duplicate $id `{schema_uri}` in {document}: first at `{first_location}`, again at `{second_location}`",
    document = format_schema_document(schema_document_uri)
)]
pub struct DuplicateSchemaResourceInDocument {
    pub schema_uri: SchemaUri,
    pub schema_document_uri: SchemaUri,
    pub first_location: String,
    pub second_location: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error(
    "duplicate $id `{schema_uri}`: already defined in {existing_document} at `{existing_location}`, also claimed by {conflicting_document} at `{conflicting_location}`",
    existing_document = format_schema_document(existing_schema_document_uri),
    conflicting_document = format_schema_document(conflicting_schema_document_uri)
)]
pub struct DuplicateSchemaResourceAcrossDocuments {
    pub schema_uri: SchemaUri,
    pub existing_schema_document_uri: SchemaUri,
    pub existing_location: String,
    pub conflicting_schema_document_uri: SchemaUri,
    pub conflicting_location: String,
}

impl Error {
    /// Convert a failed schema fetch (or policy refusal) for `schema_uri`.
    pub(crate) fn from_schema_fetch_error(
        schema_uri: &SchemaUri,
        error: crate::FetchError,
    ) -> Self {
        match error {
            crate::FetchError::HostNotTrusted => Self::SchemaHostNotTrusted {
                schema_uri: schema_uri.clone(),
            },
            error => Self::SchemaFetchFailed {
                schema_uri: schema_uri.clone(),
                reason: error.to_string(),
            },
        }
    }

    #[inline]
    pub fn to_warning_diagnostic(&self, span: tombi_text::Span) -> tombi_diagnostic::Diagnostic {
        tombi_diagnostic::Diagnostic::new_warning(
            self.to_string(),
            SCHEMA_RESOLUTION_DIAGNOSTIC_CODE,
            span,
        )
    }
}

pub(crate) fn format_schema_document(uri: &SchemaUri) -> String {
    if uri.scheme() == "file"
        && let Ok(path) = uri.to_file_path()
        && let Some(file_name) = path.file_name()
    {
        return file_name.to_string_lossy().into_owned();
    }
    uri.to_string()
}
