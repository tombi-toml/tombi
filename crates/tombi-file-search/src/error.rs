use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0:?} file not found")]
    FileNotFound(PathBuf),

    #[error(transparent)]
    GlobSearchFailed(tombi_glob::Error),
}
