/// Options for loading a Tombi config.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Do not fetch remote configs referenced by `extends`.
    pub offline: bool,
}
