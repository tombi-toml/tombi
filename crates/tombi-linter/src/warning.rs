#[derive(thiserror::Error, Debug)]
pub enum WarningKind {
    #[error("An empty quoted key is allowed, but it is not recommended")]
    KeyEmpty,
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
