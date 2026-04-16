use crate::{BoolDefaultFalse, BoolDefaultTrue};

pub const CARGO_EXTENSION_NAME: &str = "tombi-toml/cargo";
pub const PYPROJECT_EXTENSION_NAME: &str = "tombi-toml/pyproject";
pub const TOMBI_EXTENSION_NAME: &str = "tombi-toml/tombi";

macro_rules! extension_features {
    (
        $feature_enum:ident,

        $(#[$($meta:tt)*])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$($field_meta:tt)*])*
                $field_vis:vis $field_name:ident : Option<$field_type:ty>,
            )*
        }
    ) => {
        $(#[$($meta)*])*
        $vis struct $struct_name {
            $(
                $(#[$($field_meta)*])*
                $field_vis $field_name: Option<$field_type>,
            )*
        }

        impl Default for $feature_enum {
            fn default() -> Self {
                Self::Features($struct_name::default())
            }
        }

        impl $feature_enum {
            pub fn enabled(&self) -> $crate::BoolDefaultTrue {
                match self {
                    Self::Enabled(enabled) => enabled.enabled,
                    Self::Features(_) => Default::default(),
                }
            }

            $(
                pub fn $field_name(&self) -> Option<&$field_type> {
                    match self {
                        Self::Enabled(_) => None,
                        Self::Features(features) => features.$field_name.as_ref(),
                    }
                }
            )*
        }
    };
}

macro_rules! lsp_features {
    (
        $feature_enum:ident,

        $(#[$($meta:tt)*])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$($field_meta:tt)*])*
                $field_vis:vis $field_name:ident : Option<$field_type:ty>,
            )*
        }
    ) => {
        $(#[$($meta)*])*
        $vis struct $struct_name {
            $(
                $(#[$($field_meta)*])*
                $field_vis $field_name: Option<$field_type>,
            )*
        }

        impl Default for $feature_enum {
            fn default() -> Self {
                Self::Features($struct_name::default())
            }
        }

        impl $feature_enum {
            pub fn enabled(&self) -> $crate::BoolDefaultTrue {
                match self {
                    Self::Enabled(enabled) => enabled.enabled,
                    Self::Features(_) => Default::default(),
                }
            }

            $(
                pub fn $field_name(&self) -> Option<$field_type> {
                    match self {
                        Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                            <$field_type>::default()
                        } else {
                            <$field_type>::Enabled(enabled.clone())
                        }),
                        Self::Features(features) => features.$field_name.clone(),
                    }
                }
            )*
        }
    };
}

macro_rules! toggle_features {
    (
        $feature_enum:ident,

        $(#[$($meta:tt)*])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$($field_meta:tt)*])*
                $field_vis:vis $field_name:ident : Option<$field_type:ty>,
            )*
        }
    ) => {
        $(#[$($meta)*])*
        $vis struct $struct_name {
            $(
                $(#[$($field_meta)*])*
                $field_vis $field_name: Option<$field_type>,
            )*
        }

        impl Default for $feature_enum {
            fn default() -> Self {
                Self::Features($struct_name::default())
            }
        }

        impl $feature_enum {
            pub fn enabled(&self) -> $crate::BoolDefaultTrue {
                match self {
                    Self::Enabled(enabled) => enabled.enabled,
                    Self::Features(_) => Default::default(),
                }
            }

            $(
                pub fn $field_name(&self) -> Option<$field_type> {
                    match self {
                        Self::Enabled(enabled) => enabled.into(),
                        Self::Features(features) => features.$field_name,
                    }
                }
            )*
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
    pub fn cargo_enabled(&self) -> BoolDefaultTrue {
        self.cargo
            .as_ref()
            .map(|cargo| cargo.enabled())
            .unwrap_or_default()
    }

    pub fn cargo_features(&self) -> Option<&CargoExtensionFeatures> {
        self.cargo.as_ref()
    }

    pub fn pyproject_enabled(&self) -> BoolDefaultTrue {
        self.pyproject
            .as_ref()
            .map(|pyproject| pyproject.enabled())
            .unwrap_or_default()
    }

    pub fn pyproject_features(&self) -> Option<&PyprojectExtensionFeatures> {
        self.pyproject.as_ref()
    }

    pub fn tombi_enabled(&self) -> BoolDefaultTrue {
        self.tombi
            .as_ref()
            .map(|tombi| tombi.enabled())
            .unwrap_or_default()
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
    pub enabled: BoolDefaultTrue,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
    pub fn enabled(&self) -> BoolDefaultTrue {
        self.enabled.unwrap_or_default()
    }
}

impl From<&EnabledOnly> for Option<ToggleFeatureDefaultTrue> {
    fn from(enabled_only: &EnabledOnly) -> Self {
        Some(if enabled_only.enabled.value() {
            Default::default()
        } else {
            ToggleFeatureDefaultTrue {
                enabled: Some(BoolDefaultTrue(false)),
            }
        })
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
    pub fn enabled(&self) -> BoolDefaultFalse {
        self.enabled.unwrap_or_default()
    }
}

impl From<&EnabledOnly> for Option<ToggleFeatureDefaultFalse> {
    fn from(enabled_only: &EnabledOnly) -> Self {
        Some(if enabled_only.enabled.value() {
            Default::default()
        } else {
            ToggleFeatureDefaultFalse {
                enabled: Some(BoolDefaultFalse(false)),
            }
        })
    }
}
