#[derive(thiserror::Error, Debug)]
pub enum DiagnosticKind {
    #[error("An empty key is discouraged")]
    KeyEmpty,
    #[error("Defining dotted keys out-of-order is discouraged")]
    DottedKeysOutOfOrder,
    #[error("Defining tables out-of-order is discouraged")]
    TablesOutOfOrder,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub level: tombi_config::SeverityLevel,
    pub range: tombi_text::Range,
}

impl Diagnostic {
    pub fn code(&self) -> &'static str {
        match self.kind {
            DiagnosticKind::KeyEmpty => "key-empty",
            DiagnosticKind::DottedKeysOutOfOrder => "dotted-keys-out-of-order",
            DiagnosticKind::TablesOutOfOrder => "tables-out-of-order",
        }
    }
}

impl tombi_diagnostic::SetDiagnostics for Diagnostic {
    fn set_diagnostics(self, diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>) {
        match self.level {
            tombi_config::SeverityLevel::Error => {
                diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                    self.kind.to_string(),
                    self.code(),
                    self.range,
                ));
            }
            tombi_config::SeverityLevel::Warn => {
                diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
                    self.kind.to_string(),
                    self.code(),
                    self.range,
                ));
            }
            tombi_config::SeverityLevel::Off => {}
        }
    }
}
