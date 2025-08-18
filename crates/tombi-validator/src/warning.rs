use tombi_schema_store::{SchemaAccessors, SchemaUri};

#[derive(thiserror::Error, Debug)]
pub enum WarningKind {
    #[error("`{0}` is deprecated")]
    Deprecated(SchemaAccessors),

    #[error(
        r#"In strict mode, `{accessors}` does not allow "{key}" key.
Please add `"additionalProperties": true` to the location where `{accessors}` is defined in {schema_uri},
or add `#:tombi schema.strict = false` as a document comment directive at the top of your document,
or set `schema.strict = false` in your `tombi.toml`."#
    )]
    StrictAdditionalProperties {
        accessors: SchemaAccessors,
        key: String,
        schema_uri: SchemaUri,
    },
}

#[derive(Debug)]
pub struct Warning {
    pub kind: Box<WarningKind>,
    pub range: tombi_text::Range,
}

impl Warning {
    #[inline]
    pub fn code(&self) -> &'static str {
        match *self.kind {
            WarningKind::Deprecated { .. } => "deprecated",
            WarningKind::StrictAdditionalProperties { .. } => "strict-additional-properties",
        }
    }
}

impl tombi_diagnostic::SetDiagnostics for Warning {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
            self.kind.to_string(),
            self.code(),
            self.range,
        ))
    }
}
