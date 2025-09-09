#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum SeverityLevel {
    Off,
    Warn,
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

        impl From<SeverityLevel> for $name {
            fn from(level: SeverityLevel) -> Self {
                Self(level)
            }
        }

        impl PartialEq<SeverityLevel> for $name {
            fn eq(&self, other: &SeverityLevel) -> bool {
                self.0 == *other
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
