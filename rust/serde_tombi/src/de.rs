use ast::AstNode;
use document::IntoDocument;
use document_tree::IntoDocumentTreeAndErrors;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use toml_version::TomlVersion;

/// Deserialize a TOML string into a Rust data structure.
///
/// # Note
///
/// This function is not yet implemented and will return an error.
/// The example below shows the expected usage once implemented.
///
/// # Examples
///
/// ```no_run
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     ip: String,
///     port: u16,
///     keys: Vec<String>,
/// }
///
/// let toml = r#"
/// ip = "127.0.0.1"
/// port = 8080
/// keys = ["key1", "key2"]
/// "#;
///
/// let config: Config = serde_tombi::from_str(toml).unwrap();
/// ```
pub fn from_str<T>(s: &str) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let document = parse_str(s)?;
    from_document(document)
}

/// Parse a TOML string into a Document.
pub fn parse_str(s: &str) -> crate::Result<document::Document> {
    // Parse the source string using the parser
    let parsed = parser::parse(s);

    let errors = parsed.errors(TomlVersion::default()).collect_vec();
    // Check if there are any parsing errors
    if !errors.is_empty() {
        return Err(crate::Error::Parser(
            errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    // Cast the parsed result to an AST Root node
    let root = ast::Root::cast(parsed.into_syntax_node())
        .ok_or_else(|| crate::Error::Parser("Failed to cast to AST Root".to_string()))?;

    // Convert the AST to a document tree
    let (document_tree, errors) = root
        .into_document_tree_and_errors(TomlVersion::default())
        .into();

    // Check for errors during document tree construction
    if !errors.is_empty() {
        return Err(crate::Error::DocumentTree(
            errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    // Convert to a Document
    Ok(document_tree.into_document(TomlVersion::default()))
}

/// Deserialize a Document into a Rust data structure.
pub fn from_document<T>(_document: document::Document) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    // Implementation not yet available
    Err(crate::Error::Deserialization(
        "Not implemented yet".to_string(),
    ))
}
