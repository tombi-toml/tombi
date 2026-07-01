/// A schema's deprecation state.
///
/// A schema is deprecated only when `deprecated: true` is set; its mere presence
/// (`Option::is_some`) means deprecated. The VS Code `deprecationMessage` extension is
/// read only alongside `deprecated: true` — a `deprecationMessage` on its own (without
/// `deprecated: true`) is ignored (parsed to `None`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Deprecation {
    /// `deprecated: true` without a message.
    True,
    /// `deprecated: true` with a `deprecationMessage`, surfaced in diagnostics.
    Message(String),
}

impl Deprecation {
    /// Reads the deprecation state from a schema object, returning `None` when the
    /// schema is not deprecated (no `deprecated: true`).
    ///
    /// `deprecationMessage` is only honored when `deprecated: true` is also present.
    pub fn new(object: &tombi_json::ObjectNode) -> Option<Self> {
        if object.get("deprecated").and_then(|value| value.as_bool()) != Some(true) {
            return None;
        }

        match object
            .get("deprecationMessage")
            .and_then(|value| value.as_str())
        {
            Some(message) => Some(Self::Message(message.to_string())),
            None => Some(Self::True),
        }
    }

    /// The custom deprecation message, if any.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::True => None,
            Self::Message(message) => Some(message),
        }
    }
}
