//! Provides serialization functionality for converting Rust types to TOML.
//!
//! # Examples
//!
//! ## Basic usage
//!
//! ```rust
//! use std::collections::HashMap;
//!
//! use serde::Serialize;
//! use tokio;
//!
//! #[derive(Serialize)]
//! struct Package {
//!     name: String,
//!     version: String,
//!     authors: Vec<String>,
//! }
//!
//! #[derive(Serialize)]
//! struct CargoToml {
//!     package: Package,
//!     dependencies: HashMap<String, String>,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let cargo_toml = CargoToml {
//!         package: Package {
//!             name: "serde_tombi".to_string(),
//!             version: "0.1.0".to_string(),
//!             authors: vec!["The Tombi Team".to_string()],
//!         },
//!         dependencies: [
//!             ("serde".to_string(), "1.0".to_string()),
//!             ("thiserror".to_string(), "1.0".to_string()),
//!         ].into_iter().collect(),
//!     };
//!
//!     // Simple serialization
//!     let toml = serde_tombi::to_string_async(&cargo_toml).await.unwrap();
//! }
//! ```
//!
//! ## Using TypedBuilder pattern
//!
//! ```rust
//! use std::collections::HashMap;
//!
//! use serde::Serialize;
//! use serde_tombi::Serializer;
//! use tokio;
//!
//! #[derive(Serialize)]
//! struct Package {
//!     name: String,
//!     version: String,
//!     authors: Vec<String>,
//! }
//!
//! #[derive(Serialize)]
//! struct CargoToml {
//!     package: Package,
//!     dependencies: HashMap<String, String>,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let cargo_toml = CargoToml {
//!         package: Package {
//!             name: "serde_tombi".to_string(),
//!             version: "0.1.0".to_string(),
//!             authors: vec!["The Tombi Team".to_string()],
//!         },
//!         dependencies: [
//!             ("serde".to_string(), "1.0".to_string()),
//!             ("thiserror".to_string(), "1.0".to_string()),
//!         ].into_iter().collect(),
//!     };
//!
//!     // Using either new() or the builder pattern
//!     // Builder pattern:
//!     let serializer = Serializer::builder()
//!         .source_path(std::path::Path::new("Cargo.toml"))
//!         .build();
//!
//!     let toml = serializer.to_string_async(&cargo_toml).await.unwrap();
//! }
//! ```
//!
pub mod config;
mod de;
mod document;
mod ser;

pub use de::{Deserializer, from_document, from_str_async};
pub use document::{
    Array, ArrayKind, Boolean, Document, Float, Integer, IntegerKind, Key, LocalDate,
    LocalDateTime, LocalTime, OffsetDateTime, String, StringKind, Table, TableKind, Value,
};

pub use ser::{Serializer, to_document, to_string_async};
use std::fmt;
use thiserror::Error;

/// Error that can occur when processing TOML.
#[derive(Debug, Error)]
pub enum Error {
    /// Error occurred while parsing the TOML document.
    #[error("Parser error: {0}")]
    Parser(std::string::String),

    /// Error occurred during document tree construction.
    #[error("Document tree error: {0}")]
    DocumentTree(std::string::String),

    /// Error occurred during serialization.
    #[error("Serialization error: {0}")]
    Serialization(std::string::String),

    /// Error occurred during deserialization.
    #[error("Deserialization error: {0}")]
    Deserialization(std::string::String),
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serialization(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Deserialization(msg.to_string())
    }
}

/// A specialized `Result` type for serde_tombi operations.
pub type Result<T> = std::result::Result<T, Error>;
