use quote::{format_ident, quote};

pub const PUNCTUATIONS: &[PunctuationItem] = &[
    PunctuationItem::new(",", "COMMA"),
    PunctuationItem::new(".", "DOT"),
    PunctuationItem::new("=", "EQUAL"),
    PunctuationItem::new("[", "BRACKET_START"),
    PunctuationItem::new("]", "BRACKET_END"),
    PunctuationItem::new("{", "BRACE_START"),
    PunctuationItem::new("}", "BRACE_END"),
    PunctuationItem::new("[[", "DOUBLE_BRACKET_START"),
    PunctuationItem::new("]]", "DOUBLE_BRACKET_END"),
];

pub const LITERALS: &[&str] = &[
    "BASIC_STRING",
    "MULTI_LINE_BASIC_STRING",
    "LITERAL_STRING",
    "MULTI_LINE_LITERAL_STRING",
    "INTEGER_DEC",
    "INTEGER_HEX",
    "INTEGER_OCT",
    "INTEGER_BIN",
    "FLOAT",
    "BOOLEAN",
    "OFFSET_DATE_TIME",
    "LOCAL_DATE_TIME",
    "LOCAL_DATE",
    "LOCAL_TIME",
];
pub const TOKENS: &[&str] = &["WHITESPACE", "LINE_BREAK", "COMMENT", "ERROR"];

pub const NODES: &[&str] = &[
    "ROOT",
    "KEYS",
    "KEY",
    "VALUE",
    "KEY_VALUE",
    "ARRAY",
    "TABLE",
    "INLINE_TABLE",
    "ARRAY_OF_TABLE",
];

#[derive(Debug)]
pub struct PunctuationItem<'a> {
    pub token: &'a str,
    pub name: &'a str,
}

impl<'a> PunctuationItem<'a> {
    pub const fn new(token: &'a str, name: &'a str) -> Self {
        Self { token, name }
    }

    pub fn to_attr_token(&self) -> proc_macro2::TokenStream {
        let name = format_ident!("{}", self.name);
        quote! { #name }
    }
}
