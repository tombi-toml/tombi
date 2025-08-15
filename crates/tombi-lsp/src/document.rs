#[derive(Debug, Clone)]
pub struct DocumentSource {
    /// The text of the document.
    pub text: String,

    /// The version of the document.
    ///
    /// If the file has never been opened in the editor, None will be entered.
    pub version: Option<i32>,
}

impl DocumentSource {
    pub fn new(text: impl Into<String>, version: Option<i32>) -> Self {
        Self {
            text: text.into(),
            version,
        }
    }
}
