use crate::BoolDefaultTrue;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspOptions {
    /// # Code Action Feature options
    pub code_action: Option<LspCodeAction>,

    /// # Completion Feature options
    pub completion: Option<LspCompletion>,

    /// # Diagnostic Feature options
    diagnostic: Option<LspDiagnostic>,

    /// # Diagnostic Feature options
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `lsp.diagnostic` instead.
    #[cfg_attr(feature = "jsonschema", deprecated)]
    diagnostics: Option<LspDiagnostic>,

    /// # Document Link Feature options
    pub document_link: Option<LspDocumentLink>,

    /// # Formatting Feature options
    pub formatting: Option<LspFormatting>,

    /// # Goto Declaration Feature options
    pub goto_declaration: Option<LspGotoDefinition>,

    /// # Goto Definition Feature options
    pub goto_definition: Option<LspGotoDefinition>,

    /// # Goto Type Definition Feature options
    pub goto_type_definition: Option<LspGotoDefinition>,

    /// # Hover Feature options
    pub hover: Option<LspHover>,

    /// # Workspace Diagnostics Feature options
    pub workspace_diagnostic: Option<LspWorkspaceDiagnostic>,
}

impl LspOptions {
    pub const fn default() -> Self {
        Self {
            code_action: None,
            completion: None,
            diagnostic: None,
            #[allow(deprecated)]
            diagnostics: None,
            document_link: None,
            formatting: None,
            goto_declaration: None,
            goto_definition: None,
            goto_type_definition: None,
            hover: None,
            workspace_diagnostic: None,
        }
    }

    pub fn diagnostic(&self) -> Option<&LspDiagnostic> {
        self.diagnostic.as_ref().or({
            #[allow(deprecated)]
            self.diagnostics.as_ref()
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspHover {
    /// # Enable hover feature
    ///
    /// Whether to enable hover.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspCodeAction {
    /// # Enable code action feature
    ///
    /// Whether to enable code action.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspCompletion {
    /// # Enable completion feature
    ///
    /// Whether to enable completion.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspFormatting {
    /// # Enable formatting feature
    ///
    /// Whether to enable formatting.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspDiagnostic {
    /// # Enable diagnostic feature
    ///
    /// Whether to enable diagnostic.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspDocumentLink {
    /// # Enable document link feature
    ///
    /// Whether to enable document link.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspGotoDefinition {
    /// # Enable goto definition feature
    ///
    /// Whether to enable goto definition.
    pub enabled: Option<BoolDefaultTrue>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspWorkspaceDiagnostic {
    /// # Enable workspace diagnostic feature
    ///
    /// Whether to enable workspace diagnostic.
    pub enabled: Option<BoolDefaultTrue>,

    /// # Throttle interval in seconds
    ///
    /// Controls the throttling behavior of workspace diagnostics:
    /// - 0: Run only once (first execution), then always skip
    /// - >0: Skip if within the specified interval, allow if interval has passed
    ///
    /// Default: Follows the editor's execution frequency (no additional throttling).
    ///
    pub throttle_seconds: Option<u64>,

    /// # Enable file watcher for workspace diagnostics
    ///
    /// Whether to enable file system watching for workspace diagnostics.
    /// When enabled, only changed files are diagnosed instead of full scans.
    ///
    /// Default: true
    pub file_watcher_enabled: Option<BoolDefaultTrue>,

    /// # File watcher debounce interval in milliseconds
    ///
    /// Time to wait before processing file system events to batch rapid changes.
    /// Higher values reduce CPU usage but increase diagnostic latency.
    ///
    /// Default: 100
    pub file_watcher_debounce_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_workspace_diagnostic_default() {
        let workspace_diagnostic = LspWorkspaceDiagnostic::default();
        assert_eq!(workspace_diagnostic.enabled, None);
        assert_eq!(workspace_diagnostic.throttle_seconds, None);
        assert_eq!(workspace_diagnostic.file_watcher_enabled, None);
        assert_eq!(workspace_diagnostic.file_watcher_debounce_ms, None);
    }

    #[test]
    fn test_lsp_workspace_diagnostic_with_file_watcher_fields() {
        let workspace_diagnostic = LspWorkspaceDiagnostic {
            enabled: Some(BoolDefaultTrue(true)),
            throttle_seconds: Some(5),
            file_watcher_enabled: Some(BoolDefaultTrue(true)),
            file_watcher_debounce_ms: Some(150),
        };

        assert_eq!(workspace_diagnostic.enabled.map(|v| v.value()), Some(true));
        assert_eq!(workspace_diagnostic.throttle_seconds, Some(5));
        assert_eq!(workspace_diagnostic.file_watcher_enabled.map(|v| v.value()), Some(true));
        assert_eq!(workspace_diagnostic.file_watcher_debounce_ms, Some(150));
    }

    #[test]
    fn test_lsp_workspace_diagnostic_file_watcher_only() {
        let workspace_diagnostic = LspWorkspaceDiagnostic {
            enabled: None,
            throttle_seconds: None,
            file_watcher_enabled: Some(BoolDefaultTrue(false)),
            file_watcher_debounce_ms: Some(200),
        };

        assert_eq!(workspace_diagnostic.file_watcher_enabled.map(|v| v.value()), Some(false));
        assert_eq!(workspace_diagnostic.file_watcher_debounce_ms, Some(200));
    }
}
