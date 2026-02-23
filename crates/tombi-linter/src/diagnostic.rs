#[derive(thiserror::Error, Debug)]
pub enum DiagnosticKind {
    #[error("An empty key is discouraged")]
    KeyEmpty,
    #[error("Defining dotted keys out-of-order is discouraged")]
    DottedKeysOutOfOrder,
    #[error("Defining tables out-of-order is discouraged")]
    TablesOutOfOrder,
    #[error("inline table must be single line in TOML v1.0.0 or earlier")]
    InlineTableMustSingleLine,
    #[error("trailing comma in inline table not allowed in TOML v1.0.0 or earlier")]
    ForbiddenInlineTableLastComma,
    #[error("missing ','")]
    MissingArrayComma,
    #[error("missing ','")]
    MissingInlineTableComma,
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
            DiagnosticKind::InlineTableMustSingleLine => "inline-table-must-single-line",
            DiagnosticKind::ForbiddenInlineTableLastComma => "forbidden-inline-table-last-comma",
            DiagnosticKind::MissingArrayComma => "missing-array-comma",
            DiagnosticKind::MissingInlineTableComma => "missing-inline-table-comma",
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
