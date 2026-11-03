#[derive(thiserror::Error, Debug)]
pub enum DiagnosticKind {
    #[error("defining dotted keys out-of-order is discouraged")]
    DottedKeysOutOfOrder,
    #[error("defining tables out-of-order is discouraged")]
    TablesOutOfOrder,
    #[error("trailing comma after key-value is not allowed")]
    ForbiddenKeyValueTrailingComma,
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
    pub span: tombi_text::Span,
}

impl Diagnostic {
    pub fn code(&self) -> &'static str {
        match self.kind {
            DiagnosticKind::DottedKeysOutOfOrder => "discouraged/dotted-keys-out-of-order",
            DiagnosticKind::TablesOutOfOrder => "discouraged/tables-out-of-order",
            DiagnosticKind::ForbiddenKeyValueTrailingComma => {
                "syntax/forbidden-key-value-trailing-comma"
            }
            DiagnosticKind::InlineTableMustSingleLine => "syntax/inline-table-must-single-line",
            DiagnosticKind::ForbiddenInlineTableLastComma => {
                "syntax/forbidden-inline-table-last-comma"
            }
            DiagnosticKind::MissingArrayComma => "syntax/missing-array-comma",
            DiagnosticKind::MissingInlineTableComma => "syntax/missing-inline-table-comma",
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
                    self.span,
                ));
            }
            tombi_config::SeverityLevel::Warn => {
                diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
                    self.kind.to_string(),
                    self.code(),
                    self.span,
                ));
            }
            tombi_config::SeverityLevel::Off => {}
        }
    }
}
