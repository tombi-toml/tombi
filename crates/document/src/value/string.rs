use config::TomlVersion;
use document_tree::support;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringKind {
    BasicString,
    LiteralString,
    MultiLineBasicString,
    MultiLineLiteralString,
}

impl From<document_tree::StringKind> for StringKind {
    fn from(kind: document_tree::StringKind) -> Self {
        match kind {
            document_tree::StringKind::BasicString(_) => Self::BasicString,
            document_tree::StringKind::LiteralString(_) => Self::LiteralString,
            document_tree::StringKind::MultiLineBasicString(_) => Self::MultiLineBasicString,
            document_tree::StringKind::MultiLineLiteralString(_) => Self::MultiLineLiteralString,
        }
    }
}

impl From<&document_tree::StringKind> for StringKind {
    fn from(kind: &document_tree::StringKind) -> Self {
        match kind {
            document_tree::StringKind::BasicString(_) => Self::BasicString,
            document_tree::StringKind::LiteralString(_) => Self::LiteralString,
            document_tree::StringKind::MultiLineBasicString(_) => Self::MultiLineBasicString,
            document_tree::StringKind::MultiLineLiteralString(_) => Self::MultiLineLiteralString,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct String {
    kind: StringKind,
    value: std::string::String,
}

impl String {
    #[inline]
    pub fn new(kind: StringKind, value: std::string::String) -> Self {
        Self { kind, value }
    }

    #[inline]
    pub fn kind(&self) -> StringKind {
        self.kind
    }

    #[inline]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl From<document_tree::String> for String {
    fn from(node: document_tree::String) -> Self {
        Self {
            kind: node.kind().into(),
            value: node.value().to_string(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.kind {
            StringKind::BasicString => {
                support::string::try_from_basic_string(&self.value, TomlVersion::latest())
            }
            StringKind::LiteralString => support::string::try_from_literal_string(&self.value),
            StringKind::MultiLineBasicString => support::string::try_from_multi_line_basic_string(
                &self.value,
                TomlVersion::latest(),
            ),
            StringKind::MultiLineLiteralString => {
                support::string::try_from_multi_line_literal_string(&self.value)
            }
        }
        .map_err(|err| serde::ser::Error::custom(err))?
        .serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use config::TomlVersion;
    use serde_json::json;

    use crate::test_serialize;

    test_serialize! {
        #[test]
        fn escape_esc_v1_0_0(
            r#"
            esc = "\e There is no escape! \e"
            "#,
            TomlVersion::V1_0_0
        ) -> Err([
            ("invalid string: \\e is allowed in TOML v1.1.0 or later", ((0, 6), (0, 33)))
        ])
    }

    test_serialize! {
        #[test]
        fn escape_esc_v1_1_0(
            r#"
            esc = "\e There is no escape! \e"
            "#,
            TomlVersion::V1_1_0_Preview
        ) -> Ok(json!({"esc":"\u{001b} There is no escape! \u{001b}"}))
    }

    test_serialize!(
        #[test]
        fn escape_tricky(
            r#"
            end_esc = "String does not end here\" but ends here\\"
            lit_end_esc = 'String ends here\'

            multiline_unicode = """
            \u00a0"""

            multiline_not_unicode = """
            \\u0041"""

            multiline_end_esc = """When will it end? \"""...""\" should be here\""""

            lit_multiline_not_unicode = '''
            \u007f'''

            lit_multiline_end = '''There is no escape\'''
            "#
        ) -> Ok(json!({
            "end_esc": "String does not end here\" but ends here\\",
            "lit_end_esc": "String ends here\\",
            "multiline_unicode": "\u{00a0}",
            "multiline_not_unicode": "\\u0041",
            "multiline_end_esc": "When will it end? \"\"\"...\"\"\" should be here\"",
            "lit_multiline_not_unicode": "\\u007f",
            "lit_multiline_end": "There is no escape\\"
        }))
    );

    test_serialize!(
        #[test]
        fn multiline_empty(
            r#"
            empty-1 = """"""

            # A newline immediately following the opening delimiter will be trimmed.
            empty-2 = """
            """

            # \ at the end of line trims newlines as well; note that last \ is followed by
            # two spaces, which are ignored.
            empty-3 = """\
                """
            empty-4 = """\
                \
                \
                """
            "#
        ) -> Ok(json!({"empty-1":"","empty-2":"","empty-3":"","empty-4":""}))
    );

    test_serialize!(
        #[test]
        fn string_us(
            r#"
            string-us   = "null"
            "#
        ) -> Err([
            ("invalid string: invalid control character in input", ((0, 14), (0, 21)))
        ])
    );

    test_serialize!(
        #[test]
        fn rawstring_us(
            r#"
            rawstring-us   = 'null'
            "#
        ) -> Err([
            ("invalid string: invalid control character in input", ((0, 17), (0, 24)))
        ])
    );
}
