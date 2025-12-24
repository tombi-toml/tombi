define_toml_version! {
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
    #[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
    #[allow(non_camel_case_types)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum TomlVersion {
        #[default]
        V1_0_0 => "v1.0.0",
        #[deprecated(note = "Please use V1_1_0 instead")]
        V1_1_0_Preview => "v1.1.0-preview",
        V1_1_0 => "v1.1.0",
    }

    impl std::fmt::Display for TomlVersion;
    impl std::str::FromStr for TomlVersion;
}

impl TomlVersion {
    pub const fn latest() -> Self {
        Self::V1_1_0
    }
}

#[macro_export]
macro_rules! define_toml_version {
    (
        $(#[$attr:meta])*
        pub enum TomlVersion {
            $($(#[$variant_attr:meta])* $variant:ident => $version:literal),* $(,)?
        }

        impl std::fmt::Display for TomlVersion;
        impl std::str::FromStr for TomlVersion;
    ) => {
        /// # TOML version
        $(#[$attr])*
        pub enum TomlVersion {
            $(
                $(#[$variant_attr])*
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
        assert!(crate::TomlVersion::V1_1_0_Preview < crate::TomlVersion::V1_1_0);
    }
}
