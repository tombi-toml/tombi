use std::path::PathBuf;

use crate::{CatalogUri, SchemaUri};

pub const SCHEMA_RESOLUTION_DIAGNOSTIC_CODE: &str = "jsonschema/schema-resolution";

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

    /// The level to log the error at, when the store goes on without the schema.
    ///
    /// A schema that the user pointed at with a local path or URI, and that cannot be used, is
    /// an error: the result is not what the user asked for, and only the user can fix it.
    /// Anything else, such as a fetch that failed or a schema of someone else that is broken,
    /// is a warning: the result is degraded, but it may change when the network is back.
    pub fn log_level(&self) -> log::Level {
        match self {
            Self::InvalidSchemaUri { .. }
            | Self::InvalidSchemaUriOrFilePath { .. }
            | Self::SchemaFileNotFound { .. }
            | Self::SchemaFileReadFailed { .. } => log::Level::Error,
            Self::SchemaFileParseFailed { schema_uri, .. } if schema_uri.scheme() == "file" => {
                log::Level::Error
            }
            _ => log::Level::Warn,
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    macro_rules! test_log_level {
        (#[test] fn $name:ident($error:expr) -> $level:ident) => {
            #[test]
            fn $name() {
                pretty_assertions::assert_eq!($error.log_level(), log::Level::$level);
            }
        };
    }

    fn uri(uri: &str) -> SchemaUri {
        SchemaUri::from_str(uri).unwrap()
    }

    test_log_level! {
        #[test]
        fn missing_local_schema_is_an_error(Error::SchemaFileNotFound {
            schema_path: PathBuf::from("schema.json"),
        }) -> Error
    }

    test_log_level! {
        #[test]
        fn invalid_schema_uri_is_an_error(Error::InvalidSchemaUri {
            schema_uri: "::".to_string(),
        }) -> Error
    }

    test_log_level! {
        #[test]
        fn broken_local_schema_is_an_error(Error::SchemaFileParseFailed {
            schema_uri: uri("file:///project/schema.json"),
            reason: "expected value".to_string(),
        }) -> Error
    }

    test_log_level! {
        #[test]
        fn broken_remote_schema_is_a_warning(Error::SchemaFileParseFailed {
            schema_uri: uri("https://example.com/schema.json"),
            reason: "expected value".to_string(),
        }) -> Warn
    }

    test_log_level! {
        #[test]
        fn failed_fetch_is_a_warning(Error::SchemaFetchFailed {
            schema_uri: uri("https://example.com/schema.json"),
            reason: "connection refused".to_string(),
        }) -> Warn
    }

    test_log_level! {
        #[test]
        fn untrusted_host_is_a_warning(Error::SchemaHostNotTrusted {
            schema_uri: uri("https://example.com/schema.json"),
        }) -> Warn
    }
}
