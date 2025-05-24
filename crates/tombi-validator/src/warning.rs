#[derive(thiserror::Error, Debug)]
pub enum WarningKind {
    #[error("`{0}` is deprecated")]
    Deprecated(tombi_schema_store::SchemaAccessors),

    #[error("\"{key}\" is not allowed; In strict mode, the JSON schema must be explicitly set to `\"additionalProperties\": true`. ")]
    StrictAdditionalProperties { key: String },
}

#[derive(Debug)]
pub struct Warning {
    pub kind: WarningKind,
    pub range: tombi_text::Range,
}

impl tombi_diagnostic::SetDiagnostics for Warning {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
            self.kind.to_string(),
            self.range,
        ))
    }
}
