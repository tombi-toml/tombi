use super::{EnabledOnly, ToggleFeature};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoExtensionFeatures {
    Enabled(EnabledOnly),
    Features(CargoExtensionFeatureTree),
}

default_to_features!(CargoExtensionFeatures, CargoExtensionFeatureTree);

impl CargoExtensionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn lsp(&self) -> Option<&CargoLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
        }
    }

    pub fn lsp_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, CargoLspFeatures::enabled)
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::completion_enabled)
    }

    pub fn dependency_version_completion_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::dependency_version_completion_enabled,
            )
    }

    pub fn dependency_feature_completion_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::dependency_feature_completion_enabled,
            )
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::path_completion_enabled)
    }

    pub fn inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::inlay_hint_enabled)
    }

    pub fn dependency_version_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::dependency_version_inlay_hint_enabled,
            )
    }

    pub fn default_features_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::default_features_inlay_hint_enabled)
    }

    pub fn workspace_value_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::workspace_value_inlay_hint_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::document_link_enabled)
    }

    pub fn cargo_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::cargo_toml_document_link_enabled)
    }

    pub fn workspace_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::workspace_document_link_enabled)
    }

    pub fn git_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::git_document_link_enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::path_document_link_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, CargoLspFeatures::hover_enabled)
    }

    pub fn dependency_detail_hover_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::dependency_detail_hover_enabled)
    }

    pub fn crates_io_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::crates_io_document_link_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::code_action_enabled)
    }

    pub fn inherit_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::inherit_from_workspace_code_action_enabled,
            )
    }

    pub fn inherit_dependency_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::inherit_dependency_from_workspace_code_action_enabled,
            )
    }

    pub fn convert_dependency_to_table_format_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::convert_dependency_to_table_format_code_action_enabled,
            )
    }

    pub fn add_to_workspace_and_inherit_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::add_to_workspace_and_inherit_dependency_code_action_enabled,
            )
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
pub struct CargoExtensionFeatureTree {
    /// # Cargo LSP feature options
    pub lsp: Option<CargoLspFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoLspFeatures {
    Enabled(EnabledOnly),
    Features(CargoLspFeatureTree),
}

default_to_features!(CargoLspFeatures, CargoLspFeatureTree);

impl CargoLspFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn completion(&self) -> Option<&CargoCompletionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.completion.as_ref(),
        }
    }

    pub fn inlay_hint(&self) -> Option<&CargoInlayHintFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.inlay_hint.as_ref(),
        }
    }

    pub fn goto_definition(&self) -> Option<&CargoNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_definition.as_ref(),
        }
    }

    pub fn goto_declaration(&self) -> Option<&CargoNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_declaration.as_ref(),
        }
    }

    pub fn document_link(&self) -> Option<&CargoDocumentLinkFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.document_link.as_ref(),
        }
    }

    pub fn code_action(&self) -> Option<&CargoCodeActionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.code_action.as_ref(),
        }
    }

    pub fn hover(&self) -> Option<&CargoHoverFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.hover.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::enabled)
    }

    pub fn dependency_version_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::dependency_version_enabled)
    }

    pub fn dependency_feature_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::dependency_feature_enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::path_enabled)
    }

    pub fn inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::enabled)
    }

    pub fn dependency_version_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::dependency_version_enabled)
    }

    pub fn default_features_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::default_features_enabled)
    }

    pub fn workspace_value_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::workspace_value_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::enabled)
    }

    pub fn cargo_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::cargo_toml_enabled)
    }

    pub fn workspace_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::workspace_enabled)
    }

    pub fn git_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::git_enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::path_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.hover().map_or(true, CargoHoverFeatures::enabled)
    }

    pub fn dependency_detail_hover_enabled(&self) -> bool {
        self.enabled()
            && self
                .hover()
                .map_or(true, CargoHoverFeatures::dependency_detail_enabled)
    }

    pub fn crates_io_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::crates_io_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .code_action()
                .map_or(true, CargoCodeActionFeatures::enabled)
    }

    pub fn inherit_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::inherit_from_workspace_enabled,
            )
    }

    pub fn inherit_dependency_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::inherit_dependency_from_workspace_enabled,
            )
    }

    pub fn convert_dependency_to_table_format_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::convert_dependency_to_table_format_enabled,
            )
    }

    pub fn add_to_workspace_and_inherit_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::add_to_workspace_and_inherit_dependency_enabled,
            )
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
pub struct CargoLspFeatureTree {
    /// # Completion feature options
    pub completion: Option<CargoCompletionFeatures>,

    /// # Inlay hint feature options
    pub inlay_hint: Option<CargoInlayHintFeatures>,

    /// # Goto definition feature options
    pub goto_definition: Option<CargoNavigationFeatures>,

    /// # Goto declaration feature options
    pub goto_declaration: Option<CargoNavigationFeatures>,

    /// # Document link feature options
    pub document_link: Option<CargoDocumentLinkFeatures>,

    /// # Hover feature options
    pub hover: Option<CargoHoverFeatures>,

    /// # Code action feature options
    pub code_action: Option<CargoCodeActionFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub enum CargoHoverFeatures {
    Enabled(EnabledOnly),
    Features(CargoHoverFeatureTree),
}

default_to_features!(CargoHoverFeatures, CargoHoverFeatureTree);

impl CargoHoverFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_detail_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_detail
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub enum CargoInlayHintFeatures {
    Enabled(EnabledOnly),
    Features(CargoInlayHintFeatureTree),
}

default_to_features!(CargoInlayHintFeatures, CargoInlayHintFeatureTree);

impl CargoInlayHintFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_version_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_version
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn default_features_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .default_features
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn workspace_value_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .workspace_value
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
pub struct CargoHoverFeatureTree {
    /// # Dependency detail hover feature
    ///
    /// Whether hover shows detailed dependency metadata.
    pub dependency_detail: Option<ToggleFeature>,
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
pub struct CargoInlayHintFeatureTree {
    /// # Dependency version inlay hint feature
    ///
    /// Whether inlay hints show dependency versions.
    pub dependency_version: Option<ToggleFeature>,

    /// # Default features inlay hint feature
    ///
    /// Whether inlay hints show `default-features` values.
    pub default_features: Option<ToggleFeature>,

    /// # Workspace value inlay hint feature
    ///
    /// Whether inlay hints show values inherited from the Cargo workspace.
    pub workspace_value: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoCompletionFeatures {
    Enabled(EnabledOnly),
    Features(CargoCompletionFeatureTree),
}

default_to_features!(CargoCompletionFeatures, CargoCompletionFeatureTree);

impl CargoCompletionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_version_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_version
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn dependency_feature_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_feature
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.path.as_ref().map_or(true, ToggleFeature::enabled)
                }
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
pub struct CargoCompletionFeatureTree {
    /// # Dependency version completion feature
    ///
    /// Whether completion suggests dependency versions.
    pub dependency_version: Option<ToggleFeature>,

    /// # Dependency feature completion feature
    ///
    /// Whether completion suggests dependency features.
    pub dependency_feature: Option<ToggleFeature>,

    /// # Path completion feature
    ///
    /// Whether completion suggests filesystem paths.
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoNavigationFeatures {
    Enabled(EnabledOnly),
    Features(CargoNavigationFeatureTree),
}

default_to_features!(CargoNavigationFeatures, CargoNavigationFeatureTree);

impl CargoNavigationFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn member_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .member
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.path.as_ref().map_or(true, ToggleFeature::enabled)
                }
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
pub struct CargoNavigationFeatureTree {
    /// # Dependency navigation feature
    ///
    /// Whether navigation resolves dependency definitions and declarations.
    pub dependency: Option<ToggleFeature>,

    /// # Member navigation feature
    ///
    /// Whether navigation resolves workspace member targets.
    pub member: Option<ToggleFeature>,

    /// # Path navigation feature
    ///
    /// Whether navigation resolves filesystem paths.
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(CargoDocumentLinkFeatureTree),
}

default_to_features!(CargoDocumentLinkFeatures, CargoDocumentLinkFeatureTree);

impl CargoDocumentLinkFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn cargo_toml_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .cargo_toml
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn workspace_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .workspace
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn git_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.git.as_ref().map_or(true, ToggleFeature::enabled)
                }
            }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.path.as_ref().map_or(true, ToggleFeature::enabled)
                }
            }
    }

    pub fn crates_io_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .crates_io
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
pub struct CargoDocumentLinkFeatureTree {
    /// # Cargo.toml document link feature
    ///
    /// Whether document links are created for `Cargo.toml` references.
    pub cargo_toml: Option<ToggleFeature>,

    /// # crates.io document link feature
    ///
    /// Whether document links are created for crates.io package references.
    pub crates_io: Option<ToggleFeature>,

    /// # Git document link feature
    ///
    /// Whether document links are created for Git references.
    pub git: Option<ToggleFeature>,

    /// # Path document link feature
    ///
    /// Whether document links are created for filesystem paths.
    pub path: Option<ToggleFeature>,

    /// # Workspace document link feature
    ///
    /// Whether document links are created for `workspace = true` references.
    pub workspace: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoCodeActionFeatures {
    Enabled(EnabledOnly),
    Features(CargoCodeActionFeatureTree),
}

default_to_features!(CargoCodeActionFeatures, CargoCodeActionFeatureTree);

impl CargoCodeActionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn inherit_from_workspace_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .inherit_from_workspace
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn inherit_dependency_from_workspace_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .inherit_dependency_from_workspace
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn convert_dependency_to_table_format_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .convert_dependency_to_table_format
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn add_to_workspace_and_inherit_dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .add_to_workspace_and_inherit_dependency
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
pub struct CargoCodeActionFeatureTree {
    /// # Inherit-from-workspace code action feature
    ///
    /// Whether code actions can replace a value with `workspace = true`.
    pub inherit_from_workspace: Option<ToggleFeature>,

    /// # Inherit-dependency-from-workspace code action feature
    ///
    /// Whether code actions can inherit dependency settings from the workspace.
    pub inherit_dependency_from_workspace: Option<ToggleFeature>,

    /// # Convert-dependency-to-table-format code action feature
    ///
    /// Whether code actions can rewrite inline dependencies to table format.
    pub convert_dependency_to_table_format: Option<ToggleFeature>,

    /// # Add-to-workspace-and-inherit-dependency code action feature
    ///
    /// Whether code actions can add a dependency to the workspace and inherit it.
    pub add_to_workspace_and_inherit_dependency: Option<ToggleFeature>,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::BoolDefaultTrue;

    use super::{CargoDocumentLinkFeatureTree, CargoInlayHintFeatureTree, ToggleFeature};

    #[test]
    fn cargo_inlay_hint_feature_tree_deserializes_workspace_value_key() {
        let features: CargoInlayHintFeatureTree = serde_json::from_value(serde_json::json!({
            "workspace-value": {
                "enabled": false
            }
        }))
        .expect("workspace-value should deserialize");

        assert_eq!(
            features.workspace_value,
            Some(ToggleFeature {
                enabled: Some(BoolDefaultTrue::from(false)),
            })
        );
    }

    #[test]
    fn cargo_inlay_hint_feature_tree_serializes_workspace_value_key() {
        let value = serde_json::to_value(CargoInlayHintFeatureTree {
            dependency_version: None,
            default_features: None,
            workspace_value: Some(ToggleFeature {
                enabled: Some(BoolDefaultTrue::from(false)),
            }),
        })
        .expect("workspace-value should serialize");

        assert_eq!(
            value.get("workspace-value"),
            Some(&serde_json::json!({
                "enabled": false
            }))
        );
        assert!(value.get("workspace").is_none());
    }

    #[test]
    fn cargo_document_link_feature_tree_deserializes_workspace_key() {
        let features: CargoDocumentLinkFeatureTree = serde_json::from_value(serde_json::json!({
            "workspace": {
                "enabled": false
            }
        }))
        .expect("workspace should deserialize");

        assert_eq!(
            features.workspace,
            Some(ToggleFeature {
                enabled: Some(BoolDefaultTrue::from(false)),
            })
        );
    }

    #[test]
    fn cargo_document_link_feature_tree_serializes_workspace_key() {
        let value = serde_json::to_value(CargoDocumentLinkFeatureTree {
            cargo_toml: None,
            workspace: Some(ToggleFeature {
                enabled: Some(BoolDefaultTrue::from(false)),
            }),
            git: None,
            path: None,
            crates_io: None,
        })
        .expect("workspace should serialize");

        assert_eq!(
            value.get("workspace"),
            Some(&serde_json::json!({
                "enabled": false
            }))
        );
    }
}
