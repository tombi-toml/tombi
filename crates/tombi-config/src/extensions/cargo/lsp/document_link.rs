use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

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
                    .map_or(true, ToggleFeatureDefaultTrue::enabled),
            }
    }

    pub fn workspace_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .workspace
                    .as_ref()
                    .map_or(true, ToggleFeatureDefaultTrue::enabled),
            }
    }

    pub fn git_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.git.as_ref().map_or(true, ToggleFeatureDefaultTrue::enabled)
                }
            }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.path.as_ref().map_or(true, ToggleFeatureDefaultTrue::enabled)
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
                    .map_or(true, ToggleFeatureDefaultTrue::enabled),
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
    pub cargo_toml: Option<ToggleFeatureDefaultTrue>,

    /// # crates.io document link feature
    ///
    /// Whether document links are created for crates.io package references.
    pub crates_io: Option<ToggleFeatureDefaultTrue>,

    /// # Git document link feature
    ///
    /// Whether document links are created for Git references.
    pub git: Option<ToggleFeatureDefaultTrue>,

    /// # Path document link feature
    ///
    /// Whether document links are created for filesystem paths.
    pub path: Option<ToggleFeatureDefaultTrue>,

    /// # Workspace document link feature
    ///
    /// Whether document links are created for `workspace = true` references.
    pub workspace: Option<ToggleFeatureDefaultTrue>,
}
