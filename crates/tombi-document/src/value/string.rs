pub use tombi_document_tree::StringKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct String {
    kind: StringKind,
    pub(crate) value: std::string::String,
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

impl From<tombi_document_tree::String> for crate::String {
    fn from(node: tombi_document_tree::String) -> Self {
        Self {
            kind: node.kind(),
            value: node.value().to_owned(),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for crate::String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tombi_toml_version::TomlVersion;

    use crate::test_deserialize;

    test_deserialize! {
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

    test_deserialize! {
        #[test]
        fn escape_esc_v1_1_0(
            r#"
            esc = "\e There is no escape! \e"
            "#,
            TomlVersion::V1_1_0_Preview
        ) -> Ok(json!({"esc":"\u{001b} There is no escape! \u{001b}"}))
    }

    test_deserialize! {
        #[test]
        fn escape_unicode_v1_0_0(
            r#"
            € = 'Euro'
            😂 = "rofl"
            "#,
            TomlVersion::V1_0_0
        ) -> Err([
            ("invalid string: unicode key is allowed in TOML v1.1.0 or later", ((0, 0), (0, 1))),
            ("invalid string: unicode key is allowed in TOML v1.1.0 or later", ((1, 0), (1, 1))),
        ])
    }

    test_deserialize! {
        #[test]
        fn escape_unicode_v1_1_0(
            r#"
            € = 'Euro'
            😂 = "rofl"
            "#,
            TomlVersion::V1_1_0_Preview
        ) -> Err([
            ("invalid string: unicode key is allowed in TOML v1.1.0 or later", ((0, 0), (0, 1))),
            ("invalid string: unicode key is allowed in TOML v1.1.0 or later", ((1, 0), (1, 1))),
        ])
    }

    // TODO: In toml `v1.1.0`, bare key unicode support was not merged and will likely be deferred for discussion in `v1.2.0`.
    //       See: https://github.com/toml-lang/toml/issues/954#issuecomment-1932268939
    //
    // test_deserialize! {
    //     #[test]
    //     fn escape_unicode_v1_1_0(
    //         r#"
    //         # TOML 1.1 supports Unicode for bare keys.

    //         € = 'Euro'
    //         😂 = "rofl"
    //         a‍b = "zwj"
    //         ÅÅ = "U+00C5 U+0041 U+030A"

    //         [中文]
    //         中文 = {中文 = "Chinese language"}

    //         [[tiếng-Việt]]
    //         tiəŋ˧˦.viət̚˧˨ʔ = "north"

    //         [[tiếng-Việt]]
    //         tiəŋ˦˧˥.viək̚˨˩ʔ = "central"
    //         "#,
    //         TomlVersion::V1_1_0_Preview
    //     ) -> Ok(json!({
    //         "€": "Euro",
    //         "😂": "rofl",
    //         "a‍b": "zwj",
    //         "ÅÅ": "U+00C5 U+0041 U+030A",
    //         "中文": {"中文": {"中文": "Chinese language"}},
    //         "tiếng-Việt": [
    //             {"tiəŋ˧˦": {"viət̚˧˨ʔ": "north"}},
    //             {"tiəŋ˦˧˥": {"viək̚˨˩ʔ": "central"}}
    //         ]
    //     }))
    // }

    test_deserialize!(
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

    test_deserialize! {
        #[test]
        fn hex_escape_v1_0_0(
            r#"
            # \x for the first 255 codepoints

            whitespace      = "\x20 \x09 \x1b \x0d\x0a"
            bs              = "\x7f"
            nul             = "\x00"
            hello           = "\x68\x65\x6c\x6c\x6f\x0a"
            higher-than-127 = "S\xf8rmirb\xe6ren"

            multiline = """
            \x20 \x09 \x1b \x0d\x0a
            \x7f
            \x00
            \x68\x65\x6c\x6c\x6f\x0a
            \x53\xF8\x72\x6D\x69\x72\x62\xE6\x72\x65\x6E
            """

            # Not inside literals.
            literal = '\x20 \x09 \x0d\x0a'
            multiline-literal = '''
            \x20 \x09 \x0d\x0a
            '''
            "#,
            TomlVersion::V1_0_0
        ) -> Err([
            ("invalid string: \\xXX is allowed in TOML v1.1.0 or later", ((2, 18), (2, 43))),
            ("invalid string: \\xXX is allowed in TOML v1.1.0 or later", ((3, 18), (3, 24))),
            ("invalid string: \\xXX is allowed in TOML v1.1.0 or later", ((4, 18), (4, 24))),
            ("invalid string: \\xXX is allowed in TOML v1.1.0 or later", ((5, 18), (5, 44))),
            ("invalid string: \\xXX is allowed in TOML v1.1.0 or later", ((6, 18), (6, 37))),
            ("invalid string: \\xXX is allowed in TOML v1.1.0 or later", ((8, 12), (14, 3))),
        ])
    }

    test_deserialize! {
        #[test]
        fn hex_escape_v1_1_0(
            r#"
            # \x for the first 255 codepoints

            whitespace      = "\x20 \x09 \x1b \x0d\x0a"
            bs              = "\x7f"
            nul             = "\x00"
            hello           = "\x68\x65\x6c\x6c\x6f\x0a"
            higher-than-127 = "S\xf8rmirb\xe6ren"

            multiline = """
            \x20 \x09 \x1b \x0d\x0a
            \x7f
            \x00
            \x68\x65\x6c\x6c\x6f\x0a
            \x53\xF8\x72\x6D\x69\x72\x62\xE6\x72\x65\x6E
            """

            # Not inside literals.
            literal = '\x20 \x09 \x0d\x0a'
            multiline-literal = '''
            \x20 \x09 \x0d\x0a
            '''
            "#,
            TomlVersion::V1_1_0_Preview
        ) -> Ok(json!({
            "whitespace": "  \t \u{001b} \r\n",
            "bs": "\u{007f}",
            "nul": "\u{0000}",
            "hello": "hello\n",
            "higher-than-127": "Sørmirbæren",
            "multiline": "  \t \x1b \r\n\n\x7f\n\x00\nhello\n\nSørmirbæren\n",
            "literal": "\\x20 \\x09 \\x0d\\x0a",
            "multiline-literal": "\\x20 \\x09 \\x0d\\x0a\n"
        }))
    }

    test_deserialize!(
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

    test_deserialize!(
        #[test]
        fn string_us(
            r#"
            string-us   = "null"
            "#
        ) -> Err([
            ("invalid string: invalid control character in input", ((0, 14), (0, 21)))
        ])
    );

    test_deserialize!(
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
