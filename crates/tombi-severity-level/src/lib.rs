#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum SeverityLevel {
    /// # Disable the Rule.
    Off,

    /// # Display as Warning.
    Warn,

    /// # Display as Error.
    Error,
}

macro_rules! severity_level_wrapper {
    ($name:ident, $level:ident, $default:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
        #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "jsonschema", schemars(extend("default" = $default)))]
        pub struct $name(SeverityLevel);

        impl Default for $name {
            fn default() -> Self {
                Self(SeverityLevel::$level)
            }
        }

        impl From<$name> for SeverityLevel {
            fn from(level: $name) -> Self {
                level.0
            }
        }
    };
}

severity_level_wrapper!(SeverityLevelDefaultWarn, Warn, "warn");
severity_level_wrapper!(SeverityLevelDefaultError, Error, "error");
severity_level_wrapper!(SeverityLevelDefaultOff, Off, "off");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value() {
        assert!(SeverityLevel::from(SeverityLevelDefaultWarn::default()) == SeverityLevel::Warn);
        assert!(SeverityLevel::from(SeverityLevelDefaultError::default()) == SeverityLevel::Error);
        assert!(SeverityLevel::from(SeverityLevelDefaultOff::default()) == SeverityLevel::Off);
    }
}
