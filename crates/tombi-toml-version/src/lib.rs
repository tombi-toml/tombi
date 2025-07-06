define_toml_version! {
    pub enum TomlVersion {
        #[default]
        V1_0_0 => "v1.0.0",
        V1_1_0_Preview => "v1.1.0-preview"
    }
}

impl TomlVersion {
    pub const fn latest() -> Self {
        Self::V1_1_0_Preview
    }
}

#[macro_export]
macro_rules! define_toml_version {
    (
        pub enum TomlVersion {
            $($(#[$attr:meta])* $variant:ident => $version:literal),* $(,)?
        }
    ) => {
        /// # TOML version.
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[allow(non_camel_case_types)]
        pub enum TomlVersion {
            $(
                $(#[$attr])*
                #[cfg_attr(feature = "serde", serde(rename = $version))]
                #[cfg_attr(feature = "clap", value(name = $version))]
                $variant,
            )*
        }

        impl std::fmt::Display for TomlVersion {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant => write!(f, $version),)*
                }
            }
        }

        impl std::str::FromStr for TomlVersion {
            type Err = ();

            #[allow(clippy::result_unit_err)]
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $(
                        $version => Ok(Self::$variant),
                    )*
                    _ => Err(()),
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn toml_version_comp() {
        assert!(crate::TomlVersion::V1_0_0 < crate::TomlVersion::V1_1_0_Preview);
    }
}
