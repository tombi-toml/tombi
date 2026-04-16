use crate::{BoolDefaultFalse, BoolDefaultTrue};

pub const CARGO_EXTENSION_NAME: &str = "tombi-toml/cargo";
pub const PYPROJECT_EXTENSION_NAME: &str = "tombi-toml/pyproject";
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

mod cargo;
mod pyproject;
mod tombi;

pub use cargo::*;
pub use pyproject::*;
pub use tombi::*;

/// # Extension options
///
/// 🚧 Currently, third-party extensions are not supported,
/// and only built-in extensions are provided. 🚧
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub struct Extensions {
    #[cfg_attr(feature = "serde", serde(rename = "tombi-toml/cargo"))]
    /// # Cargo Extension
    ///
    /// Configure built-in support for `Cargo.toml`.
    pub cargo: Option<CargoExtensionFeatures>,

    #[cfg_attr(feature = "serde", serde(rename = "tombi-toml/pyproject"))]
    /// # Pyproject Extension
    ///
    /// Configure built-in support for `pyproject.toml`.
    pub pyproject: Option<PyprojectExtensionFeatures>,

    #[cfg_attr(feature = "serde", serde(rename = "tombi-toml/tombi"))]
    /// # Tombi Extension
    ///
    /// Configure built-in support for `tombi.toml`.
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

    pub fn pyproject_enabled(&self) -> bool {
        self.pyproject
            .as_ref()
            .map_or(true, PyprojectExtensionFeatures::enabled)
    }

    pub fn pyproject_features(&self) -> Option<&PyprojectExtensionFeatures> {
        self.pyproject.as_ref()
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
    /// # Enable feature
    ///
    /// Whether this feature is enabled.
    pub enabled: Option<BoolDefaultTrue>,
}

impl EnabledOnly {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ToggleFeatureDefaultTrue {
    /// # Enable feature
    ///
    /// Whether this nested feature is enabled.
    pub enabled: Option<BoolDefaultTrue>,
}

impl ToggleFeatureDefaultTrue {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ToggleFeatureDefaultFalse {
    /// # Enable feature
    ///
    /// Whether this nested feature is enabled.
    pub enabled: Option<BoolDefaultFalse>,
}

impl ToggleFeatureDefaultFalse {
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default().value()
    }
}
