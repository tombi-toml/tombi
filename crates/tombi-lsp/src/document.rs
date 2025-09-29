use tombi_text::{LineIndex, WideEncoding};

#[derive(Debug, Clone)]
pub struct DocumentSource {
    /// The text of the document.
    text: String,

    line_index: LineIndex<'static>,

    /// The version of the document.
    ///
    /// If the file has never been opened in the editor, None will be entered.
    pub version: Option<i32>,
}

impl DocumentSource {
    pub fn new(text: impl Into<String>, version: Option<i32>, wide_encoding: WideEncoding) -> Self {
        let text = text.into();
        let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(text.as_str()) };

        Self {
            text,
            line_index: LineIndex::new(text_ref, wide_encoding),
            version,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(self.text.as_str()) };
        self.line_index = LineIndex::new(text_ref, self.line_index.wide_encoding);
    }

    pub fn line_index(&self) -> &LineIndex<'static> {
        &self.line_index
    }
}
