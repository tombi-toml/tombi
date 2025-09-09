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
}
