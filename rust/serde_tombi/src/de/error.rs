use itertools::Itertools;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(Box<InnerError>);

#[derive(Debug, thiserror::Error)]
enum InnerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Config(#[from] tombi_config::Error),

    #[error(transparent)]
    SchemaStore(#[from] tombi_schema_store::Error),

    #[error(transparent)]
    DocumentDeserialize(#[from] tombi_document::de::Error),

    #[error("{}", .0.iter().map(|e| e.to_string()).collect_vec().join(", "))]
    Parser(Vec<tombi_parser::Error>),

    #[error("{}", .0.iter().map(|e| e.to_string()).collect_vec().join(", "))]
    DocumentTree(Vec<tombi_document_tree::Error>),
}

impl From<InnerError> for Error {
    fn from(error: InnerError) -> Self {
        Self(Box::new(error))
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self(Box::new(InnerError::Io(error)))
    }
}

impl From<tombi_config::Error> for Error {
    fn from(error: tombi_config::Error) -> Self {
        Self(Box::new(InnerError::Config(error)))
    }
}

impl From<Vec<tombi_parser::Error>> for Error {
    fn from(errors: Vec<tombi_parser::Error>) -> Self {
        Self(Box::new(InnerError::Parser(errors)))
    }
}

impl From<Vec<tombi_document_tree::Error>> for Error {
    fn from(errors: Vec<tombi_document_tree::Error>) -> Self {
        Self(Box::new(InnerError::DocumentTree(errors)))
    }
}

impl From<tombi_schema_store::Error> for Error {
    fn from(error: tombi_schema_store::Error) -> Self {
        Self(Box::new(InnerError::SchemaStore(error)))
    }
}

impl From<tombi_document::de::Error> for Error {
    fn from(error: tombi_document::de::Error) -> Self {
        Self(Box::new(InnerError::DocumentDeserialize(error)))
    }
}
