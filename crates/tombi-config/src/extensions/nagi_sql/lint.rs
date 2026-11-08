use super::EnabledOnly;
use crate::{BoolDefaultTrue, SeverityLevel};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum NagiSqlLintFeatures {
    Enabled(EnabledOnly),
    Features(NagiSqlLintFeatureTree),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NagiSqlLintFeatureTree {
    pub rules: Option<NagiSqlLintRules>,
}

impl NagiSqlLintFeatureTree {
    pub fn disabled() -> Self {
        Self {
            rules: Some(NagiSqlLintRules::off()),
        }
    }
}

impl Default for NagiSqlLintFeatures {
    fn default() -> Self {
        Self::Features(NagiSqlLintFeatureTree::default())
    }
}

impl NagiSqlLintFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }
    pub fn rules(&self) -> Option<&NagiSqlLintRules> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.rules.as_ref(),
        }
    }
    pub fn feature_tree(&self) -> NagiSqlLintFeatureTree {
        match self {
            Self::Enabled(enabled) if !enabled.enabled.value() => {
                NagiSqlLintFeatureTree::disabled()
            }
            Self::Enabled(_) => NagiSqlLintFeatureTree::default(),
            Self::Features(features) => *features,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case", default)
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NagiSqlLintRules {
    /// Report workspace member patterns that match no configuration file.
    pub missing_member_config: SeverityLevel,
}

impl Default for NagiSqlLintRules {
    fn default() -> Self {
        Self {
            missing_member_config: SeverityLevel::Error,
        }
    }
}

impl NagiSqlLintRules {
    pub fn off() -> Self {
        Self {
            missing_member_config: SeverityLevel::Off,
        }
    }
}
