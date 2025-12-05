use itertools::Itertools;
use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote};

use super::syntax_kind_src::{LITERALS, NODES, PUNCTUATIONS, TOKENS};

pub fn generate_syntax_kind() -> Result<String, anyhow::Error> {
    let punctuation_values = PUNCTUATIONS.iter().map(|item| {
        let token = item.token;
        match token {
            "{" | "}" | "[" | "]" => {
                let c = token.chars().next().unwrap();
                quote! { #c }
            }
            "[[" | "]]" => {
                quote!(#token)
            }
            _ => {
                let cs = token.chars().map(|c| Punct::new(c, Spacing::Alone));
                quote! { #(#cs)* }
            }
        }
    });
    let punctuations = PUNCTUATIONS
        .iter()
        .map(|item| format_ident!("{}", item.name))
        .collect_vec();
    let attr_punctuations = PUNCTUATIONS
        .iter()
        .map(|item| item.to_attr_token())
        .collect_vec();

    let attr_literals = LITERALS
        .iter()
        .map(|literal| format_ident!("{}", literal))
        .collect_vec();

    let attr_tokens = TOKENS
        .iter()
        .map(|token| format_ident!("{}", token))
        .collect_vec();

    let nodes = NODES
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect_vec();

    let token = quote! {
        #[doc = r" The kind of syntax node, e.g. `WHITESPACE`, `COMMENT`, or `TABLE`."]
        #[repr(u16)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[allow(non_camel_case_types)]
        pub enum SyntaxKind {
            #[doc(hidden)]
            TOMBSTONE,
            #[doc(hidden)]
            EOF,
            #(#attr_punctuations,)*
            #(#attr_literals,)*
            #(#attr_tokens,)*
            #(#nodes,)*
            #[doc(hidden)]
            INVALID_TOKEN,
            #[doc(hidden)]
            __LAST,
        }

        impl SyntaxKind {
            #[inline]
            pub fn is_trivia(self) -> bool {
                matches!(self, SyntaxKind::WHITESPACE)
            }
        }

        impl From<SyntaxKind> for tombi_rg_tree::SyntaxKind {
            #[inline]
            fn from(k: SyntaxKind) -> Self {
                Self(k as u16)
            }
        }

        impl From<u16> for SyntaxKind {
            #[inline]
            fn from(d: u16) -> SyntaxKind {
                debug_assert!(d <= (SyntaxKind::__LAST as u16));
                unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
            }
        }

        impl From<SyntaxKind> for u16 {
            #[inline]
            fn from(k: SyntaxKind) -> u16 {
                k as u16
            }
        }

        /// Utility macro for creating a SyntaxKind through simple macro syntax
        #[macro_export]
        macro_rules! T {
            // Punctuation
            #([#punctuation_values] => { $crate::SyntaxKind::#punctuations };)*
            // Bare key
            [bare_key] => { $crate::SyntaxKind::BARE_KEY };
            // String
            [basic_string] => { $crate::SyntaxKind::BASIC_STRING };
            [multi_line_basic_string] => { $crate::SyntaxKind::MULTI_LINE_BASIC_STRING };
            [literal_string] => { $crate::SyntaxKind::LITERAL_STRING };
            [multi_line_literal_string] => { $crate::SyntaxKind::MULTI_LINE_LITERAL_STRING };
            // Integer
            [integer_dec] => { $crate::SyntaxKind::INTEGER_DEC };
            [integer_hex] => { $crate::SyntaxKind::INTEGER_HEX };
            [integer_oct] => { $crate::SyntaxKind::INTEGER_OCT };
            [integer_bin] => { $crate::SyntaxKind::INTEGER_BIN };
            // Float
            [float] => { $crate::SyntaxKind::FLOAT };
            // Date and time
            [offset_date_time] => { $crate::SyntaxKind::OFFSET_DATE_TIME };
            [local_date_time] => { $crate::SyntaxKind::LOCAL_DATE_TIME };
            [local_date] => { $crate::SyntaxKind::LOCAL_DATE };
            [local_time] => { $crate::SyntaxKind::LOCAL_TIME };
        }
    };

    crate::utils::reformat(token)
}
