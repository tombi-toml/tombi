use crate::{BoolDefaultTrue, EnabledOnly, ToggleFeature};

macro_rules! default_to_features {
    ($enum:ident, $features:ident) => {
        impl Default for $enum {
            fn default() -> Self {
                Self::Features($features::default())
            }
        }
    };
}

/// # Language Server options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LspOptions {
    /// # Code Action feature options
    pub code_action: Option<LspCodeAction>,

    /// # Completion feature options
    pub completion: Option<LspCompletion>,

    /// # Diagnostic feature options
    pub diagnostic: Option<LspDiagnostic>,

    /// # Document Link feature options
    pub document_link: Option<LspDocumentLink>,

    /// # Formatting feature options
    pub formatting: Option<LspFormatting>,

    /// # Goto Declaration feature options
    pub goto_declaration: Option<LspGotoDeclaration>,

    /// # Goto Definition feature options
    pub goto_definition: Option<LspGotoDefinition>,

    /// # Goto Type Definition feature options
    pub goto_type_definition: Option<LspGotoTypeDefinition>,

    /// # Hover feature options
    pub hover: Option<LspHover>,

    /// # Workspace Diagnostics feature options
    pub workspace_diagnostic: Option<LspWorkspaceDiagnostic>,
}

impl LspOptions {
    pub fn code_action_enabled(&self) -> bool {
        self.code_action
            .as_ref()
            .map_or(true, LspCodeAction::enabled)
    }

    pub fn code_action_dot_keys_to_inline_table_enabled(&self) -> bool {
        self.code_action
            .as_ref()
            .map_or(true, LspCodeAction::dot_keys_to_inline_table_enabled)
    }

    pub fn code_action_inline_table_to_dot_keys_enabled(&self) -> bool {
        self.code_action
            .as_ref()
            .map_or(true, LspCodeAction::inline_table_to_dot_keys_enabled)
    }

    pub fn code_action_extension_enabled(&self) -> bool {
        self.code_action
            .as_ref()
            .map_or(true, LspCodeAction::extension_enabled)
    }

    pub fn completion_enabled(&self) -> bool {
        self.completion
            .as_ref()
            .map_or(true, LspCompletion::enabled)
    }

    pub fn completion_directive_enabled(&self) -> bool {
        self.completion
            .as_ref()
            .map_or(true, LspCompletion::directive_enabled)
    }

    pub fn completion_schema_enabled(&self) -> bool {
        self.completion
            .as_ref()
            .map_or(true, LspCompletion::schema_enabled)
    }

    pub fn completion_extension_enabled(&self) -> bool {
        self.completion
            .as_ref()
            .map_or(true, LspCompletion::extension_enabled)
    }

    pub fn diagnostic_enabled(&self) -> bool {
        self.diagnostic
            .as_ref()
            .map_or(true, LspDiagnostic::enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.document_link
            .as_ref()
            .map_or(true, LspDocumentLink::enabled)
    }

    pub fn document_link_schema_enabled(&self) -> bool {
        self.document_link
            .as_ref()
            .map_or(true, LspDocumentLink::schema_enabled)
    }

    pub fn document_link_extension_enabled(&self) -> bool {
        self.document_link
            .as_ref()
            .map_or(true, LspDocumentLink::extension_enabled)
    }

    pub fn formatting_enabled(&self) -> bool {
        self.formatting
            .as_ref()
            .map_or(true, LspFormatting::enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.goto_declaration
            .as_ref()
            .map_or(true, LspGotoDeclaration::enabled)
    }

    pub fn goto_declaration_extension_enabled(&self) -> bool {
        self.goto_declaration
            .as_ref()
            .map_or(true, LspGotoDeclaration::extension_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.goto_definition
            .as_ref()
            .map_or(true, LspGotoDefinition::enabled)
    }

    pub fn goto_definition_schema_enabled(&self) -> bool {
        self.goto_definition
            .as_ref()
            .map_or(true, LspGotoDefinition::schema_enabled)
    }

    pub fn goto_definition_extension_enabled(&self) -> bool {
        self.goto_definition
            .as_ref()
            .map_or(true, LspGotoDefinition::extension_enabled)
    }

    pub fn goto_type_definition_enabled(&self) -> bool {
        self.goto_type_definition
            .as_ref()
            .map_or(true, LspGotoTypeDefinition::enabled)
    }

    pub fn goto_type_definition_directive_enabled(&self) -> bool {
        self.goto_type_definition
            .as_ref()
            .map_or(true, LspGotoTypeDefinition::directive_enabled)
    }

    pub fn goto_type_definition_schema_enabled(&self) -> bool {
        self.goto_type_definition
            .as_ref()
            .map_or(true, LspGotoTypeDefinition::schema_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.hover.as_ref().map_or(true, LspHover::enabled)
    }

    pub fn hover_directive_enabled(&self) -> bool {
        self.hover
            .as_ref()
            .map_or(true, LspHover::directive_enabled)
    }

    pub fn hover_schema_enabled(&self) -> bool {
        self.hover.as_ref().map_or(true, LspHover::schema_enabled)
    }

    pub fn hover_extension_enabled(&self) -> bool {
        self.hover
            .as_ref()
            .map_or(true, LspHover::extension_enabled)
    }

    pub fn workspace_diagnostic_enabled(&self) -> bool {
        self.workspace_diagnostic
            .as_ref()
            .map_or(true, LspWorkspaceDiagnostic::enabled)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LspDiagnostic {
    /// # Enable diagnostic feature
    ///
    /// Whether to enable diagnostics.
    pub enabled: Option<BoolDefaultTrue>,
}

impl LspDiagnostic {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LspFormatting {
    /// # Enable formatting feature
    ///
    /// Whether to enable formatting.
    pub enabled: Option<BoolDefaultTrue>,
}

impl LspFormatting {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct LspWorkspaceDiagnostic {
    /// # Enable workspace diagnostic feature
    ///
    /// Whether to enable workspace diagnostics.
    pub enabled: Option<BoolDefaultTrue>,
}

impl LspWorkspaceDiagnostic {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspCompletion {
    Enabled(EnabledOnly),
    Features(LspCompletionFeatureTree),
}

default_to_features!(LspCompletion, LspCompletionFeatureTree);

impl LspCompletion {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn directive_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .directive
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn schema_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .schema
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn extension_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .extension
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspCompletionFeatureTree {
    /// # Directive completion feature
    ///
    /// Whether completion is available in Tombi comment directives.
    pub directive: Option<ToggleFeature>,

    /// # Schema completion feature
    ///
    /// Whether schema-driven completion is enabled.
    pub schema: Option<ToggleFeature>,

    /// # Extension completion feature
    ///
    /// Whether extension-provided completion is enabled.
    pub extension: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspHover {
    Enabled(EnabledOnly),
    Features(LspHoverFeatureTree),
}

default_to_features!(LspHover, LspHoverFeatureTree);

impl LspHover {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn directive_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .directive
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn schema_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .schema
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn extension_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .extension
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspHoverFeatureTree {
    /// # Directive hover feature
    ///
    /// Whether hover is available in Tombi comment directives.
    pub directive: Option<ToggleFeature>,

    /// # Schema hover feature
    ///
    /// Whether schema-derived hover is enabled.
    pub schema: Option<ToggleFeature>,

    /// # Extension hover feature
    ///
    /// Whether extension-provided hover is enabled.
    pub extension: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspDocumentLink {
    Enabled(EnabledOnly),
    Features(LspDocumentLinkFeatureTree),
}

default_to_features!(LspDocumentLink, LspDocumentLinkFeatureTree);

impl LspDocumentLink {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn schema_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .schema
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn extension_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .extension
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspDocumentLinkFeatureTree {
    /// # Schema document link feature
    ///
    /// Whether schema directives produce document links.
    pub schema: Option<ToggleFeature>,

    /// # Extension document link feature
    ///
    /// Whether extension-provided document links are enabled.
    pub extension: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspGotoDefinition {
    Enabled(EnabledOnly),
    Features(LspGotoDefinitionFeatureTree),
}

default_to_features!(LspGotoDefinition, LspGotoDefinitionFeatureTree);

impl LspGotoDefinition {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn schema_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .schema
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn extension_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .extension
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspGotoDefinitionFeatureTree {
    /// # Schema go-to-definition feature
    ///
    /// Whether go-to-definition resolves schema directives.
    pub schema: Option<ToggleFeature>,

    /// # Extension go-to-definition feature
    ///
    /// Whether extension-provided go-to-definition is enabled.
    pub extension: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspGotoDeclaration {
    Enabled(EnabledOnly),
    Features(LspGotoDeclarationFeatureTree),
}

default_to_features!(LspGotoDeclaration, LspGotoDeclarationFeatureTree);

impl LspGotoDeclaration {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn extension_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .extension
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspGotoDeclarationFeatureTree {
    /// # Extension go-to-declaration feature
    ///
    /// Whether extension-provided go-to-declaration is enabled.
    pub extension: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspGotoTypeDefinition {
    Enabled(EnabledOnly),
    Features(LspGotoTypeDefinitionFeatureTree),
}

default_to_features!(LspGotoTypeDefinition, LspGotoTypeDefinitionFeatureTree);

impl LspGotoTypeDefinition {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn directive_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .directive
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn schema_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .schema
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspGotoTypeDefinitionFeatureTree {
    /// # Directive go-to-type-definition feature
    ///
    /// Whether go-to-type-definition resolves Tombi comment directives.
    pub directive: Option<ToggleFeature>,

    /// # Schema go-to-type-definition feature
    ///
    /// Whether go-to-type-definition resolves schema-backed types.
    pub schema: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum LspCodeAction {
    Enabled(EnabledOnly),
    Features(LspCodeActionFeatureTree),
}

default_to_features!(LspCodeAction, LspCodeActionFeatureTree);

impl LspCodeAction {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dot_keys_to_inline_table_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dot_keys_to_inline_table
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn inline_table_to_dot_keys_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .inline_table_to_dot_keys
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn extension_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .extension
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct LspCodeActionFeatureTree {
    /// # Dot-keys to inline-table code action feature
    ///
    /// Whether the code action converting dot-keys to inline tables is enabled.
    pub dot_keys_to_inline_table: Option<ToggleFeature>,

    /// # Inline-table to dot-keys code action feature
    ///
    /// Whether the code action converting inline tables to dot-keys is enabled.
    pub inline_table_to_dot_keys: Option<ToggleFeature>,

    /// # Extension code action feature
    ///
    /// Whether extension-provided code actions are enabled.
    pub extension: Option<ToggleFeature>,
}
