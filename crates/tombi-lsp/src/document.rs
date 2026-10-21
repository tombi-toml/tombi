use tombi_document_snapshot::DocumentSnapshot;
pub use tombi_document_snapshot::ParsedText;
use tombi_text::EncodingKind;

/// A [`DocumentSnapshot`] of an open document, with what the language server knows about it.
///
/// A clone shares the snapshot, so it is cheap to hand it to a handler.
#[derive(Clone, Debug)]
pub struct DocumentSource {
    snapshot: DocumentSnapshot,

    /// The column unit of the LSP client.
    encoding_kind: EncodingKind,

    /// The version of the document.
    ///
    /// If the file has never been opened in the editor, None will be entered.
    pub version: Option<i32>,
}

impl std::ops::Deref for DocumentSource {
    type Target = DocumentSnapshot;

    fn deref(&self) -> &DocumentSnapshot {
        &self.snapshot
    }
}

impl DocumentSource {
    pub fn new(
        parsed: ParsedText,
        version: Option<i32>,
        toml_version: tombi_config::TomlVersion,
        encoding_kind: EncodingKind,
    ) -> Self {
        Self {
            snapshot: DocumentSnapshot::new(parsed, toml_version),
            encoding_kind,
            version,
        }
    }

    /// A snapshot of `text`, with the version and the encoding of this one.
    pub fn with_text(
        &self,
        text: impl Into<Box<str>>,
        toml_version: tombi_config::TomlVersion,
    ) -> Self {
        Self::new(
            ParsedText::parse(text),
            self.version,
            toml_version,
            self.encoding_kind,
        )
    }

    pub fn set_text(&mut self, text: impl Into<Box<str>>, toml_version: tombi_config::TomlVersion) {
        *self = self.with_text(text, toml_version);
    }

    /// Rebuilds only the document tree for `toml_version`, reusing the parsed AST.
    pub fn set_toml_version(&mut self, toml_version: tombi_config::TomlVersion) {
        *self = self.with_toml_version(toml_version);
    }

    /// A snapshot that builds only the document tree for `toml_version`, reusing the parsed AST.
    pub fn with_toml_version(&self, toml_version: tombi_config::TomlVersion) -> Self {
        Self {
            snapshot: self.snapshot.with_toml_version(toml_version),
            encoding_kind: self.encoding_kind,
            version: self.version,
        }
    }

    /// The snapshot, to hand it to code that outlives the request.
    pub fn snapshot(&self) -> &DocumentSnapshot {
        &self.snapshot
    }

    /// The column unit of the LSP client, to convert spans into LSP ranges.
    pub fn encoding_kind(&self) -> EncodingKind {
        self.encoding_kind
    }
}
