use crate::BoolDefaultTrue;

pub const CARGO_EXTENSION_NAME: &str = "tombi-toml/cargo";
pub const UV_EXTENSION_NAME: &str = "tombi-toml/uv";
pub const TOMBI_EXTENSION_NAME: &str = "tombi-toml/tombi";

macro_rules! default_to_features {
    ($enum:ident, $features:ident) => {
        impl Default for $enum {
            fn default() -> Self {
                Self::Features($features::default())
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct Extensions {
    #[cfg_attr(feature = "serde", serde(rename = "tombi-toml/cargo"))]
    pub cargo: Option<CargoExtensionFeatures>,
    #[cfg_attr(feature = "serde", serde(rename = "tombi-toml/uv"))]
    pub uv: Option<UvExtensionFeatures>,
    #[cfg_attr(feature = "serde", serde(rename = "tombi-toml/tombi"))]
    pub tombi: Option<TombiExtensionFeatures>,
}

impl Extensions {
    pub fn cargo_enabled(&self) -> bool {
        self.cargo
            .as_ref()
            .map_or(true, CargoExtensionFeatures::enabled)
    }

    pub fn cargo_features(&self) -> Option<&CargoExtensionFeatures> {
        self.cargo.as_ref()
    }

    pub fn uv_enabled(&self) -> bool {
        self.uv.as_ref().map_or(true, UvExtensionFeatures::enabled)
    }

    pub fn uv_features(&self) -> Option<&UvExtensionFeatures> {
        self.uv.as_ref()
    }

    pub fn tombi_enabled(&self) -> bool {
        self.tombi
            .as_ref()
            .map_or(true, TombiExtensionFeatures::enabled)
    }

    pub fn tombi_features(&self) -> Option<&TombiExtensionFeatures> {
        self.tombi.as_ref()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct EnabledOnly {
    pub enabled: Option<BoolDefaultTrue>,
}

impl EnabledOnly {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}

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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct CargoExtensionFeatureTree {
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct CargoLspFeatureTree {
    pub completion: Option<CargoCompletionFeatures>,
    pub goto_definition: Option<CargoNavigationFeatures>,
    pub goto_declaration: Option<CargoNavigationFeatures>,
    pub document_link: Option<CargoDocumentLinkFeatures>,
    pub code_action: Option<CargoCodeActionFeatures>,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct CargoCompletionFeatureTree {
    pub dependency_version: Option<ToggleFeature>,
    pub dependency_feature: Option<ToggleFeature>,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct CargoNavigationFeatureTree {
    pub dependency: Option<ToggleFeature>,
    pub member: Option<ToggleFeature>,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct CargoDocumentLinkFeatureTree {
    pub cargo_toml: Option<ToggleFeature>,
    pub git: Option<ToggleFeature>,
    pub path: Option<ToggleFeature>,
    pub crates_io: Option<ToggleFeature>,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct CargoCodeActionFeatureTree {
    pub inherit_from_workspace: Option<ToggleFeature>,
    pub inherit_dependency_from_workspace: Option<ToggleFeature>,
    pub convert_dependency_to_table_format: Option<ToggleFeature>,
    pub add_to_workspace_and_inherit_dependency: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum UvExtensionFeatures {
    Enabled(EnabledOnly),
    Features(UvExtensionFeatureTree),
}

default_to_features!(UvExtensionFeatures, UvExtensionFeatureTree);

impl UvExtensionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn lsp(&self) -> Option<&UvLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, UvLspFeatures::completion_enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::path_completion_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_definition_enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_definition_dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_definition_member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_definition_path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_declaration_enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_declaration_dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_declaration_member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::goto_declaration_path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::document_link_enabled)
    }

    pub fn pyproject_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::pyproject_toml_document_link_enabled)
    }

    pub fn pypi_org_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, UvLspFeatures::pypi_org_document_link_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, UvLspFeatures::code_action_enabled)
    }

    pub fn use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                UvLspFeatures::use_workspace_dependency_code_action_enabled,
            )
    }

    pub fn add_to_workspace_and_use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                UvLspFeatures::add_to_workspace_and_use_workspace_dependency_code_action_enabled,
            )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct UvExtensionFeatureTree {
    pub lsp: Option<UvLspFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum UvLspFeatures {
    Enabled(EnabledOnly),
    Features(UvLspFeatureTree),
}

default_to_features!(UvLspFeatures, UvLspFeatureTree);

impl UvLspFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn completion(&self) -> Option<&UvCompletionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.completion.as_ref(),
        }
    }

    pub fn goto_definition(&self) -> Option<&UvNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_definition.as_ref(),
        }
    }

    pub fn goto_declaration(&self) -> Option<&UvNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_declaration.as_ref(),
        }
    }

    pub fn document_link(&self) -> Option<&UvDocumentLinkFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.document_link.as_ref(),
        }
    }

    pub fn code_action(&self) -> Option<&UvCodeActionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.code_action.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, UvCompletionFeatures::enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, UvCompletionFeatures::path_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, UvNavigationFeatures::enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, UvNavigationFeatures::dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, UvNavigationFeatures::member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, UvNavigationFeatures::path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, UvNavigationFeatures::enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, UvNavigationFeatures::dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, UvNavigationFeatures::member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, UvNavigationFeatures::path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, UvDocumentLinkFeatures::enabled)
    }

    pub fn pyproject_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, UvDocumentLinkFeatures::pyproject_toml_enabled)
    }

    pub fn pypi_org_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, UvDocumentLinkFeatures::pypi_org_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .code_action()
                .map_or(true, UvCodeActionFeatures::enabled)
    }

    pub fn use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .code_action()
                .map_or(true, UvCodeActionFeatures::use_workspace_dependency_enabled)
    }

    pub fn add_to_workspace_and_use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                UvCodeActionFeatures::add_to_workspace_and_use_workspace_dependency_enabled,
            )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct UvLspFeatureTree {
    pub completion: Option<UvCompletionFeatures>,
    pub goto_definition: Option<UvNavigationFeatures>,
    pub goto_declaration: Option<UvNavigationFeatures>,
    pub document_link: Option<UvDocumentLinkFeatures>,
    pub code_action: Option<UvCodeActionFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum UvCompletionFeatures {
    Enabled(EnabledOnly),
    Features(UvCompletionFeatureTree),
}

default_to_features!(UvCompletionFeatures, UvCompletionFeatureTree);

impl UvCompletionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct UvCompletionFeatureTree {
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum UvNavigationFeatures {
    Enabled(EnabledOnly),
    Features(UvNavigationFeatureTree),
}

default_to_features!(UvNavigationFeatures, UvNavigationFeatureTree);

impl UvNavigationFeatures {
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct UvNavigationFeatureTree {
    pub dependency: Option<ToggleFeature>,
    pub member: Option<ToggleFeature>,
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum UvDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(UvDocumentLinkFeatureTree),
}

default_to_features!(UvDocumentLinkFeatures, UvDocumentLinkFeatureTree);

impl UvDocumentLinkFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn pyproject_toml_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .pyproject_toml
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn pypi_org_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .pypi_org
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct UvDocumentLinkFeatureTree {
    pub pyproject_toml: Option<ToggleFeature>,
    pub pypi_org: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum UvCodeActionFeatures {
    Enabled(EnabledOnly),
    Features(UvCodeActionFeatureTree),
}

default_to_features!(UvCodeActionFeatures, UvCodeActionFeatureTree);

impl UvCodeActionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn use_workspace_dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .use_workspace_dependency
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn add_to_workspace_and_use_workspace_dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .add_to_workspace_and_use_workspace_dependency
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct UvCodeActionFeatureTree {
    pub use_workspace_dependency: Option<ToggleFeature>,
    pub add_to_workspace_and_use_workspace_dependency: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub enum TombiExtensionFeatures {
    Enabled(EnabledOnly),
    Features(TombiExtensionFeatureTree),
}

default_to_features!(TombiExtensionFeatures, TombiExtensionFeatureTree);

impl TombiExtensionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn lsp(&self) -> Option<&TombiLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::completion_enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::path_completion_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::goto_definition_enabled)
    }

    pub fn path_goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::path_goto_definition_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::document_link_enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::path_document_link_enabled)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct TombiExtensionFeatureTree {
    pub lsp: Option<TombiLspFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiLspFeatures {
    Enabled(EnabledOnly),
    Features(TombiLspFeatureTree),
}

default_to_features!(TombiLspFeatures, TombiLspFeatureTree);

impl TombiLspFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn completion(&self) -> Option<&TombiCompletionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.completion.as_ref(),
        }
    }

    pub fn goto_definition(&self) -> Option<&TombiGotoDefinitionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_definition.as_ref(),
        }
    }

    pub fn document_link(&self) -> Option<&TombiDocumentLinkFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.document_link.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, TombiCompletionFeatures::enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, TombiCompletionFeatures::path_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, TombiGotoDefinitionFeatures::enabled)
    }

    pub fn path_goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, TombiGotoDefinitionFeatures::path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, TombiDocumentLinkFeatures::enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, TombiDocumentLinkFeatures::path_enabled)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct TombiLspFeatureTree {
    pub completion: Option<TombiCompletionFeatures>,
    pub goto_definition: Option<TombiGotoDefinitionFeatures>,
    pub document_link: Option<TombiDocumentLinkFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub enum TombiCompletionFeatures {
    Enabled(EnabledOnly),
    Features(TombiCompletionFeatureTree),
}

default_to_features!(TombiCompletionFeatures, TombiCompletionFeatureTree);

impl TombiCompletionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct TombiCompletionFeatureTree {
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub enum TombiGotoDefinitionFeatures {
    Enabled(EnabledOnly),
    Features(TombiGotoDefinitionFeatureTree),
}

default_to_features!(TombiGotoDefinitionFeatures, TombiGotoDefinitionFeatureTree);

impl TombiGotoDefinitionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct TombiGotoDefinitionFeatureTree {
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(TombiDocumentLinkFeatureTree),
}

default_to_features!(TombiDocumentLinkFeatures, TombiDocumentLinkFeatureTree);

impl TombiDocumentLinkFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
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
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct TombiDocumentLinkFeatureTree {
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ToggleFeature {
    pub enabled: Option<BoolDefaultTrue>,
}

impl ToggleFeature {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}
